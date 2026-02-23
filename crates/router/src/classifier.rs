use tracing::info_span;

const CAUSAL_SIGNALS: &[&str] =
    &["impact", "depends", "breaks", "affects", "upstream", "downstream"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryIntent {
    Search,
    Causal,
    Hybrid,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct IntentClassifier;

impl IntentClassifier {
    pub fn classify(query: &str) -> QueryIntent {
        classify(query)
    }
}

pub fn classify(query: &str) -> QueryIntent {
    let _span = info_span!("spec_db.router.classify", query_len = query.len()).entered();

    let lower = query.to_lowercase();
    let has_explicit_spec = lower.contains("spec::");
    let tokens: Vec<&str> = lower.split_whitespace().collect();

    let has_causal = tokens.iter().any(|token| {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
        CAUSAL_SIGNALS.contains(&clean)
    });

    let non_causal_tokens = tokens
        .iter()
        .filter(|token| {
            let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
            !clean.is_empty() && !CAUSAL_SIGNALS.contains(&clean)
        })
        .count();

    if has_causal && has_explicit_spec {
        QueryIntent::Causal
    } else if has_causal && non_causal_tokens > 0 {
        QueryIntent::Hybrid
    } else if has_causal {
        QueryIntent::Causal
    } else {
        QueryIntent::Search
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn each_causal_signal_classifies_as_causal() {
        for signal in CAUSAL_SIGNALS {
            assert_eq!(classify(signal), QueryIntent::Causal, "signal '{signal}' failed");
        }
    }

    #[test]
    fn plain_query_classifies_as_search() {
        assert_eq!(classify("rate limiting api"), QueryIntent::Search);
    }

    #[test]
    fn mixed_query_classifies_as_hybrid() {
        assert_eq!(classify("what depends on rate limiting"), QueryIntent::Hybrid);
    }

    #[test]
    fn explicit_spec_with_causal_signal_classifies_as_causal() {
        assert_eq!(classify("what depends on spec::auth::login"), QueryIntent::Causal);
    }

    #[test]
    fn classification_average_is_below_five_ms() {
        let iterations: u32 = 1_000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = classify("what depends on rate limiting");
        }
        let elapsed = start.elapsed();
        let average = elapsed / iterations;

        assert!(average < Duration::from_millis(5), "average classification time was {average:?}");
    }
}
