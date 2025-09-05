package main

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"log"
	"net/http"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
	"github.com/nbd-wtf/go-nostr"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

type BlossomServer struct {
	db          *gorm.DB
	redis       *redis.Client
	minioClient *minio.Client
	config      Config
}

type Config struct {
	ServerPort     string `env:"SERVER_PORT" envDefault:"8004"`
	DatabaseURL    string `env:"DATABASE_URL" required:"true"`
	RedisURL       string `env:"REDIS_URL" required:"true"`
	MinioEndpoint  string `env:"MINIO_ENDPOINT" envDefault:"localhost:9000"`
	MinioAccessKey string `env:"MINIO_ACCESS_KEY" required:"true"`
	MinioSecretKey string `env:"MINIO_SECRET_KEY" required:"true"`
	MinioBucket    string `env:"MINIO_BUCKET" envDefault:"blossom-files"`
	MaxFileSize    int64  `env:"MAX_FILE_SIZE" envDefault:"104857600"` // 100MB
	AllowedTypes   string `env:"ALLOWED_TYPES" envDefault:"image/jpeg,image/png,image/gif,image/webp,video/mp4,audio/mpeg"`
}

type FileMetadata struct {
	ID          uint      `gorm:"primaryKey" json:"id"`
	Hash        string    `gorm:"uniqueIndex;not null" json:"sha256"`
	Size        int64     `gorm:"not null" json:"size"`
	MimeType    string    `gorm:"not null" json:"type"`
	FileName    string    `gorm:"not null" json:"name"`
	UserPubkey  string    `gorm:"index;not null" json:"uploaded_by"`
	UploadedAt  time.Time `gorm:"autoCreateTime" json:"uploaded_at"`
	AccessCount int64     `gorm:"default:0" json:"access_count"`
	IsPublic    bool      `gorm:"default:true" json:"is_public"`
	Tags        string    `json:"tags,omitempty"`
}

type BlossomResponse struct {
	Message string `json:"message,omitempty"`
	SHA256  string `json:"sha256,omitempty"`
	Size    int64  `json:"size,omitempty"`
	Type    string `json:"type,omitempty"`
	URL     string `json:"url,omitempty"`
}

func NewBlossomServer(config Config) (*BlossomServer, error) {
	// Connect to PostgreSQL
	db, err := gorm.Open(postgres.Open(config.DatabaseURL), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %v", err)
	}

	// Auto-migrate the schema
	if err := db.AutoMigrate(&FileMetadata{}); err != nil {
		return nil, fmt.Errorf("failed to migrate database: %v", err)
	}

	// Connect to Redis
	opt, err := redis.ParseURL(config.RedisURL)
	if err != nil {
		return nil, fmt.Errorf("failed to parse redis URL: %v", err)
	}
	rdb := redis.NewClient(opt)

	// Connect to MinIO
	minioClient, err := minio.New(config.MinioEndpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(config.MinioAccessKey, config.MinioSecretKey, ""),
		Secure: false, // Set to true for HTTPS
	})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to MinIO: %v", err)
	}

	// Ensure bucket exists
	err = minioClient.MakeBucket(nil, config.MinioBucket, minio.MakeBucketOptions{})
	if err != nil {
		// Check if bucket already exists
		exists, errBucketExists := minioClient.BucketExists(nil, config.MinioBucket)
		if errBucketExists == nil && exists {
			log.Printf("Bucket %s already exists", config.MinioBucket)
		} else {
			return nil, fmt.Errorf("failed to create bucket: %v", err)
		}
	}

	return &BlossomServer{
		db:          db,
		redis:       rdb,
		minioClient: minioClient,
		config:      config,
	}, nil
}

