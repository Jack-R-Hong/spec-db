# Story 7.2: OpenTelemetry Traces & Metrics

Status: ready-for-dev

## Story

As an operator,
I want OpenTelemetry traces and metrics emitted for all key operations,
so that I can monitor performance, diagnose issues, and track system health.

## Acceptance Criteria (BDD)

**Given** OpenTelemetry export configured in `.spec-db/config.yaml`
**When** a search query executes
**Then** a trace span `spec_db.search.query` is emitted with query text and result count (FR44)
**And** a metric `spec_db.search.latency_ms` is recorded (FR45)

**Given** a graph traversal (trace_impact or find_dependencies)
**When** it executes
**Then** a trace span `spec_db.graph.traverse` is emitted with spec ID, direction, and depth (FR44)

**Given** a sync operation (full or incremental)
**When** it executes
**Then** a trace span `spec_db.sync.{mode}` is emitted with duration and spec count (FR44)
**And** a metric `spec_db.sync.duration_ms` is recorded (FR45)

**Given** any MCP tool call
**When** it executes
**Then** a trace span `spec_db.mcp.tool_call` is emitted with tool name and duration (FR44)

**Given** a consistency check detecting drift
**When** drift is found
**Then** a metric `spec_db.consistency.drift_detected` is incremented (FR46)
**And** a metric `spec_db.consistency.check_result` records pass/fail (FR45)

**Given** OpenTelemetry export is NOT configured
**When** the system runs
**Then** no telemetry is exported — zero external network calls (NFR25)
**And** local console logging via `tracing-subscriber` still works (human-readable format)

**Given** OpenTelemetry export is configured
**When** the system emits traces and metrics
**Then** they use standard OTLP protocol compatible with Jaeger, Grafana, and Datadog (NFR22)

## Tasks / Subtasks

- [ ] Extend config model for opt-in telemetry in `crates/core/src/types.rs` (or config struct module) and parsing in startup code.
  - [ ] Add `TelemetryConfig` with explicit `enabled`/endpoint/protocol fields under `.spec-db/config.yaml`.
  - [ ] Ensure default when telemetry section is absent is local-only (`enabled = false`, no exporter construction).
  - [ ] Add config validation for supported protocols and endpoint presence when enabled.
- [ ] Implement telemetry bootstrap in `src/main.rs` (or dedicated `src/telemetry.rs` module).
  - [ ] Add `fn init_observability(config: &Config) -> Result<ObservabilityGuards, SpecDbError>` to build subscriber stack once at startup.
  - [ ] Always install a `tracing_subscriber::fmt` layer for local logs.
  - [ ] Conditionally add OTel layers only when telemetry is configured, preserving NFR25 zero-export behavior otherwise.
- [ ] Implement trace provider setup using OpenTelemetry 0.31 ecosystem.
  - [ ] Configure `opentelemetry_otlp::SpanExporter::builder()` with `.with_tonic()` or `.with_http()` based on config protocol.
  - [ ] Build `opentelemetry_sdk::trace::SdkTracerProvider::builder().with_batch_exporter(...)` and service resource attributes.
  - [ ] Attach bridge layer via `tracing_opentelemetry::layer().with_tracer(tracer)`.
- [ ] Implement metric provider setup using OpenTelemetry 0.31 ecosystem.
  - [ ] Configure `opentelemetry_otlp::MetricExporter::builder()` with selected OTLP protocol.
  - [ ] Build `opentelemetry_sdk::metrics::SdkMeterProvider::builder().with_periodic_exporter(...)`.
  - [ ] Register metric instruments once and share handles through a central registry (`crates/core/src/telemetry.rs` or equivalent).
- [ ] Define metric instruments and semantic field keys in shared telemetry module (`crates/core/src/telemetry.rs`).
  - [ ] Histogram: `spec_db.search.latency_ms`.
  - [ ] Histogram: `spec_db.sync.duration_ms`.
  - [ ] Counter: `spec_db.consistency.drift_detected`.
  - [ ] Counter/UpDownCounter: `spec_db.consistency.check_result` with result label `pass|fail`.
  - [ ] Gauge/Counter for document-count snapshots (`spec_db.store.doc_count`, labels `store=tantivy|fjall`) after sync/check cycles.
- [ ] Retrofit span instrumentation across all implementation crates (cross-cutting requirement).
  - [ ] Search crate (`crates/search/src/query.rs`, `crates/search/src/indexer.rs`): add spans for query execution and indexing commits; include query text redaction strategy and result count.
  - [ ] Causal crate (`crates/causal/src/traversal.rs`, `crates/causal/src/engine.rs`): add `spec_db.graph.traverse` spans for `trace_impact` and `find_dependencies` including `spec_id`, `direction`, `depth`.
  - [ ] Ingest crate (`crates/ingest/src/sync.rs`, `crates/ingest/src/consistency.rs`, `crates/ingest/src/parser.rs`): add `spec_db.sync.full`, `spec_db.sync.incremental`, and consistency-check spans with timing and record counts.
  - [ ] Router crate (`crates/router/src/lib.rs`, `crates/router/src/composer.rs`): add routing/composition spans and intent tags.
  - [ ] MCP crate (`crates/mcp/src/tools.rs`, `crates/mcp/src/server.rs`): wrap each tool invocation in `spec_db.mcp.tool_call` span with tool name and duration.
  - [ ] Root binary (`src/main.rs`): startup/shutdown spans and telemetry init status events.
