---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
inputDocuments:
  - docs/index.md
  - docs/project-context.md
  - docs/architecture-backend.md
  - docs/project-overview.md
  - docs/integration-architecture.md
  - docs/development-guide-backend.md
  - docs/architecture-web-ui.md
  - docs/development-guide-web-ui.md
  - docs/source-tree-analysis.md
  - crates/search/src/lib.rs
  - crates/core/src/traits.rs
documentCounts:
  briefCount: 0
  researchCount: 0
  brainstormingCount: 0
  projectDocsCount: 11
classification:
  projectType: developer_tool
  domain: scientific
  complexity: medium-high
  projectContext: brownfield
workflowType: 'prd'
---

# Product Requirements Document - lattice

**Author:** Jack
**Date:** 2026-02-27

## Executive Summary

**Lattice Multi-Backend Search** 將 lattice 從單一 Tantivy FTS 引擎升級為 **multi-store search infrastructure**，支援 Agent Team 協作場景。不同 AI agents 可透過統一的 MCP 介面，存取各自被授權的知識庫 — 實現 **scoped knowledge access**。

**Target Users:** 使用 lattice 管理 specs 的開發團隊，特別是部署多個 AI agents 協作的場景。

**Problem Solved:** 當前 lattice 僅支援單一 Tantivy FTS backend，所有 agents 看到相同的搜尋結果。這無法滿足：
- Agent 角色分離（不同 agent 該看不同資料）
- Semantic search 需求（vector similarity vs keyword matching）
- 部署彈性（embedded vs client-server）

### What Makes This Special

**Core Insight:** Agent teams 需要 **knowledge isolation** — 這不是搜尋技術選型問題，而是 agent collaboration architecture 問題。

**Differentiator:**
- **Multi-store coexistence** — 同一 lattice 實例同時連接 Tantivy + LanceDB + Qdrant
- **Agent-scoped routing** — Query router 根據 agent context 路由到對應 backend
- **Unified MCP interface** — 對 agents 完全透明，MCP tools 不變
- **Hybrid search** — Vector + FTS 結果合併，取長補短

**Why now:** Multi-agent systems 正在從 single-agent 演進到 agent teams。Knowledge infrastructure 需要跟上。

## Project Classification

| Attribute | Value |
|-----------|-------|
| **Project Type** | Developer Tool (CLI + MCP server for AI agents) |
| **Domain** | Scientific / AI Infrastructure |
| **Complexity** | Medium-High |
| **Project Context** | Brownfield (extending existing Rust codebase) |

**Technical Context:**
- 現有 `SearchEngine` trait 提供良好抽象基礎
- 需新增: `VectorSearchBackend` trait, filter translation layer, score normalization
- Backend options: Tantivy (FTS), LanceDB (embedded vector), Qdrant (server vector)

## Success Criteria

### User Success

| Criteria | Measurement |
|----------|-------------|
| **Multi-backend 設定成功** | 開發者可在 10 分鐘內透過 config 設定 2+ backends |
| **Agent isolation 有效** | 不同 agent context 確實路由到不同 backend，搜尋結果符合預期 |
| **Hybrid search 提升明顯** | Vector + FTS 合併結果比純 FTS 更相關 |
| **開發者體驗良好** | Config-driven，無需改程式碼即可切換/新增 backends |

**「Aha!」時刻：**
- 「原來不同 agent 真的看到不同東西！」
- 「Hybrid search 找到了純關鍵字搜不到的 specs！」

### Business Success

| Criteria | Target |
|----------|--------|
| **開源社群採用** | GitHub stars、issues、PRs 有活躍度 |
| **Documentation 完整** | 新使用者可自行設定，無需問作者 |
| **架構可擴展** | 新增 Qdrant backend 時，核心邏輯不需改動 |

### Technical Success

| Criteria | Measurement |
|----------|-------------|
| **Trait abstraction 乾淨** | `VectorSearchBackend` trait 清晰，新 backend 實作 < 500 LOC |
| **Score normalization 正確** | 跨 backend 分數可比較 (0-1 normalized) |
| **Filter translation 運作** | 統一 filter API，自動轉換為 LanceDB SQL / Qdrant structured |
| **效能可接受** | Search latency < 100ms for 1000 specs |
| **測試覆蓋** | 各 backend 有 integration tests |

