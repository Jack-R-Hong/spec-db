//! Acceptance tests for Story 5.1: Query Intent Classification

use std::time::{Duration, Instant};

use spec_db_router::{IntentClassifier, QueryIntent, classify};

fn _assert_intent(query: &str, expected: QueryIntent) {
    assert_eq!(classify(query), expected, "query '{query}' classified incorrectly");
    assert_eq!(IntentClassifier::classify(query), expected, "classifier mismatch for '{query}'");
}

/// AC1: Queries containing causal signal words classify as Causal.
#[test]
fn ac1_causal_signals_classify_as_causal() {
    for signal in ["impact", "depends", "breaks", "affects", "upstream", "downstream"] {
        _assert_intent(signal, QueryIntent::Causal);
        _assert_intent(&format!("how does this {signal} services"), QueryIntent::Hybrid);
    }
}

/// AC2: Queries without causal signals classify as Search.
#[test]
fn ac2_non_causal_queries_classify_as_search() {
    _assert_intent("rate limiting api", QueryIntent::Search);
    _assert_intent("jwt token expiration policy", QueryIntent::Search);
    _assert_intent("login retry thresholds", QueryIntent::Search);
}

/// AC3: Queries with both causal signal and search terms classify as Hybrid.
#[test]
fn ac3_mixed_queries_classify_as_hybrid() {
    _assert_intent("what depends on rate limiting", QueryIntent::Hybrid);
    _assert_intent("which flows breaks auth login", QueryIntent::Hybrid);
    _assert_intent("downstream effects of api gateway", QueryIntent::Hybrid);
}

/// AC4: Classification overhead remains under five milliseconds.
#[test]
fn ac4_classification_overhead_under_five_milliseconds() {
    let iterations: u32 = 5_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = classify("what depends on rate limiting");
    }
    let average = start.elapsed() / iterations;

    assert!(average < Duration::from_millis(5), "average classification latency was {average:?}");
}
