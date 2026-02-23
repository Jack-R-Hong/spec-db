use std::sync::OnceLock;

use spec_db_core::config::TelemetryConfig;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static OBSERVABILITY_INIT: OnceLock<()> = OnceLock::new();

pub fn init_observability(config: &TelemetryConfig) -> anyhow::Result<()> {
    if OBSERVABILITY_INIT.get().is_some() {
        return Ok(());
    }

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let init_result = if config.enabled {
        tracing_subscriber::registry().with(env_filter).with(fmt::layer().json()).try_init()
    } else {
        tracing_subscriber::registry().with(env_filter).with(fmt::layer()).try_init()
    };

    match init_result {
        Ok(()) => {
            let _ = OBSERVABILITY_INIT.set(());
            Ok(())
        }
        Err(err)
            if err.to_string().contains("global default trace dispatcher has already been set") =>
        {
            let _ = OBSERVABILITY_INIT.set(());
            Ok(())
        }
        Err(err) => Err(anyhow::anyhow!("failed to initialize observability: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_observability_default() {
        let config = TelemetryConfig::default();
        let result = init_observability(&config);
        assert!(result.is_ok());
    }
}