### Measurable Outcomes

- [ ] 2+ backends 同時運作，config-based 選擇
- [ ] Agent context → backend routing 正確
- [ ] Hybrid search 返回合併結果
- [ ] 現有 MCP tools 無 breaking changes

## Product Scope

### MVP - Minimum Viable Product

| Component | Description |
|-----------|-------------|
| **VectorSearchBackend trait** | 統一介面：`search()`, `search_hybrid()`, `index()`, `remove()` |
| **LanceDB 實作** | Embedded vector search，支援 FTS + vector hybrid |
| **Tantivy FTS 保持** | 作為 fallback，現有 `SearchEngine` trait 不變 |
| **Agent-scoped routing** | Query router 根據 agent context 選擇 backend |
| **Hybrid search** | Vector + FTS 結果合併，score normalization |
| **Config schema** | `.lattice/config.yaml` 新增 `search_backends` section |
| **Embedding 支援** | Local (sentence-transformers) + Remote (OpenAI) 可選 |

### Growth Features (Post-MVP)

| Feature | Description |
|---------|-------------|
| **Qdrant 實作** | Client-server vector DB，適合團隊部署 |
| **Multi-store queries** | 同時查詢多個 backends，合併結果 |
| **Backend migration tools** | 資料在不同 backends 間遷移 |
| **Observability** | Per-backend metrics, latency tracking |

### Vision (Future)

| Feature | Description |
|---------|-------------|
| **Auto-backend selection** | 根據 query 特性自動選最佳 backend |
| **Federated search** | 跨多個 lattice instances 搜尋 |
| **Custom embedding models** | 支援 fine-tuned domain-specific embeddings |

## User Journeys

### Journey 1: Developer — 首次設定 Multi-Backend (via Config + MCP)

**Persona: Alex，後端開發者**

**Opening Scene:**
Alex 的團隊用 lattice 管理 200+ specs。最近部署了 3 個 AI agents 協作：Code Agent（寫程式）、Review Agent（審查）、Doc Agent（文件）。問題是，所有 agents 搜尋都回傳相同結果 — Review Agent 不該看到 draft specs，Doc Agent 需要 semantic search 找相關文件。

**Rising Action:**
1. Alex 更新 lattice 到新版本
2. 編輯 `.lattice/config.yaml`，新增 `search_backends` section
3. 透過 **MCP tool** `configure_backend` 快速驗證設定
4. 設定 agent routing rules：`review-agent → tantivy`, `doc-agent → private-kb`

**Climax:**
Alex 測試搜尋 — Doc Agent 用 semantic search 找到了「概念相關但關鍵字不同」的 specs。Review Agent 只看到 approved specs。

**Resolution:**
「終於！不同 agent 真的看到不同東西了。設定只花了 15 分鐘。」

---

### Journey 2: Developer — 透過 API 動態切換 Backend

**Persona: Sarah，平台工程師**

**Opening Scene:**
Sarah 正在建構一個 multi-tenant AI platform，每個 tenant 需要獨立的 knowledge store。她需要在 runtime 動態建立和切換 backends。

**Rising Action:**
1. Sarah 使用 **REST API** `/api/backends` 動態建立新 backend
2. 透過 API 設定 routing rules，不需重啟 lattice
3. 使用 `/api/search` 帶 `X-Backend: tenant-acme` header 指定 backend

**Climax:**
新 tenant onboard 時，自動化腳本在 30 秒內建立專屬 knowledge store，無需人工介入。

**Resolution:**
「Multi-tenant 終於可行了。每個客戶的 agents 只看到自己的資料。」

---

### Journey 3: AI Agent — Runtime 搜尋與 Hybrid Results

**Persona: Doc Agent (AI)**

**Opening Scene:**
Doc Agent 收到任務：「找出所有與 authentication 相關的 specs」。

