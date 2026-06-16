// Concurrency / thread-safety tests for the service dispatch layer.
// These exercise the same functions the axum HTTP routes call, under
// concurrent load from many threads, verifying no data races, panics,
// or result corruption. Uses the OS CSPRNG (the default, thread-safe source).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use chance::services;
use chance::services::dto::*;

#[test]
fn concurrent_roll_requests_all_succeed() {
    let n = 64;
    let errors = Arc::new(AtomicUsize::new(0));
    let oks = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(n);
    for _ in 0..n {
        let errors = Arc::clone(&errors);
        let oks = Arc::clone(&oks);
        handles.push(thread::spawn(move || {
            let req = RollRequest {
                source: SourceRequest::default(),
                notation: "2d20+5".to_string(),
            };
            match services::roll(&req) {
                Ok(resp) => {
                    // 2d20+5 => total in [7, 45]
                    let total = resp.result.total;
                    assert!(
                        (7..=45).contains(&total),
                        "roll total out of range: {}",
                        total
                    );
                    oks.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    eprintln!("concurrent roll error: {}", e);
                    errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for h in handles {
        h.join().expect("worker thread panicked under concurrency");
    }
    assert_eq!(errors.load(Ordering::Relaxed), 0, "some roll requests failed");
    assert_eq!(oks.load(Ordering::Relaxed), n);
}

#[test]
fn concurrent_mixed_endpoints_no_panic() {
    let n = 48;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        handles.push(thread::spawn(move || match i % 4 {
            0 => {
                let req = RollRequest::default();
                let resp = services::roll(&req).expect("roll failed");
                assert!((1..=20).contains(&resp.result.total));
            }
            1 => {
                let req = FlipRequest {
                    source: SourceRequest::default(),
                    times: 10,
                };
                let resp = services::flip(&req).expect("flip failed");
                assert_eq!(resp.result.len(), 10);
            }
            2 => {
                let req = BytesRequest {
                    source: SourceRequest::default(),
                    count: 32,
                    encoding: "hex".to_string(),
                };
                let resp = services::bytes(&req).expect("bytes failed");
                assert_eq!(resp.result.len(), 64); // 32 bytes => 64 hex chars
            }
            _ => {
                let req = ShuffleRequest {
                    source: SourceRequest::default(),
                    items: (0..20).map(|x| x.to_string()).collect(),
                };
                let resp = services::shuffle(&req).expect("shuffle failed");
                assert_eq!(resp.result.len(), 20);
            }
        }));
    }
    for h in handles {
        h.join().expect("worker thread panicked under concurrency");
    }
}

#[test]
fn concurrent_seeded_roll_is_deterministic_per_thread() {
    // Many threads each create an independent seeded source; each thread's
    // single roll must be deterministic and equal to the known seeded value.
    let n = 32;
    let mut handles = Vec::with_capacity(n);
    for _ in 0..n {
        handles.push(thread::spawn(|| {
            let req = RollRequest {
                source: SourceRequest {
                    source: Some("chacha20".to_string()),
                    seed: Some("42".to_string()),
                },
                notation: "d100".to_string(),
            };
            let resp = services::roll(&req).expect("seeded roll failed");
            resp.result.total
        }));
    }
    let totals: Vec<i64> = handles
        .into_iter()
        .map(|h| h.join().expect("thread panicked"))
        .collect();
    // All threads used the same seed, so all totals must be identical.
    let first = totals[0];
    assert!(
        totals.iter().all(|&t| t == first),
        "seeded concurrent rolls diverged: {:?}",
        totals
    );
    assert!((1..=100).contains(&first), "d100 total out of range: {}", first);
}