func (bs *BlossomServer) setupRoutes() *gin.Engine {
	r := gin.Default()

	// Enable CORS
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD"},
		AllowHeaders:     []string{"*"},
		ExposeHeaders:    []string{"Content-Length", "Content-Type"},
		AllowCredentials: true,
		MaxAge:           12 * time.Hour,
	}))

	// Set max multipart memory (for file uploads)
	r.MaxMultipartMemory = bs.config.MaxFileSize

	// Blossom BUD-01 endpoints
	r.PUT("/upload", bs.uploadFile)
	r.GET("/:sha256", bs.getFile)
	r.HEAD("/:sha256", bs.getFileInfo)
	r.DELETE("/:sha256", bs.deleteFile)

	// Additional endpoints for management
	r.GET("/list", bs.listFiles)
	r.GET("/stats", bs.getStats)

	return r
}

func (bs *BlossomServer) uploadFile(c *gin.Context) {
	// Check if user is authenticated (Premium tier required)
	userPubkey := c.GetHeader("X-User-Pubkey")
	if userPubkey == "" {
		c.JSON(http.StatusUnauthorized, BlossomResponse{
			Message: "Authentication required for file upload",
		})
		return
	}

	// Verify Nostr auth event if present
	authHeader := c.GetHeader("Authorization")
	if authHeader != "" {
		if !bs.verifyNostrAuth(authHeader, userPubkey) {
			c.JSON(http.StatusUnauthorized, BlossomResponse{
				Message: "Invalid Nostr authentication",
			})
			return
		}
	}

	// Check if user has premium subscription
	if !bs.hasPremiumSubscription(userPubkey) {
		c.JSON(http.StatusForbidden, BlossomResponse{
			Message: "Premium subscription required for file uploads",
		})
		return
	}

	// Get file from form
	file, header, err := c.Request.FormFile("file")
	if err != nil {
		c.JSON(http.StatusBadRequest, BlossomResponse{
			Message: "No file provided",
		})
		return
	}
	defer file.Close()

	// Check file size
	if header.Size > bs.config.MaxFileSize {
		c.JSON(http.StatusRequestEntityTooLarge, BlossomResponse{
			Message: fmt.Sprintf("File too large. Max size: %d bytes", bs.config.MaxFileSize),
		})
		return
	}

	// Check MIME type
	if !bs.isAllowedMimeType(header.Header.Get("Content-Type")) {
		c.JSON(http.StatusUnsupportedMediaType, BlossomResponse{
			Message: "File type not allowed",
		})
		return
	}

	// Read file content
	fileContent, err := io.ReadAll(file)
	if err != nil {
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to read file",
		})
		return
	}

	// Calculate SHA256 hash
	hash := sha256.Sum256(fileContent)
	hashStr := hex.EncodeToString(hash[:])

	// Check if file already exists
	var existingFile FileMetadata
	if err := bs.db.Where("hash = ?", hashStr).First(&existingFile).Error; err == nil {
		// File already exists, return existing info
		c.JSON(http.StatusOK, BlossomResponse{
			Message: "File already exists",
			SHA256:  existingFile.Hash,
			Size:    existingFile.Size,
			Type:    existingFile.MimeType,
			URL:     fmt.Sprintf("/uploads/%s", existingFile.Hash),
		})
		return
	}

	// Upload to MinIO
	_, err = bs.minioClient.PutObject(
		nil,
		bs.config.MinioBucket,
		hashStr,
		strings.NewReader(string(fileContent)),
		int64(len(fileContent)),
		minio.PutObjectOptions{ContentType: header.Header.Get("Content-Type")},
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to store file",
		})
		return
	}

	// Save metadata to database
	fileMetadata := FileMetadata{
		Hash:       hashStr,
		Size:       header.Size,
		MimeType:   header.Header.Get("Content-Type"),
		FileName:   header.Filename,
		UserPubkey: userPubkey,
		IsPublic:   true,
	}

	if err := bs.db.Create(&fileMetadata).Error; err != nil {
		// If database save fails, try to clean up MinIO
		bs.minioClient.RemoveObject(nil, bs.config.MinioBucket, hashStr, minio.RemoveObjectOptions{})
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to save file metadata",
		})
		return
	}

	// Cache file info in Redis
	bs.cacheFileInfo(hashStr, fileMetadata)

	c.JSON(http.StatusCreated, BlossomResponse{
		Message: "File uploaded successfully",
		SHA256:  hashStr,
		Size:    header.Size,
		Type:    header.Header.Get("Content-Type"),
		URL:     fmt.Sprintf("/uploads/%s", hashStr),
	})
}