**Rising Action:**
1. Agent 呼叫 MCP tool `search_specs(query="authentication", mode="hybrid")`
2. Query router 檢查 agent context，路由到 `private-kb` (LanceDB)
3. LanceDB 執行 hybrid search：
   - Vector search: 找到語意相關的 specs（如 "JWT validation", "session management"）
   - FTS: 找到關鍵字匹配的 specs
4. 結果合併，score normalized

**Climax:**
Agent 收到 10 個結果，包含 3 個「純 FTS 找不到」的語意相關 specs。

**Resolution:**
Agent 完成任務，輸出更完整的相關 specs 列表。

---

### Journey 4: Ops — Backend 健康監控與 Troubleshooting

**Persona: Mike，DevOps 工程師**

**Opening Scene:**
Mike 收到告警：某個 agent 的搜尋 latency 飆升。

**Rising Action:**
1. 透過 `lattice status` CLI 查看各 backend 狀態
2. 發現 `private-kb` (LanceDB) 的 index 損壞
3. 使用 `lattice rebuild --backend=private-kb` 重建 index
4. 驗證搜尋恢復正常

**Climax:**
問題在 5 分鐘內解決，無需重啟整個 lattice 服務。

**Resolution:**
「Per-backend rebuild 太有用了，不用影響其他 agents。」

---

### Journey 5: OSS Contributor — 新增 Qdrant Backend 支援

**Persona: Chen，開源貢獻者**

**Opening Scene:**
Chen 的團隊用 Qdrant server，想貢獻 Qdrant backend 到 lattice。

**Rising Action:**
1. 閱讀 `CONTRIBUTING.md` 和 `VectorSearchBackend` trait 定義
2. 建立 `crates/search-qdrant/` 實作 trait
3. 處理 filter translation（Qdrant 用 structured filter）
4. 撰寫 integration tests
5. 提交 PR

**Climax:**
PR review 順利，trait abstraction 夠清晰，實作只需 ~400 LOC。

**Resolution:**
「架構設計得很好，新增 backend 真的很簡單。」

---

### Journey Requirements Summary

| Journey | Key Capabilities Revealed |
|---------|--------------------------|
| **Developer Setup** | Config schema, MCP configure tools, routing rules |
| **API Dynamic Config** | REST API for backend CRUD, runtime routing |
| **Agent Search** | Hybrid search, score normalization, agent context routing |
| **Ops Troubleshooting** | Per-backend status, rebuild, health monitoring |
| **OSS Contribution** | Clean trait abstraction, documentation, test patterns |

## Domain-Specific Requirements

### Data Consistency

| Concern | MVP Approach |
|---------|--------------|
| **Multi-backend sync** | 各 backend 獨立管理，不保證跨 backend 一致性 |
| **Embedding versioning** | MVP 不處理，reindex 時全量重建 |
| **Index corruption** | Per-backend rebuild command，不影響其他 backends |

### Search Quality

| Concern | MVP Approach |
|---------|--------------|
| **Hybrid result quality** | 依賴 score normalization，無 recall/precision metrics |
| **Vector threshold** | Configurable，預設值 based on LanceDB defaults |
| **Quality validation** | 依賴人工測試，MVP 無自動化 quality gates |

### Embedding Considerations

| Concern | MVP Approach |
|---------|--------------|
| **Local vs Remote latency** | Document trade-offs，用戶自行選擇 |
| **Model migration** | Full reindex required，無 incremental migration |
| **Dimension mismatch** | Runtime validation，拒絕不匹配的 vectors |

### Technical Constraints

| Constraint | Requirement |
|------------|-------------|
| **Rust-native** | 所有 backends 必須有 Rust bindings |
| **Async boundary** | Search operations 走 `spawn_blocking`，符合現有架構 |
| **No external servers (MVP)** | LanceDB embedded only，Qdrant 是 post-MVP |
| **Backward compatible** | 現有 `SearchEngine` trait 不改，新增 `VectorSearchBackend` |

### Deferred to Post-MVP

- Search quality metrics (recall, precision, MRR)
- Embedding versioning and migration tools
- Cross-backend consistency guarantees
- A/B testing infrastructure for search quality

## Developer Tool Specific Requirements

### Project-Type Overview

