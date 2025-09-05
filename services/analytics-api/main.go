package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/go-redis/redis/v8"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

type AnalyticsAPI struct {
	db    *gorm.DB
	redis *redis.Client
}

type TrafficSummary struct {
	Date         string `json:"date"`
	TotalEvents  int64  `json:"total_events"`
	UniqueUsers  int64  `json:"unique_users"`
	EventTypes   map[string]int64 `json:"event_types"`
	AvgResponse  float64 `json:"avg_response_ms"`
}

type RealtimeDashboard struct {
	ActiveConnections    int64   `json:"active_connections"`
	EventsPerSecond     float64 `json:"events_per_second"`
	TotalEventsToday    int64   `json:"total_events_today"`
	UniqueUsersToday    int64   `json:"unique_users_today"`
	MemoryUsage         int64   `json:"memory_usage_mb"`
	CPUUsage           float64 `json:"cpu_usage_percent"`
	DiskUsage          int64   `json:"disk_usage_mb"`
	TopEventTypes      []EventTypeCount `json:"top_event_types"`
	RecentErrors       []ErrorEvent `json:"recent_errors"`
}

type EventTypeCount struct {
	EventType string `json:"event_type"`
	Count     int64  `json:"count"`
}

type ErrorEvent struct {
	Timestamp   time.Time `json:"timestamp"`
	EventType   string    `json:"event_type"`
	ErrorCode   string    `json:"error_code"`
	ClientID    string    `json:"client_id"`
	Message     string    `json:"message"`
}

type BandwidthReport struct {
	Period       string `json:"period"`
	TotalBytes   int64  `json:"total_bytes"`
	InboundBytes int64  `json:"inbound_bytes"`
	OutboundBytes int64 `json:"outbound_bytes"`
	PeakBandwidth int64 `json:"peak_bandwidth_bps"`
	AvgBandwidth  int64 `json:"avg_bandwidth_bps"`
}

type ClientMetrics struct {
	ClientID       string    `json:"client_id"`
	FirstSeen      time.Time `json:"first_seen"`
	LastSeen       time.Time `json:"last_seen"`
	TotalEvents    int64     `json:"total_events"`
	EventTypes     map[string]int64 `json:"event_types"`
	AvgResponseTime float64  `json:"avg_response_time_ms"`
	ErrorRate      float64   `json:"error_rate_percent"`
	TotalBandwidth int64     `json:"total_bandwidth_bytes"`
}

func NewAnalyticsAPI(dbURL, redisURL string) (*AnalyticsAPI, error) {
	// Connect to PostgreSQL
	db, err := gorm.Open(postgres.Open(dbURL), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %v", err)
	}

	// Connect to Redis
	opt, err := redis.ParseURL(redisURL)
	if err != nil {
		return nil, fmt.Errorf("failed to parse redis URL: %v", err)
	}
	rdb := redis.NewClient(opt)

	return &AnalyticsAPI{
		db:    db,
		redis: rdb,
	}, nil
}

func (api *AnalyticsAPI) setupRoutes() *gin.Engine {
	r := gin.Default()

	// Enable CORS
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowHeaders:     []string{"*"},
		ExposeHeaders:    []string{"Content-Length"},
		AllowCredentials: true,
		MaxAge:           12 * time.Hour,
	}))

	// Analytics API endpoints
	v1 := r.Group("/api/v1/analytics")
	{
		// Dashboard endpoints
		v1.GET("/dashboard/realtime", api.getRealtimeDashboard)
		v1.GET("/dashboard/summary", api.getDashboardSummary)
		
		// Reports
		v1.GET("/reports/traffic", api.getTrafficReport)
		v1.GET("/reports/bandwidth", api.getBandwidthReport)
		v1.GET("/reports/clients", api.getClientReport)
		v1.GET("/reports/errors", api.getErrorReport)
		
		// Export endpoints
		v1.GET("/export/csv", api.exportCSV)
		v1.GET("/export/json", api.exportJSON)
		
		// Custom queries
		v1.POST("/query", api.customQuery)
		
		// Client-specific analytics
		v1.GET("/clients/:client_id/metrics", api.getClientMetrics)
		v1.GET("/clients/:client_id/events", api.getClientEvents)
		
		// Time-series data for charts
		v1.GET("/timeseries/events", api.getEventTimeseries)
		v1.GET("/timeseries/bandwidth", api.getBandwidthTimeseries)
		v1.GET("/timeseries/connections", api.getConnectionTimeseries)
	}

	return r
}