func (bs *BlossomServer) getFile(c *gin.Context) {
	sha256Hash := c.Param("sha256")

	// Get file metadata
	var fileMetadata FileMetadata
	if err := bs.db.Where("hash = ?", sha256Hash).First(&fileMetadata).Error; err != nil {
		c.JSON(http.StatusNotFound, BlossomResponse{
			Message: "File not found",
		})
		return
	}

	// Increment access count
	bs.db.Model(&fileMetadata).Update("access_count", fileMetadata.AccessCount+1)

	// Get file from MinIO
	object, err := bs.minioClient.GetObject(nil, bs.config.MinioBucket, sha256Hash, minio.GetObjectOptions{})
	if err != nil {
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to retrieve file",
		})
		return
	}
	defer object.Close()

	// Set headers
	c.Header("Content-Type", fileMetadata.MimeType)
	c.Header("Content-Length", strconv.FormatInt(fileMetadata.Size, 10))
	c.Header("Content-Disposition", fmt.Sprintf("inline; filename=\"%s\"", fileMetadata.FileName))
	c.Header("Cache-Control", "public, max-age=31536000") // 1 year cache

	// Stream file to response
	io.Copy(c.Writer, object)
}

func (bs *BlossomServer) getFileInfo(c *gin.Context) {
	sha256Hash := c.Param("sha256")

	// Check cache first
	if cached := bs.getCachedFileInfo(sha256Hash); cached != nil {
		c.Header("Content-Type", cached.MimeType)
		c.Header("Content-Length", strconv.FormatInt(cached.Size, 10))
		c.Header("X-SHA256", cached.Hash)
		c.Status(http.StatusOK)
		return
	}

	// Get from database
	var fileMetadata FileMetadata
	if err := bs.db.Where("hash = ?", sha256Hash).First(&fileMetadata).Error; err != nil {
		c.Status(http.StatusNotFound)
		return
	}

	// Cache for future requests
	bs.cacheFileInfo(sha256Hash, fileMetadata)

	c.Header("Content-Type", fileMetadata.MimeType)
	c.Header("Content-Length", strconv.FormatInt(fileMetadata.Size, 10))
	c.Header("X-SHA256", fileMetadata.Hash)
	c.Status(http.StatusOK)
}

func (bs *BlossomServer) deleteFile(c *gin.Context) {
	sha256Hash := c.Param("sha256")
	userPubkey := c.GetHeader("X-User-Pubkey")

	if userPubkey == "" {
		c.JSON(http.StatusUnauthorized, BlossomResponse{
			Message: "Authentication required",
		})
		return
	}

	// Get file metadata
	var fileMetadata FileMetadata
	if err := bs.db.Where("hash = ? AND user_pubkey = ?", sha256Hash, userPubkey).First(&fileMetadata).Error; err != nil {
		c.JSON(http.StatusNotFound, BlossomResponse{
			Message: "File not found or not owned by user",
		})
		return
	}

	// Delete from MinIO
	err := bs.minioClient.RemoveObject(nil, bs.config.MinioBucket, sha256Hash, minio.RemoveObjectOptions{})
	if err != nil {
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to delete file from storage",
		})
		return
	}

	// Delete from database
	if err := bs.db.Delete(&fileMetadata).Error; err != nil {
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to delete file metadata",
		})
		return
	}

	// Remove from cache
	bs.redis.Del(nil, fmt.Sprintf("file:%s", sha256Hash))

	c.JSON(http.StatusOK, BlossomResponse{
		Message: "File deleted successfully",
	})
}