**Type:** Developer Tool (CLI + MCP Server + REST API + Library)

Lattice Multi-Backend Search 擴展現有 lattice 工具，新增 multi-backend search 能力。所有介面層都需要支援新功能。

### API Surface

#### MCP Tools (新增)

| Tool | Description | Parameters |
|------|-------------|------------|
| `configure_backend` | 動態設定 backend | `name`, `type`, `config` |
| `list_backends` | 列出所有已設定的 backends | - |
| `search_specs` | (擴展) 新增 `backend`, `mode` 參數 | `query`, `backend?`, `mode?` |

#### REST API Endpoints (新增)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/backends` | List all backends |
| `POST` | `/api/backends` | Create new backend |
| `PUT` | `/api/backends/:name` | Update backend config |
| `DELETE` | `/api/backends/:name` | Remove backend |
| `GET` | `/api/backends/:name/status` | Backend health status |

#### CLI Commands (新增)

```bash
lattice backend list              # List configured backends
lattice backend add <name> <type> # Add new backend
lattice backend remove <name>     # Remove backend
lattice backend status [name]     # Show backend health
lattice rebuild --backend=<name>  # Rebuild specific backend
```

#### Rust Library API (公開 traits)

```rust
// 新增 trait - 公開給 embedding 使用
pub trait VectorSearchBackend: Send + Sync {
    fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>>;
    fn search_hybrid(&self, query: HybridQuery) -> Result<Vec<SearchHit>>;
    fn index(&mut self, doc: &SpecDoc, embedding: &[f32]) -> Result<()>;
    fn remove(&mut self, id: &SpecId) -> Result<()>;
}

// 保持現有 trait 不變
pub trait SearchEngine { /* 現有介面 */ }
```

### Configuration Schema

```yaml
# .lattice/config.yaml
search_backends:
  default: tantivy  # 預設 backend
  
  routing:  # Agent-scoped routing rules
    - agent: "doc-agent"
      backend: "lancedb-private"
    - agent: "review-agent"
      backend: "tantivy"
  
  backends:
    - name: tantivy
      type: fts
      # 使用現有 Tantivy 設定
    
    - name: lancedb-private
      type: lancedb
      path: ./data/lancedb
      embedding:
        provider: local        # local | openai | cohere
        model: all-MiniLM-L6-v2
        dimensions: 384
    
    - name: qdrant-team  # Post-MVP
      type: qdrant
      url: http://localhost:6333
      collection: specs
      embedding:
        provider: openai
        model: text-embedding-3-small
```

### Installation & Setup

```bash
# 升級現有 lattice
cargo install lattice --features vector-search

# 或從源碼
cargo build --release --features vector-search

# 初始化 (現有用戶)
lattice migrate  # 檢測並升級 config schema
```

### Migration Guide (現有用戶)

| Step | Action |
|------|--------|
| 1 | 備份 `.lattice/` 目錄 |
| 2 | 升級 lattice binary |
| 3 | 執行 `lattice migrate` — 自動更新 config schema |
| 4 | (選用) 新增 vector backend 設定 |
| 5 | (選用) 執行 `lattice rebuild --backend=<name>` 建立 vector index |

**Backward Compatibility:**
- 未設定 `search_backends` 時，行為與舊版完全相同
- 現有 MCP tools 參數保持 optional
- 純 FTS 用戶無需任何變更

### Documentation Requirements

| Document | Priority |
|----------|----------|
| **Migration Guide** | P0 — 現有用戶升級 |
| **Backend Configuration Reference** | P0 — 設定各 backend |
| **API Reference (MCP/REST/CLI)** | P0 — 介面文件 |
| **Architecture Overview** | P1 — 貢獻者理解系統 |
| **Embedding Provider Setup** | P1 — 設定 OpenAI/Local |

### Code Examples

```rust
// Example: Custom backend implementation
use lattice::VectorSearchBackend;

pub struct MyCustomBackend { /* ... */ }

impl VectorSearchBackend for MyCustomBackend {
    fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        // Custom implementation
    }
    // ...
}
```

