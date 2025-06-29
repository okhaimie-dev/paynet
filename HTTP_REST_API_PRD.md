# PRD: HTTP REST API Transport Layer for Cashu NUT Compliance

## Executive Summary

**Project Scope**: Add HTTP REST API transport layer to expose existing Cashu-compliant functionality alongside the current gRPC service.

**Key Discovery**: Comprehensive codebase analysis reveals that **all Cashu NUT business logic, data types, and JSON serialization already exist**. This project requires implementing only a thin HTTP transport layer, not rebuilding functionality.

**Strategic Value**: 
- Enable web-standard API access to mint functionality
- Provide feature-flag flexibility for deployment scenarios
- Maintain zero code duplication through shared application state
- Support both gRPC (internal/efficient) and HTTP REST (web-compatible) protocols

## Current State Analysis

### What Already Exists ✅

**Complete Cashu NUT Implementation Suite**
- **NUT-00 through NUT-07**: All specifications implemented with full request/response types
- **NUT-19**: Response caching infrastructure operational
- **JSON Serialization**: All types have Serde support matching Cashu specifications exactly
- **Business Logic**: Complete implementation in `GrpcState` with all `inner_*()` methods
- **Infrastructure**: PostgreSQL integration, error handling, configuration system, observability

**Development Dependencies Ready**
- **Axum HTTP Framework**: Version 0.8.1 available in workspace dependencies
- **Supporting Libraries**: tower, http, serde_json all configured
- **Feature Flag System**: Already in use for TLS, Starknet, keyset-rotation

**Robust State Management**
```rust
pub struct GrpcState {
    pub pg_pool: PgPool,                    // Database connectivity
    pub signer: SignerClient,               // Cryptographic operations  
    pub keyset_cache: KeysetCache,          // Performance optimization
    pub nuts: NutsSettingsState,            // Configuration management
    pub quote_ttl: Arc<QuoteTTLConfigState>, // Quote lifecycle
    pub liquidity_sources: LiquiditySources, // Payment processing
    pub response_cache: Arc<InMemResponseCache>, // Shared response caching
}
```

### What Needs Implementation ❌

**HTTP Transport Layer Only**
1. Axum route handlers that call existing `inner_*()` methods
2. HTTP server initialization alongside gRPC server  
3. Feature flag for conditional compilation (`http`)
4. State struct rename (`GrpcState` → `AppState`) for shared usage
5. HTTP-compatible cache key generation (same `(Route, u64)` format as gRPC)

## Technical Implementation Plan

### Phase 1: Foundation Setup (0.5 days)
**State Management Refactor**
- Rename `GrpcState` to `AppState` in `grpc_service.rs`
- Move `AppState` to `app_state.rs` for shared access
- Update feature flags in `Cargo.toml`:
  ```toml
  [features]
  default = ["grpc"]
  grpc = []
  http = ["dep:axum"]
  ```

### Phase 2: HTTP Service Implementation (2 days)
**Create HTTP Transport Layer**
- `http_service.rs`: Axum handlers calling existing business logic
- `initialization/http.rs`: HTTP server setup and routing configuration
- Direct method mapping:

| HTTP Endpoint | Existing Method | Notes |
|---------------|----------------|-------|
| `GET /v1/keys` | `keysets()` + `keys()` | Combine for NUT-01 compliance |
| `POST /v1/swap` | `swap()` | Direct mapping |
| `POST /v1/mint/quote/{method}` | `mint_quote()` | Direct mapping |
| `POST /v1/mint/{method}` | `mint()` | Direct mapping |
| `POST /v1/melt/{method}` | `melt()` | Direct mapping |
| `GET /v1/info` | `get_node_info()` | Direct mapping |
| `POST /v1/checkstate` | `check_state()` | Direct mapping |

### Phase 3: Integration & Configuration (1 day)
**Server Launch & Environment**
- Update `main.rs` with conditional compilation:
  ```rust
  #[cfg(feature = "grpc")]
  let grpc_server = launch_grpc_server(app_state.clone());
  
  #[cfg(feature = "http")] 
  let http_server = launch_http_server(app_state.clone());
  ```