func (api *AnalyticsAPI) getRealtimeDashboard(c *gin.Context) {
	dashboard := RealtimeDashboard{}
	
	// Get real-time metrics from Redis and database
	var result struct {
		ActiveConnections int64   `gorm:"column:active_connections"`
		EventsPerSecond  float64 `gorm:"column:events_per_second"`
		MemoryUsage      int64   `gorm:"column:memory_usage_bytes"`
		CPUUsage         float64 `gorm:"column:cpu_usage_percent"`
		DiskUsage        int64   `gorm:"column:disk_usage_bytes"`
	}
	
	api.db.Raw(`
		SELECT 
			active_connections,
			events_per_second,
			memory_usage_bytes,
			cpu_usage_percent,
			disk_usage_bytes
		FROM connection_metrics 
		ORDER BY timestamp DESC 
		LIMIT 1
	`).Scan(&result)
	
	dashboard.ActiveConnections = result.ActiveConnections
	dashboard.EventsPerSecond = result.EventsPerSecond
	dashboard.MemoryUsage = result.MemoryUsage / 1024 / 1024 // Convert to MB
	dashboard.CPUUsage = result.CPUUsage
	dashboard.DiskUsage = result.DiskUsage / 1024 / 1024 // Convert to MB
	
	// Get today's stats
	today := time.Now().Format("2006-01-02")
	api.db.Raw(`
		SELECT COUNT(*) as total_events, COUNT(DISTINCT client_id) as unique_users
		FROM traffic_events 
		WHERE DATE(timestamp) = ?
	`, today).Scan(&struct {
		TotalEvents int64 `gorm:"column:total_events"`
		UniqueUsers int64 `gorm:"column:unique_users"`
	}{
		TotalEvents: dashboard.TotalEventsToday,
		UniqueUsers: dashboard.UniqueUsersToday,
	})
	
	// Get top event types
	var eventTypes []EventTypeCount
	api.db.Raw(`
		SELECT event_type, COUNT(*) as count
		FROM traffic_events 
		WHERE timestamp >= NOW() - INTERVAL '1 hour'
		GROUP BY event_type 
		ORDER BY count DESC 
		LIMIT 5
	`).Scan(&eventTypes)
	dashboard.TopEventTypes = eventTypes
	
	// Get recent errors
	var errors []ErrorEvent
	api.db.Raw(`
		SELECT timestamp, event_type, error_code, client_id, 
			   COALESCE(metadata->>'message', 'Unknown error') as message
		FROM traffic_events 
		WHERE error_code IS NOT NULL 
		ORDER BY timestamp DESC 
		LIMIT 10
	`).Scan(&errors)
	dashboard.RecentErrors = errors
	
	c.JSON(http.StatusOK, dashboard)
}

func (api *AnalyticsAPI) getTrafficReport(c *gin.Context) {
	startDate := c.DefaultQuery("start_date", time.Now().AddDate(0, 0, -7).Format("2006-01-02"))
	endDate := c.DefaultQuery("end_date", time.Now().Format("2006-01-02"))
	granularity := c.DefaultQuery("granularity", "day")
	
	var reports []TrafficSummary
	
	var dateFormat string
	switch granularity {
	case "hour":
		dateFormat = "DATE_TRUNC('hour', timestamp)"
	case "day":
		dateFormat = "DATE(timestamp)"
	case "week":
		dateFormat = "DATE_TRUNC('week', timestamp)"
	case "month":
		dateFormat = "DATE_TRUNC('month', timestamp)"
	default:
		dateFormat = "DATE(timestamp)"
	}
	
	api.db.Raw(fmt.Sprintf(`
		SELECT 
			%s as date,
			COUNT(*) as total_events,
			COUNT(DISTINCT client_id) as unique_users,
			AVG(response_time_ms) as avg_response
		FROM traffic_events 
		WHERE DATE(timestamp) BETWEEN ? AND ?
		GROUP BY %s
		ORDER BY date
	`, dateFormat, dateFormat), startDate, endDate).Scan(&reports)
	
	// Get event types for each period
	for i := range reports {
		var eventTypes []struct {
			EventType string `gorm:"column:event_type"`
			Count     int64  `gorm:"column:count"`
		}
		
		api.db.Raw(`
			SELECT event_type, COUNT(*) as count
			FROM traffic_events 
			WHERE DATE(timestamp) = ? 
			GROUP BY event_type
		`, reports[i].Date).Scan(&eventTypes)
		
		reports[i].EventTypes = make(map[string]int64)
		for _, et := range eventTypes {
			reports[i].EventTypes[et.EventType] = et.Count
		}
	}
	
	c.JSON(http.StatusOK, reports)
}