```yaml
# Example: Multi-agent config
search_backends:
  routing:
    - agent: "code-agent"
      backend: "tantivy"      # FTS for code search
    - agent: "doc-agent"
      backend: "lancedb"      # Semantic for docs
    - agent: "*"
      backend: "tantivy"      # Default fallback
```

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-solving MVP — 最小可行的 agent knowledge isolation 解決方案

**Core Value Proposition:** 讓不同 AI agents 存取各自被授權的知識庫，透過統一介面。

**Resource Requirements:**
- 1 Rust developer (熟悉 lattice codebase)
- 預估 MVP 工期：4-6 週

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**
- ✅ Journey 1: Developer 首次設定 Multi-Backend
- ✅ Journey 3: AI Agent Runtime 搜尋
- ✅ Journey 4: Ops 基本 Troubleshooting
- ⚠️ Journey 2: API 動態設定 (部分支援)
- ❌ Journey 5: OSS Contributor (Post-MVP documentation)

**Must-Have Capabilities:**

| Category | MVP Features |
|----------|--------------|
| **Backend Abstraction** | `VectorSearchBackend` trait |
| **LanceDB Implementation** | Embedded vector search with hybrid |
| **Tantivy FTS** | 保持現有，作為 default/fallback |
| **Agent Routing** | Config-based agent → backend mapping |
| **Hybrid Search** | Vector + FTS 結果合併，score normalization |
| **Embedding** | Local (sentence-transformers) + OpenAI |
| **Config** | `search_backends` section in config.yaml |
| **MCP Tools** | `configure_backend`, `list_backends`, 擴展 `search_specs` |
| **REST API** | `/api/backends` CRUD + status |
| **CLI** | `lattice backend list/add/remove/status` |
| **Migration** | `lattice migrate` for existing users |

**Explicitly Out of MVP:**
- Qdrant backend (需要外部 server)
- Multi-store simultaneous queries
- Per-backend observability/metrics
- Search quality metrics (recall/precision)
- Embedding versioning
- Advanced filter operators

### Post-MVP Features

**Phase 2 (Growth):**

| Feature | Priority | Dependency |
|---------|----------|------------|
| Qdrant backend | P1 | MVP complete |
| Multi-store queries | P2 | Qdrant |
| Backend migration tools | P2 | Multi-backend stable |
| Per-backend metrics | P2 | Observability infra |
| Advanced filters | P3 | User feedback |

**Phase 3 (Vision):**

| Feature | Description |
|---------|-------------|
| Auto-backend selection | Query analysis → optimal backend |
| Federated search | Cross-instance search |
| Custom embedding models | Fine-tuned domain embeddings |
| A/B testing | Search quality experiments |

### Risk Mitigation Strategy

**Technical Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Filter translation 複雜度 | Medium | Medium | MVP 只支援 simple filters (eq, in) |
| Score normalization 不準確 | Low | Medium | 使用 min-max normalization，可調參數 |
| LanceDB Rust bindings 問題 | Low | High | LanceDB 有官方 Rust support |
| Embedding latency | Medium | Low | 支援 batch indexing，async embedding |

**Market Risks:**

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| 用戶不需要 agent isolation | Low | 開發者自己是目標用戶 |
| Hybrid search 無明顯提升 | Medium | 先驗證 POC，再 commit full implementation |

**Resource Risks:**

| Risk | Mitigation |
|------|------------|
| 工作量超預期 | 優先順序：MCP > CLI > REST API |
| 介面層太多 | 可先 ship MCP-only MVP |
| 測試覆蓋不足 | 先 integration tests，後 unit tests |

### MVP Exit Criteria

MVP 完成的定義：

- [ ] 2 backends (Tantivy + LanceDB) 同時運作
- [ ] Agent routing 正確路由到指定 backend
- [ ] Hybrid search 返回合併結果
- [ ] 所有介面 (MCP/REST/CLI) 可操作 backends
- [ ] Migration tool 可升級現有用戶
- [ ] Integration tests 通過
- [ ] 基本文件完成 (Migration Guide, Config Reference)

## Functional Requirements

### Backend Management

