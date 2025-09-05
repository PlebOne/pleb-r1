package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/pleb-one/api-gateway/internal/config"
	"github.com/pleb-one/api-gateway/internal/handlers"
	"github.com/pleb-one/api-gateway/internal/middleware"
	"github.com/pleb-one/api-gateway/internal/services"
	"github.com/pleb-one/api-gateway/pkg/database"
	"github.com/pleb-one/api-gateway/pkg/logger"
	"github.com/pleb-one/api-gateway/pkg/metrics"
	"github.com/rs/zerolog"
)

// @title Pleb.One API Gateway
// @version 1.0
// @description API Gateway for Pleb.One community-owned Nostr infrastructure
// @termsOfService https://pleb.one/terms

// @contact.name Pleb.One API Support
// @contact.url https://pleb.one/support
// @contact.email api@pleb.one

// @license.name MIT
// @license.url https://opensource.org/licenses/MIT

// @host api.pleb.one
// @BasePath /api/v1

// @securityDefinitions.apikey BearerAuth
// @in header
// @name Authorization
// @description Type "Bearer" followed by a space and JWT token.

func main() {
	// Load configuration
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	// Initialize logger
	logger := logger.New(cfg.Logger)
	
	// Initialize metrics
	metricsCollector := metrics.New()
	
	// Initialize database
	db, err := database.New(cfg.Database)
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to connect to database")
	}
	defer db.Close()

	// Run database migrations
	if err := db.Migrate(); err != nil {
		logger.Fatal().Err(err).Msg("Failed to run database migrations")
	}

	// Initialize services
	userService := services.NewUserService(db, logger)
	subscriptionService := services.NewSubscriptionService(db, logger)
	relayService := services.NewRelayService(cfg.Relay, logger)
	authService := services.NewAuthService(cfg.Auth, logger)

	// Initialize handlers
	userHandler := handlers.NewUserHandler(userService, logger)
	subscriptionHandler := handlers.NewSubscriptionHandler(subscriptionService, logger)
	relayHandler := handlers.NewRelayHandler(relayService, logger)
	authHandler := handlers.NewAuthHandler(authService, logger)
	healthHandler := handlers.NewHealthHandler(db, logger)

	// Setup Gin router
	if cfg.Environment == "production" {
		gin.SetMode(gin.ReleaseMode)
	}

	router := gin.New()

	// Global middleware
	router.Use(middleware.Logger(logger))
	router.Use(middleware.Recovery(logger))
	router.Use(middleware.CORS())
	router.Use(middleware.Metrics(metricsCollector))
	router.Use(middleware.RequestID())

	// Health check endpoints (no auth required)
	router.GET("/health", healthHandler.Health)
	router.GET("/ready", healthHandler.Ready)

	// Metrics endpoint
	router.GET("/metrics", gin.WrapH(metricsCollector.Handler()))

	// API routes
	v1 := router.Group("/api/v1")
	{
		// Authentication routes
		auth := v1.Group("/auth")
		{
			auth.POST("/register", authHandler.Register)
			auth.POST("/login", authHandler.Login)
			auth.POST("/refresh", authHandler.RefreshToken)
			auth.POST("/logout", middleware.Auth(authService), authHandler.Logout)
		}

		// User routes
		users := v1.Group("/users")
		users.Use(middleware.Auth(authService))
		{
			users.GET("/me", userHandler.GetCurrentUser)
			users.PUT("/me", userHandler.UpdateCurrentUser)
			users.GET("/:pubkey", userHandler.GetUser)
			users.GET("/:pubkey/subscriptions", subscriptionHandler.GetUserSubscriptions)
		}

		// Subscription routes
		subscriptions := v1.Group("/subscriptions")
		subscriptions.Use(middleware.Auth(authService))
		{
			subscriptions.POST("/", subscriptionHandler.CreateSubscription)
			subscriptions.GET("/:id", subscriptionHandler.GetSubscription)
			subscriptions.PUT("/:id", subscriptionHandler.UpdateSubscription)
			subscriptions.DELETE("/:id", subscriptionHandler.CancelSubscription)
			subscriptions.POST("/:id/renew", subscriptionHandler.RenewSubscription)
		}

		// Relay routes
		relay := v1.Group("/relay")
		{
			relay.GET("/status", relayHandler.GetStatus)
			relay.GET("/metrics", relayHandler.GetMetrics)
			relay.GET("/info", relayHandler.GetInfo)
		}

		// Admin routes (require admin role)
		admin := v1.Group("/admin")
		admin.Use(middleware.Auth(authService))
		admin.Use(middleware.RequireRole("admin"))
		{
			admin.GET("/users", userHandler.ListUsers)
			admin.GET("/subscriptions", subscriptionHandler.ListSubscriptions)
			admin.PUT("/users/:pubkey/role", userHandler.UpdateUserRole)
			admin.DELETE("/users/:pubkey", userHandler.DeleteUser)
		}
	}

	// Create HTTP server
	server := &http.Server{
		Addr:         fmt.Sprintf(":%d", cfg.Server.Port),
		Handler:      router,
		ReadTimeout:  cfg.Server.ReadTimeout,
		WriteTimeout: cfg.Server.WriteTimeout,
		IdleTimeout:  cfg.Server.IdleTimeout,
	}

	// Start server in a goroutine
	go func() {
		logger.Info().
			Int("port", cfg.Server.Port).
			Str("environment", cfg.Environment).
			Msg("Starting API Gateway server")

		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Fatal().Err(err).Msg("Failed to start server")
		}
	}()

	// Wait for interrupt signal to gracefully shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	logger.Info().Msg("Shutting down server...")

	// Give outstanding requests 30 seconds to complete
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		logger.Error().Err(err).Msg("Server forced to shutdown")
		return
	}

	logger.Info().Msg("Server exited")
}
