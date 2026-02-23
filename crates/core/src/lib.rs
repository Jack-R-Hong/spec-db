// Downstream: Story 1.2 consumes SpecNode/CausalEdge; Stories 1.3/1.4 consume CausalGraph.

pub mod config;
pub mod error;
pub mod telemetry;
pub mod traits;
pub mod types;

pub use config::{SpecDbConfig, TelemetryConfig, load_config};
pub use error::SpecDbError;
pub use traits::{CausalGraph, SearchEngine, SpecStore};
pub use types::{CausalEdge, EdgeOrigin, SpecDoc, SpecId, SpecNode, TrustLevel};