- **FR1:** Operators can add a new search backend via configuration file
- **FR2:** Operators can add a new search backend via REST API at runtime
- **FR3:** Operators can add a new search backend via MCP tool at runtime
- **FR4:** Operators can remove an existing search backend
- **FR5:** Operators can list all configured search backends
- **FR6:** Operators can view health status of each search backend
- **FR7:** Operators can rebuild the index of a specific search backend

### Agent Routing

- **FR8:** Operators can define agent-to-backend routing rules in configuration
- **FR9:** System can route search queries to the appropriate backend based on agent context
- **FR10:** System can fall back to default backend when no routing rule matches
- **FR11:** Operators can use wildcard patterns in routing rules

### Search Capabilities

- **FR12:** Agents can search specs using full-text search (FTS) mode
- **FR13:** Agents can search specs using vector similarity mode
- **FR14:** Agents can search specs using hybrid mode (FTS + vector combined)
- **FR15:** System can normalize scores across different backends for comparable results
- **FR16:** Agents can filter search results by tags
- **FR17:** Agents can specify which backend to use for a search query

### Indexing & Data Management

- **FR18:** System can index spec documents into the configured search backend
- **FR19:** System can generate embeddings using local embedding models
- **FR20:** System can generate embeddings using remote API (OpenAI)
- **FR21:** System can remove spec documents from search backends
- **FR22:** System can sync spec documents from git to search backends

### Configuration

- **FR23:** Operators can configure embedding provider (local or remote) per backend
- **FR24:** Operators can configure embedding model and dimensions per backend
- **FR25:** Operators can set a default search backend
- **FR26:** Operators can configure backend-specific storage paths

### Migration & Compatibility

- **FR27:** Existing users can migrate configuration schema using CLI command
- **FR28:** System can operate with legacy configuration (Tantivy-only, no search_backends section)
- **FR29:** Existing MCP tools can function without breaking changes when new parameters are omitted

### Interfaces

- **FR30:** Developers can interact with backends via MCP tools (`configure_backend`, `list_backends`, extended `search_specs`)
- **FR31:** Developers can interact with backends via REST API (`/api/backends` endpoints)
- **FR32:** Developers can interact with backends via CLI (`lattice backend` commands)
- **FR33:** Developers can implement custom backends using the public `VectorSearchBackend` trait

## Non-Functional Requirements

### Performance

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| **NFR1:** Search latency (FTS) | < 50ms | p95 for 1000 specs |
| **NFR2:** Search latency (Vector) | < 100ms | p95 for 1000 specs |
| **NFR3:** Search latency (Hybrid) | < 150ms | p95 for 1000 specs |
| **NFR4:** Indexing throughput | > 10 docs/sec | Batch indexing |
| **NFR5:** Embedding generation (local) | < 200ms/doc | Single document |
| **NFR6:** Backend startup time | < 5 seconds | Per backend initialization |

### Security

| Requirement | Description |
|-------------|-------------|
| **NFR7:** API keys stored securely | Embedding provider API keys not logged or exposed in errors |
| **NFR8:** No credentials in config examples | Documentation examples use placeholders |
| **NFR9:** Input validation | All user inputs validated before processing |

### Reliability

| Requirement | Description |
|-------------|-------------|
| **NFR10:** Backend failure isolation | One backend failure does not crash other backends |
| **NFR11:** Graceful degradation | If vector backend unavailable, fall back to FTS |
| **NFR12:** Index corruption recovery | `lattice rebuild` can recover from corrupted index |
| **NFR13:** Configuration validation | Invalid config rejected with clear error message |

### Integration

| Requirement | Description |
|-------------|-------------|
| **NFR14:** Embedding provider abstraction | Support for multiple embedding providers via config |
| **NFR15:** Backend trait stability | `VectorSearchBackend` trait API stable for external implementations |
| **NFR16:** MCP protocol compliance | All MCP tools conform to MCP specification |

### Compatibility

| Requirement | Description |
|-------------|-------------|
| **NFR17:** Rust version | Supports Rust 1.85+ (matches existing codebase) |
| **NFR18:** Platform support | Linux, macOS, Windows (same as existing lattice) |
| **NFR19:** Backward compatibility | Existing lattice users can upgrade without data loss |