- [ ] Enforce N5 span naming conventions in a shared helper (`crates/core/src/telemetry.rs`).
  - [ ] Define constants for required span names: `spec_db.search.query`, `spec_db.graph.traverse`, `spec_db.sync.full`, `spec_db.sync.incremental`, `spec_db.mcp.tool_call`, `spec_db.consistency.check`.
  - [ ] Add compile-time/CI lint-style test asserting no legacy/incorrect span prefixes remain.
  - [ ] Map dynamic sync mode to `spec_db.sync.{mode}` while preserving deterministic values (`full|incremental`).
- [ ] Implement F4 output mode switching in subscriber composition.
  - [ ] When OTel is not configured: human-readable `fmt` layer only.
  - [ ] When OTel is configured: structured JSON local logging + OTel trace/metric layers.
  - [ ] Enforce mutually exclusive local format modes (never both human + JSON simultaneously).
- [ ] Record AC-specific metrics in operation code paths.
  - [ ] Search query path records `spec_db.search.latency_ms` histogram with query type tags.
  - [ ] Sync paths record `spec_db.sync.duration_ms` histogram for full and incremental modes.
  - [ ] Consistency path increments `spec_db.consistency.drift_detected` on drift and records `spec_db.consistency.check_result` on every run.
  - [ ] Publish doc count measurements from both stores after sync/check completion.
- [ ] Add tests validating telemetry behavior and opt-in safety.
  - [ ] Unit tests in telemetry module verify provider creation is skipped when config absent/disabled.
  - [ ] Integration tests in each crate assert spans/metrics are emitted for instrumented paths using test subscribers/exporters.
  - [ ] NFR25 test asserts zero network exporter initialization/calls when telemetry not configured.
  - [ ] Formatting test asserts F4 mode switch: human-readable default vs JSON when OTel configured.
  - [ ] Shutdown test ensures tracer and meter providers flush/shutdown cleanly on process exit.

## Dev Notes

- This story implements FR44-FR46 and is explicitly cross-cutting across all crates; instrumentation is not isolated to one module.
- OTel export is strictly opt-in; no `.spec-db/config.yaml` telemetry config means no exporter, no OTLP client, no external telemetry traffic (NFR25).
- Required crate versions for this implementation stream:
  - `opentelemetry = 0.31.0`,
  - `opentelemetry_sdk = 0.31.0`,
  - `opentelemetry-otlp = 0.31.0`,
  - `tracing-opentelemetry = 0.32.x` (paired with OTel 0.31),
  - `tracing-subscriber = 0.3.x`.
- Current API pattern is provider-first setup: build exporter -> build tracer/meter provider -> install `tracing_opentelemetry::layer()`.
- `opentelemetry::metrics::Meter` instruments should be created once and cloned; avoid duplicate instrument registration for same metric names.
- OTLP protocol options should support both gRPC and HTTP/protobuf so emitted telemetry is backend-neutral (Jaeger, Grafana, Datadog via collector/OTLP endpoints).
- Follow architecture async boundary: instrumentation can wrap sync functions directly; do not force async conversions in search/causal/ingest code.
- Maintain error handling conventions (`thiserror` in libs, `anyhow` at binary) while adding telemetry init/shutdown error contexts.

### Project Structure Notes

- Suggested shared telemetry module path: `crates/core/src/telemetry.rs` with exports from `crates/core/src/lib.rs`.
- Bootstrap wiring lives in `src/main.rs`; this is the single location that should decide whether OTel layers are installed.
- Retroactive instrumentation scope includes all existing crates and public operations:
  - `crates/search/src/*`,
  - `crates/causal/src/*`,
  - `crates/ingest/src/*`,
  - `crates/router/src/*`,
  - `crates/mcp/src/*`,
  - `src/main.rs`.
- Keep crate boundaries intact by sharing telemetry handles/types through `spec-db-core` rather than cross-importing concrete internals.

### References

- Epic 7 Story 7.2 acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Story 7.2: OpenTelemetry Traces & Metrics]
- Observability cross-cutting and opt-in behavior: [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns Identified]
- Observability stack and F4 output mode rule: [Source: _bmad-output/planning-artifacts/architecture.md#Infrastructure & Deployment]
- Span naming convention N5 and log format F4: [Source: _bmad-output/planning-artifacts/architecture.md#Naming Patterns]
- Process pattern P1 (critical init behavior): [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- OTel crate lock and version context: [Source: _bmad-output/planning-artifacts/architecture.md#Version Audit (Feb 2026)]
- OpenTelemetry Rust API (meter instruments): [Source: https://docs.rs/opentelemetry/0.31.0/opentelemetry/metrics/struct.Meter.html]
- OTLP exporter builder patterns and protocols: [Source: https://docs.rs/opentelemetry-otlp/0.31.0/opentelemetry_otlp/]
- tracing-opentelemetry layer integration and special fields: [Source: https://docs.rs/tracing-opentelemetry/0.32.1/tracing_opentelemetry/]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story file authored with explicit opt-in-only telemetry architecture and no-telemetry-default behavior.
- Includes concrete cross-crate retrofit tasks for traces and metrics, including required N5 span names and AC metric names.
- Incorporates OpenTelemetry 0.31.0 API patterns for tracer/meter provider and OTLP exporters.

### Change Log

- 2026-02-23: Initial ready-for-dev draft created for Story 7.2.

### File List

- `_bmad-output/implementation-artifacts/7-2-opentelemetry-traces-metrics.md`