func (api *AnalyticsAPI) getBandwidthReport(c *gin.Context) {
	startDate := c.DefaultQuery("start_date", time.Now().AddDate(0, 0, -7).Format("2006-01-02"))
	endDate := c.DefaultQuery("end_date", time.Now().Format("2006-01-02"))
	
	var report BandwidthReport
	
	api.db.Raw(`
		SELECT 
			COALESCE(SUM(bytes_transferred), 0) as total_bytes,
			MAX(bytes_transferred) as peak_bandwidth,
			AVG(bytes_transferred) as avg_bandwidth
		FROM traffic_events 
		WHERE DATE(timestamp) BETWEEN ? AND ?
	`, startDate, endDate).Scan(&report)
	
	report.Period = fmt.Sprintf("%s to %s", startDate, endDate)
	report.InboundBytes = report.TotalBytes / 2  // Simplified estimation
	report.OutboundBytes = report.TotalBytes / 2
	report.PeakBandwidth = report.PeakBandwidth * 8 // Convert to bits per second
	report.AvgBandwidth = report.AvgBandwidth * 8
	
	c.JSON(http.StatusOK, report)
}

func (api *AnalyticsAPI) getClientMetrics(c *gin.Context) {
	clientID := c.Param("client_id")
	
	var metrics ClientMetrics
	metrics.ClientID = clientID
	
	// Get basic client stats
	api.db.Raw(`
		SELECT 
			MIN(timestamp) as first_seen,
			MAX(timestamp) as last_seen,
			COUNT(*) as total_events,
			AVG(response_time_ms) as avg_response_time,
			COALESCE(SUM(bytes_transferred), 0) as total_bandwidth,
			(COUNT(CASE WHEN error_code IS NOT NULL THEN 1 END) * 100.0 / COUNT(*)) as error_rate
		FROM traffic_events 
		WHERE client_id = ?
	`, clientID).Scan(&metrics)
	
	// Get event types breakdown
	var eventTypes []struct {
		EventType string `gorm:"column:event_type"`
		Count     int64  `gorm:"column:count"`
	}
	
	api.db.Raw(`
		SELECT event_type, COUNT(*) as count
		FROM traffic_events 
		WHERE client_id = ?
		GROUP BY event_type
	`, clientID).Scan(&eventTypes)
	
	metrics.EventTypes = make(map[string]int64)
	for _, et := range eventTypes {
		metrics.EventTypes[et.EventType] = et.Count
	}
	
	c.JSON(http.StatusOK, metrics)
}

func (api *AnalyticsAPI) exportCSV(c *gin.Context) {
	startDate := c.DefaultQuery("start_date", time.Now().AddDate(0, 0, -7).Format("2006-01-02"))
	endDate := c.DefaultQuery("end_date", time.Now().Format("2006-01-02"))
	
	// Call the Rust analytics service for CSV export
	resp, err := http.Get(fmt.Sprintf("http://localhost:8003/reports/export?start_date=%s&end_date=%s", startDate, endDate))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to generate CSV report"})
		return
	}
	defer resp.Body.Close()
	
	c.Header("Content-Type", "text/csv")
	c.Header("Content-Disposition", fmt.Sprintf("attachment; filename=traffic_report_%s_to_%s.csv", startDate, endDate))
	
	c.DataFromReader(http.StatusOK, resp.ContentLength, "text/csv", resp.Body, nil)
}

func (api *AnalyticsAPI) getEventTimeseries(c *gin.Context) {
	hours, _ := strconv.Atoi(c.DefaultQuery("hours", "24"))
	
	var timeseries []struct {
		Hour   string `gorm:"column:hour" json:"timestamp"`
		Events int64  `gorm:"column:events" json:"events"`
	}
	
	api.db.Raw(`
		SELECT 
			DATE_TRUNC('hour', timestamp) as hour,
			COUNT(*) as events
		FROM traffic_events 
		WHERE timestamp >= NOW() - INTERVAL '%d hours'
		GROUP BY hour
		ORDER BY hour
	`, hours).Scan(&timeseries)
	
	c.JSON(http.StatusOK, timeseries)
}

func main() {
	dbURL := "postgres://user:password@localhost:5432/nrelay_analytics?sslmode=disable"
	redisURL := "redis://localhost:6379"
	
	api, err := NewAnalyticsAPI(dbURL, redisURL)
	if err != nil {
		log.Fatal("Failed to initialize analytics API:", err)
	}
	
	r := api.setupRoutes()
	
	fmt.Println("Analytics API server starting on :8002")
	if err := r.Run(":8002"); err != nil {
		log.Fatal("Failed to start server:", err)
	}
}
