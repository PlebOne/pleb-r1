# Analytics & Reporting Features

## 📊 **Comprehensive Traffic Analytics**

### **Real-time Dashboard**
- **Active Connections**: Live count of WebSocket connections
- **Events Per Second**: Real-time throughput monitoring
- **Daily Statistics**: Total events and unique users today
- **System Metrics**: CPU, memory, and disk usage
- **Event Types Breakdown**: Distribution of different event types
- **Recent Errors**: Latest errors with details and timestamps

### **Traffic Reports**
- **Time-series Data**: Events over time with customizable granularity (hour/day/week/month)
- **User Analytics**: Unique user counts and retention
- **Event Type Analysis**: Breakdown of EVENT types, REQ, CLOSE, etc.
- **Response Times**: Performance metrics with percentiles (P50, P95, P99)
- **Error Rates**: Failure analysis and error trends

### **Bandwidth Monitoring**
- **Data Transfer**: Inbound and outbound bandwidth usage
- **Peak Usage**: Maximum bandwidth periods
- **Client Bandwidth**: Per-client data consumption
- **Cost Analysis**: Bandwidth costs for pricing optimization

### **Client-Specific Analytics**
- **Individual Client Metrics**: Per-client event counts and patterns
- **Usage Patterns**: First seen, last seen, activity trends
- **Performance**: Client-specific response times and error rates
- **Billing Data**: Usage for subscription billing

## 🔧 **Technical Implementation**

### **Rust Analytics Service** (`analytics-service`)
- **High-Performance Collection**: Efficient event ingestion
- **TimescaleDB Integration**: Time-series optimized storage
- **Real-time Processing**: Live metrics calculation
- **Background Jobs**: Automated report generation

### **Go Analytics API** (`analytics-api`)
- **RESTful Endpoints**: Easy frontend integration
- **Dashboard APIs**: Real-time data for dashboards
- **Export Functions**: CSV and JSON data export
- **Query Interface**: Flexible data retrieval

### **Database Schema**
```sql
-- Events tracking with time-series optimization
traffic_events (
    id, event_id, client_id, event_type, 
    timestamp, metadata, response_time_ms, 
    bytes_transferred, error_code
)

-- System metrics over time  
connection_metrics (
    timestamp, active_connections, peak_connections,
    events_per_second, memory_usage, cpu_usage
)
```

## 📈 **Available Reports**

### **1. Real-time Dashboard**
```
GET /api/v1/analytics/dashboard/realtime
```
- Live system status
- Current performance metrics
- Active connections and throughput

### **2. Traffic Reports**
```
GET /api/v1/analytics/reports/traffic?start_date=2024-01-01&end_date=2024-01-07&granularity=day
```
- Historical traffic analysis
- User growth and retention
- Event type distribution

### **3. Bandwidth Reports**
```
GET /api/v1/analytics/reports/bandwidth?start_date=2024-01-01&end_date=2024-01-07
```
- Data transfer analysis
- Cost calculation assistance
- Peak usage identification

### **4. Client Analytics**
```
GET /api/v1/analytics/clients/{client_id}/metrics
```
- Individual client performance
- Usage patterns and trends
- Billing-relevant data

### **5. Time-series Data**
```
GET /api/v1/analytics/timeseries/events?hours=24
GET /api/v1/analytics/timeseries/bandwidth?hours=24
GET /api/v1/analytics/timeseries/connections?hours=24
```
- Chart-ready data
- Customizable time ranges
- Multiple metric types

### **6. Export Functions**
```
GET /api/v1/analytics/export/csv?start_date=2024-01-01&end_date=2024-01-07
GET /api/v1/analytics/export/json?start_date=2024-01-01&end_date=2024-01-07
```
- Raw data export
- Compliance reporting
- External analysis tools

## 🎯 **Business Benefits**

### **For Relay Operations**
- **Performance Monitoring**: Identify bottlenecks and optimize
- **Capacity Planning**: Predict scaling needs
- **Error Detection**: Proactive issue identification
- **User Experience**: Monitor and improve response times

### **For Business Management**
- **Usage Tracking**: Understand client behavior
- **Billing Accuracy**: Precise usage measurement
- **Growth Analysis**: User acquisition and retention
- **Cost Optimization**: Bandwidth and resource management

### **For Competitive Advantage**
- **Transparency**: Detailed usage reporting for clients
- **Performance Proof**: Demonstrate relay reliability
- **Data-Driven Decisions**: Optimize based on real usage
- **Professional Service**: Enterprise-grade analytics

## 🚀 **Usage Examples**

### **Start Analytics Services**
```bash
# Start analytics collection service
make dev-analytics

# Start analytics API server  
make dev-analytics-api

# Or start everything together
make dev
```

### **View Dashboard**
Access the real-time dashboard at:
```
http://localhost:3000/dashboard/analytics
```

### **Export Monthly Report**
```bash
curl "http://localhost:8002/api/v1/analytics/export/csv?start_date=2024-01-01&end_date=2024-01-31" \
  -H "Authorization: Bearer $TOKEN" \
  -o monthly_report.csv
```

### **Monitor Specific Client**
```bash
curl "http://localhost:8002/api/v1/analytics/clients/client_123/metrics" \
  -H "Authorization: Bearer $TOKEN"
```

## 📊 **Dashboard Features**

### **Charts and Visualizations**
- **Line Charts**: Event timeline and trends
- **Bar Charts**: Daily/weekly traffic comparison  
- **Pie Charts**: Event type distribution
- **Metrics Cards**: Key performance indicators
- **Error Lists**: Recent issues with details

### **Filters and Controls**
- **Time Range Selection**: 24h, 7d, 30d, custom
- **Event Type Filtering**: Focus on specific events
- **Client Filtering**: Analyze specific users
- **Granularity Control**: Hour, day, week, month views

### **Export and Sharing**
- **CSV Export**: Spreadsheet-compatible data
- **JSON Export**: API-compatible format
- **Direct Links**: Shareable dashboard URLs
- **Scheduled Reports**: Automated report generation

## 🔒 **Security and Privacy**

- **Authentication Required**: All endpoints protected
- **Client Data Privacy**: Anonymized where possible
- **Data Retention**: Configurable retention policies
- **Access Controls**: Role-based analytics access

This comprehensive analytics system gives you complete visibility into your relay's performance and usage, enabling data-driven optimization and professional client reporting! 📈
