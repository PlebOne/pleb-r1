module github.com/pleb-one/api-gateway

go 1.21

require (
	github.com/gin-gonic/gin v1.9.1
	github.com/golang-jwt/jwt/v5 v5.0.0
	github.com/redis/go-redis/v9 v9.0.5
	github.com/lib/pq v1.10.9
	github.com/google/uuid v1.3.0
	github.com/prometheus/client_golang v1.17.0
	github.com/swaggo/gin-swagger v1.6.0
	github.com/swaggo/swag v1.16.1
	github.com/spf13/viper v1.16.0
	github.com/go-playground/validator/v10 v10.15.5
	github.com/rs/zerolog v1.31.0
	go.opentelemetry.io/otel v1.18.0
	go.opentelemetry.io/otel/trace v1.18.0
	go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin v0.44.0
)