- Add `HTTP_PORT` environment variable configuration
- Implement HTTP cache key generation using same `(Route, u64)` format as gRPC
- Ensure HTTP handlers use shared `response_cache` from `AppState`

### Phase 4: Testing & Validation (0.5 days)
**Quality Assurance**
- Verify HTTP responses match gRPC responses for identical operations
- Test feature flag compilation scenarios
- Validate Cashu NUT specification compliance

## Architecture Benefits

### Code Efficiency
- **Zero Business Logic Duplication**: HTTP handlers are thin wrappers over existing methods
- **Shared Infrastructure**: Single database pool, shared response cache, and configuration
- **Unified Caching**: Both gRPC and HTTP use the same `InMemResponseCache` with identical cache keys
- **Consistent Behavior**: Identical functionality and performance between gRPC and HTTP APIs

### Deployment Flexibility
```bash
# Current: gRPC only
cargo build

# Enhanced: Both protocols  
cargo build --features http

# Alternative: HTTP only
cargo build --no-default-features --features http
```

### Operational Advantages
- **Unified State Management**: Single source of truth for both services
- **Cross-Protocol Cache Benefits**: HTTP requests can benefit from gRPC-cached responses and vice versa
- **Cache Efficiency**: Single cache instance eliminates memory duplication and improves hit rates
- **Maintenance Efficiency**: Business logic changes apply to both APIs automatically

## Cashu NUT Compliance Mapping

| NUT Spec | HTTP Endpoint | Method | Existing Implementation |
|----------|---------------|--------|------------------------|
| **NUT-01** | `/v1/keys[/{keyset_id}]` | GET | `keysets()`, `keys()` |
| **NUT-03** | `/v1/swap` | POST | `swap()` |
| **NUT-04** | `/v1/mint/quote/{method}` | POST | `mint_quote()` |
| **NUT-04** | `/v1/mint/quote/{method}/{quote_id}` | GET | `mint_quote_state()` |
| **NUT-04** | `/v1/mint/{method}` | POST | `mint()` |
| **NUT-05** | `/v1/melt/quote/{method}` | POST | *New wrapper needed* |
| **NUT-05** | `/v1/melt/quote/{method}/{quote_id}` | GET | `melt_quote_state()` |
| **NUT-05** | `/v1/melt/{method}` | POST | `melt()` |
| **NUT-06** | `/v1/info` | GET | `get_node_info()` |
| **NUT-07** | `/v1/checkstate` | POST | `check_state()` |

## Risk Assessment & Mitigation

### Technical Risks
- **State Synchronization**: Mitigated by shared `AppState` instance
- **Cache Key Compatibility**: Mitigated by using identical `(Route, u64)` cache key format
- **Performance Impact**: Mitigated by shared caching and connection pooling  
- **Compliance Accuracy**: Mitigated by reusing existing JSON-serialized types

### Implementation Risks
- **Feature Flag Complexity**: Mitigated by following existing patterns in codebase
- **HTTP Error Handling**: Mitigated by adapting existing gRPC error conversion patterns

## Success Metrics

### Functional Validation
- All Cashu NUT endpoints accessible via HTTP with spec-compliant responses
- Feature compilation works correctly in all three modes
- Existing gRPC functionality completely preserved
- Response consistency between gRPC and HTTP for identical operations

### Performance Targets
- HTTP response times within 20% of gRPC equivalent operations
- Memory usage increase limited to Axum framework overhead only
- **Cross-protocol cache hits**: HTTP requests benefit from gRPC-cached responses
- **Improved cache efficiency**: Single shared cache improves overall hit rates

## Project Timeline

**Total Duration**: 4 days

**Resource Requirements**: 1 developer familiar with Rust, Axum, and existing codebase

**Critical Path**: Foundation setup → HTTP service implementation → Integration

**Deliverables**:
- HTTP REST API service with Cashu NUT compliance
- Feature flag system for flexible deployment
- Documentation and testing validation
- Zero impact on existing gRPC functionality

## Strategic Impact

This implementation positions the Paynet node as a dual-protocol service supporting both high-performance gRPC for internal operations and web-standard HTTP REST for broader ecosystem compatibility, achieved through efficient code reuse and shared infrastructure.