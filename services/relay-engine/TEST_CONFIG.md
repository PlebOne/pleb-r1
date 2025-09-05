# Test Configuration for NrelayOne Relay Engine

## Test Environment Setup

### Database Configuration
- **Unit Tests**: Use in-memory SQLite databases
- **Integration Tests**: Use test PostgreSQL database or fallback to SQLite
- **E2E Tests**: Use isolated test database instances

### Test Database URLs
```
# For local testing
TEST_DATABASE_URL="postgresql://test:test@localhost:5432/test_relay"

# For CI/CD
TEST_DATABASE_URL="sqlite::memory:"
```

### Environment Variables for Tests
```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
export TEST_DATABASE_URL="sqlite::memory:"
export RELAY_NAME="Test Relay"
export RELAY_DESCRIPTION="Test relay for automated testing"
```

## Test Coverage Goals

### Target Coverage: 90%+

#### Module Coverage Requirements:
- **config.rs**: 95%+ (critical configuration)
- **rate_limiter.rs**: 90%+ (security critical)
- **metrics.rs**: 85%+ (monitoring)
- **database.rs**: 90%+ (data integrity)
- **websocket.rs**: 85%+ (protocol compliance)

#### Test Types Distribution:
- **Unit Tests**: 70% of total test count
- **Integration Tests**: 20% of total test count
- **End-to-End Tests**: 10% of total test count

## Test Execution Order

1. **Unit Tests** (Fast feedback)
   - Configuration validation
   - Rate limiter logic
   - Metrics collection
   - Individual module tests

2. **Integration Tests** (Module interaction)
   - Database operations
   - WebSocket message handling
   - Component integration

3. **End-to-End Tests** (Full system)
   - Complete relay functionality
   - Protocol compliance
   - Performance under load

## Performance Benchmarks

### Target Performance Metrics:
- **Event Processing**: < 1ms per event
- **Subscription Management**: < 0.1ms per operation
- **Message Serialization**: < 0.1ms per message
- **Concurrent Connections**: Support 1000+ concurrent clients

### Benchmark Categories:
1. **Serialization/Deserialization**
2. **Database Operations**
3. **WebSocket Handling**
4. **Rate Limiting**
5. **Metrics Collection**

## Test Data Management

### Test Event Generation:
```rust
// Standard test event
let test_event = EventBuilder::new(Kind::TextNote, "Test message", [])
    .to_event(&test_keys)
    .unwrap();
```

### Test Client Simulation:
- Multiple concurrent connections
- Various message patterns
- Error condition testing
- Rate limit boundary testing

## Continuous Integration

### GitHub Actions Test Matrix:
- **OS**: Ubuntu, Windows, macOS
- **Rust**: stable, beta, nightly
- **Database**: PostgreSQL, SQLite

### Test Automation Requirements:
1. All tests must pass on all platforms
2. Coverage reports generated automatically
3. Performance regression detection
4. Security vulnerability scanning

## Test Utilities

### Mock Objects:
- Database connections
- WebSocket clients
- Rate limiter states
- Metrics collectors

### Test Helpers:
- Event builders
- Filter creators
- Connection simulators
- Time manipulation

## Error Testing

### Error Scenarios to Test:
1. **Invalid JSON Messages**
2. **Malformed Nostr Events**
3. **Database Connection Failures**
4. **Rate Limit Exceeded**
5. **WebSocket Connection Drops**
6. **Memory Pressure**
7. **Network Timeouts**

## Load Testing

### Load Test Scenarios:
1. **Steady State**: 100 events/second for 10 minutes
2. **Burst Load**: 1000 events in 10 seconds
3. **Connection Storm**: 500 simultaneous connections
4. **Memory Stress**: Large event payloads
5. **Subscription Churn**: Rapid subscribe/unsubscribe

### Performance Monitoring:
- Memory usage tracking
- CPU utilization
- Connection count
- Response times
- Error rates

## Security Testing

### Security Test Cases:
1. **Rate Limiting Effectiveness**
2. **Input Validation**
3. **SQL Injection Prevention**
4. **DoS Attack Resilience**
5. **Event Signature Validation**

## Test Maintenance

### Regular Test Updates:
- Update test data monthly
- Review test coverage quarterly
- Benchmark performance regression weekly
- Security test updates as needed

### Test Documentation:
- Document all test scenarios
- Maintain test data schemas
- Update test environment setup guides
- Keep troubleshooting guides current
