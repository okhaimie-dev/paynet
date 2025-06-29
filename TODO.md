# HTTP REST API Implementation - Completion TODO

## 📊 Current Implementation Status

### ✅ **COMPLETED (Foundation)**
- **AppState Architecture**: Successfully renamed `GrpcState` → `AppState` with shared state between gRPC and HTTP
- **Feature Flag System**: `grpc` and `http` features with conditional compilation working
- **HTTP Server Infrastructure**: Axum server with CORS, tracing, and proper middleware
- **Shared Caching**: Cross-protocol cache using identical `(Route, u64)` keys
- **NUT-01 Keys**: `GET /v1/keys` and `GET /v1/keys/{keyset_id}` implemented
- **NUT-03 Swap**: `POST /v1/swap` with full request/response caching

### 🚧 **REMAINING WORK** 

## 📋 Phase 1: NUT-04 Mint Operations (Priority: HIGH)
**Estimated Time**: 1-2 days

### Endpoints to Implement:
1. **`POST /v1/mint/quote/{method}`** → calls `mint_quote()`
2. **`GET /v1/mint/quote/{method}/{quote_id}`** → calls `mint_quote_state()`  
3. **`POST /v1/mint/{method}`** → calls `mint()`

### Implementation Tasks:
- [ ] **Add mint quote request handler**
  - Extract `{method}` path parameter and convert to `Method` enum
  - Parse request body to `MintQuoteRequest` 
  - Call `app_state.inner_mint_quote(method, amount, unit)`
  - Convert response to HTTP JSON format

- [ ] **Add mint quote state handler**
  - Extract `{method}` and `{quote_id}` path parameters
  - Convert `quote_id` string to `Uuid`
  - Call `app_state.inner_mint_quote_state(method, quote_id)`
  - Handle `MintQuoteState` enum serialization (UNPAID/PAID/ISSUED)

- [ ] **Add mint execution handler**
  - Extract `{method}` path parameter
  - Parse `MintRequest` with quote and outputs
  - Convert `BlindedMessageHttp` to internal `BlindedMessage` types
  - Call `app_state.inner_mint(method, quote_id, &outputs)`
  - Implement response caching with `(Route::Mint, hash_mint_request_http())` key

### Critical Considerations:
- **UUID Conversion**: Ensure proper error handling for invalid quote ID formats
- **Cache Key Generation**: Must match gRPC implementation for cross-protocol cache hits
- **Error Status Codes**: 400 for invalid requests, 404 for missing quotes, 500 for internal errors
- **Request Validation**: Enforce same limits as gRPC (max 64 outputs, non-empty arrays)

---

## 📋 Phase 2: NUT-05 Melt Operations (Priority: HIGH)
**Estimated Time**: 1-2 days

### Endpoints to Implement:
1. **`POST /v1/melt/quote/{method}`** → **⚠️ MISSING: Need to create melt quote logic**
2. **`GET /v1/melt/quote/{method}/{quote_id}`** → calls `melt_quote_state()`
3. **`POST /v1/melt/{method}`** → calls `melt()`

### Implementation Tasks:
- [ ] **🔍 INVESTIGATE: Melt quote creation**
  - **CRITICAL**: Check if `inner_melt_quote()` exists in gRPC service
  - If missing, need to implement melt quote creation logic
  - Pattern: `(method, request_string, unit) → MeltQuoteResponse`

- [ ] **Add melt quote request handler**
  - Extract `{method}` path parameter
  - Parse request body with payment request and unit
  - Create or call melt quote logic
  - Return quote ID, amount, fee, state, expiry

- [ ] **Add melt quote state handler**
  - Extract `{method}` and `{quote_id}` path parameters
  - Call `app_state.inner_melt_quote_state(method, quote_id)`
  - Handle `MeltQuoteState` enum serialization (UNPAID/PENDING/PAID)

- [ ] **Add melt execution handler**
  - Parse `MeltRequest` with quote and input proofs
  - Convert `ProofHttp` to internal `Proof` types
  - Call `app_state.inner_melt(method, unit, request, &inputs)`
  - Handle `transfer_ids` array in response
  - Implement caching with `(Route::Melt, hash_melt_request_http())` key

### Critical Considerations:
- **⚠️ Missing Logic**: Melt quote creation might not exist - requires investigation
- **Payment Processing**: Melt operations may have longer response times
- **State Management**: Handle PENDING state for async payment processing
- **Error Handling**: Payment failures need specific error codes and messages

---

## 📋 Phase 3: NUT-06 Node Information (Priority: MEDIUM)
**Estimated Time**: 0.5 days

### Endpoint to Implement:
- **`GET /v1/info`** → calls `get_node_info()`

### Implementation Tasks:
- [ ] **Add node info handler**
  - Call `app_state.get_node_info()`
  - Parse JSON string response from gRPC into structured JSON
  - Return proper `NodeInfoResponse` format
  - No caching needed (info is relatively static)

### Critical Considerations:
- **JSON Parsing**: gRPC returns JSON string, HTTP should return structured JSON object
- **Configuration Exposure**: Ensure sensitive configuration is not exposed
- **Version Information**: Include accurate version and node details

---

## 📋 Phase 4: NUT-07 Proof State Checking (Priority: MEDIUM)  
**Estimated Time**: 0.5 days

### Endpoint to Implement:
- **`POST /v1/checkstate`** → calls `check_state()`

