// Performance benchmarks for the Nostr relay
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use relay_engine::{AppState, Config};
use relay_engine::database::PostgresDatabase;
use relay_engine::metrics::Metrics;
use relay_engine::rate_limiter::{RateLimiter, RateLimitConfig};

use nostr::{ClientMessage, EventBuilder, Filter, Keys, Kind, RelayMessage, SubscriptionId};
use serde_json;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{runtime::Runtime, sync::RwLock};

fn create_test_app_state() -> AppState {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = Config {
            database_url: "sqlite::memory:".to_string(),
            port: 0,
            relay_name: "Benchmark Relay".to_string(),
            relay_description: "Relay for performance benchmarks".to_string(),
            relay_pubkey: None,
            relay_contact: None,
        };

        let metrics = Metrics::new().expect("Failed to create metrics");
        let rate_limiter = RateLimiter::new(RateLimitConfig::default());
        
        AppState {
            config,
            database: PostgresDatabase::new("sqlite::memory:").await.unwrap(),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter,
            metrics,
        }
    })
}

fn bench_event_serialization(c: &mut Criterion) {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "Benchmark message", [])
        .to_event(&keys)
        .unwrap();
    
    let client_msg = ClientMessage::Event(Box::new(event));
    
    c.bench_function("event_serialization", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&client_msg)).unwrap();
            black_box(json);
        })
    });
}

fn bench_event_deserialization(c: &mut Criterion) {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "Benchmark message", [])
        .to_event(&keys)
        .unwrap();
    
    let client_msg = ClientMessage::Event(Box::new(event));
    let json = serde_json::to_string(&client_msg).unwrap();
    
    c.bench_function("event_deserialization", |b| {
        b.iter(|| {
            let msg: ClientMessage = serde_json::from_str(black_box(&json)).unwrap();
            black_box(msg);
        })
    });
}

fn bench_filter_creation(c: &mut Criterion) {
    let keys = Keys::generate();
    let pubkey = keys.public_key();
    
    c.bench_function("filter_creation", |b| {
        b.iter(|| {
            let filter = Filter::new()
                .kinds([Kind::TextNote, Kind::Metadata])
                .authors([pubkey])
                .limit(100);
            black_box(filter);
        })
    });
}

fn bench_subscription_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("subscription_add_remove", |b| {
        b.iter(|| {
            rt.block_on(async {
                let subscriptions: Arc<RwLock<HashMap<String, HashMap<String, Filter>>>> = 
                    Arc::new(RwLock::new(HashMap::new()));
                
                let client_id = "test_client".to_string();
                let sub_id = "test_sub".to_string();
                let filter = Filter::new().kinds([Kind::TextNote]);
                
                // Add subscription
                {
                    let mut subs = subscriptions.write().await;
                    let client_subs = subs.entry(client_id.clone()).or_insert_with(HashMap::new);
                    client_subs.insert(sub_id.clone(), filter);
                }
                
                // Remove subscription
                {
                    let mut subs = subscriptions.write().await;
                    if let Some(client_subs) = subs.get_mut(&client_id) {
                        client_subs.remove(&sub_id);
                    }
                }
                
                black_box(subscriptions);
            })
        })
    });
}

fn bench_rate_limiter(c: &mut Criterion) {
    let rate_limiter = RateLimiter::new(RateLimitConfig {
        events_per_minute: 1000,
        queries_per_minute: 1000,
        connections_per_minute: 1000,
        max_subscriptions_per_client: 100,
        cleanup_interval_seconds: 60,
    });
    
    c.bench_function("rate_limiter_check", |b| {
        b.iter(|| {
            let allowed = rate_limiter.check_event_rate("127.0.0.1");
            black_box(allowed);
        })
    });
}

fn bench_metrics_update(c: &mut Criterion) {
    let metrics = Metrics::new().expect("Failed to create metrics");
    
    c.bench_function("metrics_increment", |b| {
        b.iter(|| {
            metrics.increment_events_received();
            metrics.increment_events_stored();
            metrics.record_event_processing_time(black_box(Duration::from_millis(5)));
        })
    });
}

fn bench_concurrent_subscriptions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    for num_clients in [1, 10, 100].iter() {
        c.bench_with_input(
            BenchmarkId::new("concurrent_subscriptions", num_clients),
            num_clients,
            |b, &num_clients| {
                b.iter(|| {
                    rt.block_on(async {
                        let subscriptions: Arc<RwLock<HashMap<String, HashMap<String, Filter>>>> = 
                            Arc::new(RwLock::new(HashMap::new()));
                        
                        let mut handles = Vec::new();
                        
                        for i in 0..num_clients {
                            let subs = Arc::clone(&subscriptions);
                            let handle = tokio::spawn(async move {
                                let client_id = format!("client_{}", i);
                                let sub_id = format!("sub_{}", i);
                                let filter = Filter::new().kinds([Kind::TextNote]);
                                
                                {
                                    let mut subs = subs.write().await;
                                    let client_subs = subs.entry(client_id).or_insert_with(HashMap::new);
                                    client_subs.insert(sub_id, filter);
                                }
                            });
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.await.unwrap();
                        }
                        
                        black_box(subscriptions);
                    })
                })
            },
        );
    }
}

fn bench_event_validation(c: &mut Criterion) {
    let keys = Keys::generate();
    
    c.bench_function("event_validation", |b| {
        b.iter(|| {
            let event = EventBuilder::new(Kind::TextNote, "Test message for validation", [])
                .to_event(black_box(&keys))
                .unwrap();
            
            // Validate event signature
            let is_valid = event.verify().is_ok();
            black_box(is_valid);
        })
    });
}

fn bench_large_event_handling(c: &mut Criterion) {
    let keys = Keys::generate();
    
    for size in [1_000, 10_000, 100_000].iter() {
        let content = "x".repeat(*size);
        
        c.bench_with_input(
            BenchmarkId::new("large_event_serialization", size),
            &content,
            |b, content| {
                b.iter(|| {
                    let event = EventBuilder::new(Kind::TextNote, content, [])
                        .to_event(&keys)
                        .unwrap();
                    
                    let client_msg = ClientMessage::Event(Box::new(event));
                    let json = serde_json::to_string(black_box(&client_msg)).unwrap();
                    black_box(json);
                })
            },
        );
    }
}

criterion_group!(
    benches,
    bench_event_serialization,
    bench_event_deserialization,
    bench_filter_creation,
    bench_subscription_management,
    bench_rate_limiter,
    bench_metrics_update,
    bench_concurrent_subscriptions,
    bench_event_validation,
    bench_large_event_handling
);

criterion_main!(benches);