func (bs *BlossomServer) listFiles(c *gin.Context) {
	userPubkey := c.GetHeader("X-User-Pubkey")
	if userPubkey == "" {
		c.JSON(http.StatusUnauthorized, BlossomResponse{
			Message: "Authentication required",
		})
		return
	}

	var files []FileMetadata
	if err := bs.db.Where("user_pubkey = ?", userPubkey).Find(&files).Error; err != nil {
		c.JSON(http.StatusInternalServerError, BlossomResponse{
			Message: "Failed to retrieve files",
		})
		return
	}

	c.JSON(http.StatusOK, files)
}

func (bs *BlossomServer) getStats(c *gin.Context) {
	var stats struct {
		TotalFiles int64 `json:"total_files"`
		TotalSize  int64 `json:"total_size"`
		TotalUsers int64 `json:"total_users"`
	}

	bs.db.Model(&FileMetadata{}).Count(&stats.TotalFiles)
	bs.db.Model(&FileMetadata{}).Select("COALESCE(SUM(size), 0)").Scan(&stats.TotalSize)
	bs.db.Model(&FileMetadata{}).Distinct("user_pubkey").Count(&stats.TotalUsers)

	c.JSON(http.StatusOK, stats)
}

// Helper functions
func (bs *BlossomServer) verifyNostrAuth(authHeader, userPubkey string) bool {
	// Parse Nostr auth event from Authorization header
	// This would implement NIP-98 HTTP Auth
	// For now, we'll return true if the header exists
	return strings.HasPrefix(authHeader, "Nostr ")
}

func (bs *BlossomServer) hasPremiumSubscription(pubkey string) bool {
	// Check if user has active premium subscription
	// This would query the billing service or database
	// For now, we'll assume all authenticated users have premium
	return pubkey != ""
}

func (bs *BlossomServer) isAllowedMimeType(mimeType string) bool {
	allowedTypes := strings.Split(bs.config.AllowedTypes, ",")
	for _, allowed := range allowedTypes {
		if strings.TrimSpace(allowed) == mimeType {
			return true
		}
	}
	return false
}

func (bs *BlossomServer) cacheFileInfo(hash string, metadata FileMetadata) {
	key := fmt.Sprintf("file:%s", hash)
	bs.redis.HSet(nil, key,
		"mime_type", metadata.MimeType,
		"size", metadata.Size,
		"hash", metadata.Hash,
	)
	bs.redis.Expire(nil, key, 24*time.Hour)
}

func (bs *BlossomServer) getCachedFileInfo(hash string) *FileMetadata {
	key := fmt.Sprintf("file:%s", hash)
	result := bs.redis.HGetAll(nil, key)
	if result.Err() != nil || len(result.Val()) == 0 {
		return nil
	}

	data := result.Val()
	size, _ := strconv.ParseInt(data["size"], 10, 64)

	return &FileMetadata{
		Hash:     data["hash"],
		Size:     size,
		MimeType: data["mime_type"],
	}
}

func main() {
	config := Config{
		ServerPort:     "8004",
		DatabaseURL:    "postgres://pleb_one:development@localhost:5432/pleb_one?sslmode=disable",
		RedisURL:       "redis://localhost:6379",
		MinioEndpoint:  "localhost:9000",
		MinioAccessKey: "minioadmin",
		MinioSecretKey: "minioadmin",
		MinioBucket:    "blossom-files",
		MaxFileSize:    104857600, // 100MB
		AllowedTypes:   "image/jpeg,image/png,image/gif,image/webp,video/mp4,audio/mpeg,text/plain",
	}

	server, err := NewBlossomServer(config)
	if err != nil {
		log.Fatal("Failed to initialize Blossom server:", err)
	}

	r := server.setupRoutes()

	fmt.Printf("🌸 Blossom server starting on port %s\n", config.ServerPort)
	log.Fatal(r.Run(":" + config.ServerPort))
}