### Implementation Tasks:
- [ ] **Add check state handler**
  - Parse `CheckStateRequest` with array of hex-encoded Y points
  - Convert hex strings to `PublicKey` objects
  - Call `app_state.inner_check_state(ys)`
  - Convert response with `ProofState` enum (UNSPENT/PENDING/SPENT)
  - Maintain request/response order consistency

### Critical Considerations:
- **Hex Decoding**: Proper error handling for invalid hex strings
- **Order Preservation**: Response must match request order exactly
- **Bulk Operations**: Efficient handling of multiple proof checks

---

## 📋 Phase 5: Testing & Validation (Priority: CRITICAL)
**Estimated Time**: 1-2 days

### Testing Strategy:
- [ ] **Feature Flag Testing**
  - `cargo build` (gRPC only)
  - `cargo build --features http` (both services)
  - `cargo build --no-default-features --features http` (HTTP only)

- [ ] **Response Consistency Testing**
  - Create test suite comparing gRPC vs HTTP responses
  - Verify identical data for same operations
  - Test error responses match between protocols

- [ ] **Cache Validation Testing**
  - Verify cross-protocol cache hits work correctly
  - Test cache key generation consistency
  - Measure cache hit rates and performance improvement

- [ ] **Integration Testing**
  - Test with real Cashu wallet implementations
  - Validate full request/response cycles
  - Test error scenarios and edge cases

- [ ] **Performance Benchmarking**
  - HTTP response times within 20% of gRPC
  - Memory usage with dual servers
  - Cache effectiveness metrics

### Critical Test Cases:
- **Concurrent Access**: Both services accessing shared state simultaneously
- **Cache Coherence**: Ensure no cache corruption between protocols
- **Error Propagation**: HTTP errors accurately reflect underlying issues
- **Resource Management**: No memory leaks or connection pool exhaustion

---

## 📋 Phase 6: Error Handling & Polish (Priority: MEDIUM)
**Estimated Time**: 0.5 days

### Improvements Needed:
- [ ] **Standardize Error Format**
  - Consistent `ErrorResponse` JSON structure across all endpoints
  - Proper HTTP status code mapping (400, 404, 500, etc.)
  - Include error codes for programmatic handling

- [ ] **Input Validation**
  - Comprehensive validation for all request parameters
  - Size limits matching gRPC implementation
  - Sanitization to prevent injection attacks

- [ ] **Logging & Observability**
  - Add structured logging for all HTTP requests
  - Include correlation IDs for request tracing
  - Metrics for HTTP endpoint usage

---

## 📋 Phase 7: Documentation (Priority: LOW)
**Estimated Time**: 0.5 days

### Documentation Tasks:
- [ ] **Update PRD** with final implementation details
- [ ] **Create API Examples** showing curl commands for each endpoint
- [ ] **Environment Variables** documentation (HTTP_PORT, etc.)
- [ ] **Troubleshooting Guide** for common issues

---

## 🚨 Critical Risks & Mitigation

### HIGH RISK: Missing Melt Quote Logic
**Risk**: Melt quote creation might not exist in current codebase
**Mitigation**: 
1. Investigate existing gRPC implementation first
2. If missing, implement following mint quote patterns
3. Ensure consistency with Cashu NUT-05 specification

### MEDIUM RISK: Cache Synchronization
**Risk**: Potential cache corruption between gRPC and HTTP services
**Mitigation**:
1. Thorough testing of concurrent access patterns
2. Implement cache debugging/monitoring
3. Consider cache isolation if issues arise

### MEDIUM RISK: Performance Impact
**Risk**: Dual servers may impact performance or stability
**Mitigation**:
1. Comprehensive load testing
2. Monitor memory usage and connection pools
3. Implement graceful degradation if needed

---

## ✅ Success Criteria Checklist

### Functional Requirements:
- [ ] All 11 Cashu NUT endpoints implemented and tested
- [ ] Feature flags enable all three compilation modes
- [ ] HTTP responses identical to gRPC for same operations
- [ ] Shared caching improves performance for both protocols
- [ ] Comprehensive error handling with proper HTTP status codes

### Performance Requirements:
- [ ] HTTP response times within 20% of gRPC equivalents
- [ ] Cross-protocol cache hits demonstrate improved efficiency  
- [ ] No memory leaks or resource exhaustion with long-running servers
- [ ] Successful integration with external Cashu wallet implementations

### Compliance Requirements:
- [ ] Full Cashu NUT specification compliance for all endpoints
- [ ] Proper JSON serialization matching Cashu standards
- [ ] Security best practices implemented (input validation, CORS, etc.)

---

## 📅 Realistic Timeline

**Total Estimated Time**: 4-6 additional days

**Critical Path**:
1. **Days 1-2**: NUT-04 Mint + NUT-05 Melt operations
2. **Day 3**: NUT-06 Info + NUT-07 CheckState + comprehensive testing
3. **Day 4**: Error handling, polish, and final validation
4. **Optional Day 5-6**: Documentation and additional testing

**Dependencies**:
- Melt quote investigation must be completed first
- Testing can begin incrementally as endpoints are implemented
- Documentation can be done in parallel with final testing

---

## 🎯 Next Immediate Actions

1. **START HERE**: Investigate melt quote logic in existing codebase
2. **Implement NUT-04**: Mint operations following established patterns
3. **Implement NUT-05**: Melt operations (create missing logic if needed)
4. **Add remaining endpoints**: Info and CheckState
5. **Comprehensive testing**: Feature flags, consistency, performance
6. **Polish and deploy**: Error handling, documentation, final validation

---

*This TODO represents a complete roadmap to finish the HTTP REST API implementation. Each phase builds on the solid foundation already established, following the same patterns for consistency and maintainability.*