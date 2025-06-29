#[cfg(feature = "http")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

#[cfg(feature = "http")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "http")]
use nuts::{
    Amount,
    nut00::{BlindedMessage, Proof, secret::Secret},
    nut01::PublicKey,
    nut02::KeysetId,
    nut19::Route,
};

#[cfg(feature = "http")]
use tonic::Request;

#[cfg(feature = "http")]
use std::str::FromStr;

#[cfg(feature = "http")]
use uuid::Uuid;

#[cfg(feature = "http")]
use crate::{
    app_state::AppState,
    methods::Method,
    response_cache::CachedResponse,
};

#[cfg(feature = "http")]
use node::{
    SwapResponse as GrpcSwapResponse, BlindSignature as GrpcBlindSignature,
    MintResponse as GrpcMintResponse, MeltResponse as GrpcMeltResponse,
};

#[cfg(feature = "http")]
use tracing;

#[cfg(feature = "http")]
use starknet_types::Unit;

/// HTTP error response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Cashu NUT-01: Get keysets
#[cfg(feature = "http")]
#[derive(Serialize)]
struct GetKeysetsResponse {
    keysets: Vec<KeysetInfo>,
}

#[cfg(feature = "http")]
#[derive(Serialize)]
struct KeysetInfo {
    id: String,
    unit: String,
    active: bool,
}

/// Cashu NUT-01: Get keys for keysets
#[cfg(feature = "http")]
#[derive(Serialize)]
struct GetKeysResponse {
    keysets: Vec<KeysetKeys>,
}

#[cfg(feature = "http")]
#[derive(Serialize)]
struct KeysetKeys {
    id: String,
    unit: String,
    active: bool,
    keys: std::collections::HashMap<u64, String>,
}

/// Cashu NUT-03: Swap request
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct SwapRequest {
    inputs: Vec<ProofHttp>,
    outputs: Vec<BlindedMessageHttp>,
}

/// Cashu NUT-03: Swap response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct SwapResponse {
    signatures: Vec<BlindSignatureHttp>,
}

/// HTTP representation of Proof
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct ProofHttp {
    amount: u64,
    #[serde(rename = "id")]
    keyset_id: String,
    secret: String,
    #[serde(rename = "C")]
    c: String,
}

/// HTTP representation of BlindedMessage
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct BlindedMessageHttp {
    amount: u64,
    #[serde(rename = "id")]
    keyset_id: String,
    #[serde(rename = "B_")]
    blinded_secret: String,
}

/// HTTP representation of BlindSignature
#[cfg(feature = "http")]
#[derive(Serialize)]
struct BlindSignatureHttp {
    amount: u64,
    #[serde(rename = "id")]
    keyset_id: String,
    #[serde(rename = "C_")]
    c: String,
}

/// Cashu NUT-04: Mint quote request
#[cfg(feature = "http")]
#[derive(Deserialize)]
struct MintQuoteRequest {
    unit: String,
    amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// Cashu NUT-04: Mint quote response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct MintQuoteResponse {
    quote: String,
    request: String,
    state: String,
    expiry: u64,
}

/// Cashu NUT-04: Mint request
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct MintRequest {
    quote: String,
    outputs: Vec<BlindedMessageHttp>,
}

/// Cashu NUT-05: Melt quote request
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct MeltQuoteRequest {
    request: String,
    unit: String,
}

/// Cashu NUT-05: Melt quote response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct MeltQuoteResponse {
    quote: String,
    amount: u64,
    fee: u64,
    state: String,
    expiry: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    transfer_ids: Vec<String>,
}

/// Cashu NUT-04: Mint response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct MintResponse {
    signatures: Vec<BlindSignatureHttp>,
}

/// Cashu NUT-05: Melt request
#[cfg(feature = "http")]
#[derive(Deserialize, Debug)]
struct MeltRequest {
    quote: String,
    inputs: Vec<ProofHttp>,
}

/// Cashu NUT-05: Melt response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct MeltResponse {
    quote: String,
    amount: u64,
    fee: u64,
    state: String,
    expiry: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    transfer_ids: Vec<String>,
}

/// Cashu NUT-06: Node info response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct NodeInfoResponse {
    // This will be the JSON string from the gRPC response
    #[serde(flatten)]
    info: serde_json::Value,
}

/// Cashu NUT-07: Check state request
#[cfg(feature = "http")]
#[derive(Deserialize)]
struct CheckStateRequest {
    #[serde(rename = "Ys")]
    ys: Vec<String>,
}

/// Cashu NUT-07: Check state response
#[cfg(feature = "http")]
#[derive(Serialize)]
struct CheckStateResponse {
    states: Vec<ProofState>,
}

#[cfg(feature = "http")]
#[derive(Serialize)]
struct ProofState {
    #[serde(rename = "Y")]
    y: String,
    state: String,
}

#[cfg(feature = "http")]
fn hash_swap_request_http(request: &SwapRequest) -> u64 {
    // For now, use a simple hash - we should match the gRPC implementation
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "http")]
fn hash_mint_request_http(request: &MintRequest) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "http")]
fn hash_melt_request_http(request: &MeltRequest) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    hasher.finish()
}

/// Convert HTTP proof to internal Proof
#[cfg(feature = "http")]
fn convert_proof_from_http(proof_http: ProofHttp) -> Result<Proof, String> {
    Ok(Proof {
        amount: Amount::from(proof_http.amount),
        keyset_id: KeysetId::from_str(&proof_http.keyset_id)
            .map_err(|e| format!("Invalid keyset_id: {}", e))?,
        secret: Secret::new(proof_http.secret)
            .map_err(|e| format!("Invalid secret: {}", e))?,
        c: PublicKey::from_str(&proof_http.c)
            .map_err(|e| format!("Invalid signature: {}", e))?,
    })
}

/// Convert HTTP blinded message to internal BlindedMessage
#[cfg(feature = "http")]
fn convert_blinded_message_from_http(bm_http: BlindedMessageHttp) -> Result<BlindedMessage, String> {
    Ok(BlindedMessage {
        amount: Amount::from(bm_http.amount),
        keyset_id: KeysetId::from_str(&bm_http.keyset_id)
            .map_err(|e| format!("Invalid keyset_id: {}", e))?,
        blinded_secret: PublicKey::from_str(&bm_http.blinded_secret)
            .map_err(|e| format!("Invalid blinded_secret: {}", e))?,
    })
}

/// NUT-01: Get keysets
#[cfg(feature = "http")]
async fn get_keysets(State(app_state): State<AppState>) -> Result<Json<GetKeysetsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut conn = app_state.pg_pool
        .acquire()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let keysets = db_node::keyset::get_keysets(&mut conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
        .map(|(id, unit, active)| KeysetInfo {
            id: hex::encode(id),
            unit,
            active,
        })
        .collect();

    Ok(Json(GetKeysetsResponse { keysets }))
}

/// NUT-01: Get keys
#[cfg(feature = "http")]
async fn get_keys(
    State(app_state): State<AppState>,
    keyset_id: Option<Path<String>>,
) -> Result<Json<GetKeysResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut db_conn = app_state.pg_pool
        .acquire()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let keysets = match keyset_id {
        Some(Path(keyset_id_str)) => {
            let keyset_id_bytes = hex::decode(&keyset_id_str)
                .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid keyset_id: {}", e) })))?;
            app_state.inner_keys_for_keyset_id(&mut db_conn, keyset_id_bytes)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
        }
        None => app_state.inner_keys_no_keyset_id(&mut db_conn)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
    };

    let response_keysets = keysets.into_iter().map(|keyset| {
        let keys = keyset.keys.into_iter()
            .map(|key| (key.amount, key.pubkey))
            .collect();
        
        KeysetKeys {
            id: hex::encode(keyset.id),
            unit: keyset.unit,
            active: keyset.active,
            keys,
        }
    }).collect();

    Ok(Json(GetKeysResponse { keysets: response_keysets }))
}

/// NUT-03: Swap tokens
#[cfg(feature = "http")]
async fn swap(
    State(app_state): State<AppState>,
    Json(request): Json<SwapRequest>,
) -> Result<Json<SwapResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Swap, hash_swap_request_http(&request));
    
    // Try to get from cache first
    if let Some(CachedResponse::Swap(swap_response)) = app_state.get_cached_response(&cache_key) {
        let http_response = SwapResponse {
            signatures: swap_response.signatures.into_iter().map(|sig| BlindSignatureHttp {
                amount: sig.amount,
                keyset_id: hex::encode(sig.keyset_id),
                c: hex::encode(sig.blind_signature),
            }).collect(),
        };
        return Ok(Json(http_response));
    }

    if request.inputs.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Too many inputs: maximum allowed is 64".to_string() })));
    }
    if request.outputs.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Too many outputs: maximum allowed is 64".to_string() })));
    }
    if request.inputs.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Inputs cannot be empty".to_string() })));
    }
    if request.outputs.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Outputs cannot be empty".to_string() })));
    }

    // Convert HTTP types to internal types
    let inputs = request.inputs.into_iter()
        .map(convert_proof_from_http)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let outputs = request.outputs.into_iter()
        .map(convert_blinded_message_from_http)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let promises = app_state.inner_swap(&inputs, &outputs)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let signatures = promises.iter().map(|p| BlindSignatureHttp {
        amount: p.amount.into(),
        keyset_id: hex::encode(p.keyset_id.to_bytes()),
        c: hex::encode(p.c.to_bytes()),
    }).collect();

    let swap_response = SwapResponse { signatures };

    // Store in cache (convert back to gRPC format for caching)
    let grpc_response = GrpcSwapResponse {
        signatures: promises.iter().map(|p| GrpcBlindSignature {
            amount: p.amount.into(),
            keyset_id: p.keyset_id.to_bytes().to_vec(),
            blind_signature: p.c.to_bytes().to_vec(),
        }).collect(),
    };
    
    if let Err(e) = app_state.cache_response(cache_key, CachedResponse::Swap(grpc_response)) {
        tracing::warn!("Failed to cache swap response: {}", e);
    }

    Ok(Json(swap_response))
}

/// NUT-04: Request mint quote
#[cfg(feature = "http")]
async fn mint_quote(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<MintQuoteRequest>,
) -> Result<Json<MintQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid method: {}", e) })))?;
    let amount = Amount::from(request.amount);
    let unit = Unit::from_str(&request.unit)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid unit: {}", e) })))?;

    let response = app_state.inner_mint_quote(method, amount, unit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(Json(MintQuoteResponse {
        quote: response.quote.to_string(),
        request: response.request,
        state: match response.state {
            nuts::nut04::MintQuoteState::Unpaid => "UNPAID".to_string(),
            nuts::nut04::MintQuoteState::Paid => "PAID".to_string(),
            nuts::nut04::MintQuoteState::Issued => "ISSUED".to_string(),
        },
        expiry: response.expiry,
    }))
}

/// NUT-04: Check mint quote state
#[cfg(feature = "http")]
async fn mint_quote_state(
    State(app_state): State<AppState>,
    Path((method, quote_id)): Path<(String, String)>,
) -> Result<Json<MintQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid method: {}", e) })))?;
    let quote_id = Uuid::from_str(&quote_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid quote ID: {}", e) })))?;

    let response = app_state.inner_mint_quote_state(method, quote_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(Json(MintQuoteResponse {
        quote: response.quote.to_string(),
        request: response.request,
        state: match response.state {
            nuts::nut04::MintQuoteState::Unpaid => "UNPAID".to_string(),
            nuts::nut04::MintQuoteState::Paid => "PAID".to_string(),
            nuts::nut04::MintQuoteState::Issued => "ISSUED".to_string(),
        },
        expiry: response.expiry,
    }))
}

/// NUT-04: Execute mint
#[cfg(feature = "http")]
async fn mint(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<MintRequest>,
) -> Result<Json<MintResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Mint, hash_mint_request_http(&request));
    
    // Try to get from cache first
    if let Some(CachedResponse::Mint(mint_response)) = app_state.get_cached_response(&cache_key) {
        let http_response = MintResponse {
            signatures: mint_response.signatures.into_iter().map(|sig| BlindSignatureHttp {
                amount: sig.amount,
                keyset_id: hex::encode(sig.keyset_id),
                c: hex::encode(sig.blind_signature),
            }).collect(),
        };
        return Ok(Json(http_response));
    }

    if request.outputs.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Too many outputs: maximum allowed is 64".to_string() })));
    }
    if request.outputs.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Outputs cannot be empty".to_string() })));
    }

    let method = Method::from_str(&method)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid method: {}", e) })))?;
    let quote_id = Uuid::from_str(&request.quote)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid quote ID: {}", e) })))?;

    // Convert HTTP types to internal types
    let outputs = request.outputs.into_iter()
        .map(convert_blinded_message_from_http)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let promises = app_state.inner_mint(method, quote_id, &outputs)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let signatures = promises.iter().map(|p| BlindSignatureHttp {
        amount: p.amount.into(),
        keyset_id: hex::encode(p.keyset_id.to_bytes()),
        c: hex::encode(p.c.to_bytes()),
    }).collect();

    let mint_response = MintResponse { signatures };

    // Store in cache (convert back to gRPC format for caching)
    let grpc_response = GrpcMintResponse {
        signatures: promises.iter().map(|p| GrpcBlindSignature {
            amount: p.amount.into(),
            keyset_id: p.keyset_id.to_bytes().to_vec(),
            blind_signature: p.c.to_bytes().to_vec(),
        }).collect(),
    };
    
    if let Err(e) = app_state.cache_response(cache_key, CachedResponse::Mint(grpc_response)) {
        tracing::warn!("Failed to cache mint response: {}", e);
    }

    Ok(Json(mint_response))
}

/// NUT-05: Request melt quote  
#[cfg(feature = "http")]
async fn melt_quote(
    State(_app_state): State<AppState>,
    Path(_method): Path<String>,
    Json(_request): Json<MeltQuoteRequest>,
) -> Result<Json<MeltQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement proper melt quote creation logic
    // Currently the melt operation in inner_melt combines quote creation and execution
    // For proper NUT-05 compliance, this should be separated
    Err((StatusCode::NOT_IMPLEMENTED, Json(ErrorResponse { 
        error: "Melt quote creation not yet implemented".to_string() 
    })))
}

/// NUT-05: Check melt quote state
#[cfg(feature = "http")]
async fn melt_quote_state(
    State(app_state): State<AppState>,
    Path((method, quote_id)): Path<(String, String)>,
) -> Result<Json<MeltQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid method: {}", e) })))?;
    let quote_id = Uuid::from_str(&quote_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid quote ID: {}", e) })))?;

    let response = app_state.inner_melt_quote_state(method, quote_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(Json(MeltQuoteResponse {
        quote: response.quote.to_string(),
        amount: response.amount.into(),
        fee: response.fee.into(),
        state: match response.state {
            nuts::nut05::MeltQuoteState::Unpaid => "UNPAID".to_string(),
            nuts::nut05::MeltQuoteState::Pending => "PENDING".to_string(),
            nuts::nut05::MeltQuoteState::Paid => "PAID".to_string(),
        },
        expiry: response.expiry,
        transfer_ids: response.transfer_ids.clone().unwrap_or_default(),
    }))
}

/// NUT-05: Execute melt
#[cfg(feature = "http")]
async fn melt(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<MeltRequest>,
) -> Result<Json<MeltResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Melt, hash_melt_request_http(&request));
    
    // Try to get from cache first
    if let Some(CachedResponse::Melt(melt_response)) = app_state.get_cached_response(&cache_key) {
        let http_response = MeltResponse {
            quote: melt_response.quote,
            amount: melt_response.amount,
            fee: melt_response.fee,
            state: match melt_response.state {
                0 => "UNSPECIFIED".to_string(),
                1 => "UNPAID".to_string(),
                2 => "PENDING".to_string(),
                3 => "PAID".to_string(),
                _ => "UNKNOWN".to_string(),
            },
            expiry: melt_response.expiry,
            transfer_ids: melt_response.transfer_ids,
        };
        return Ok(Json(http_response));
    }

    if request.inputs.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Too many inputs: maximum allowed is 64".to_string() })));
    }
    if request.inputs.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Inputs cannot be empty".to_string() })));
    }

    let method = Method::from_str(&method)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Invalid method: {}", e) })))?;
    
    // For melt, we need the unit and request string - these should be in the request
    // Following gRPC pattern (lines 217-237), it uses method, unit, request, inputs
    // We need to get unit and request from somewhere - let's assume they're in the JSON
    let unit = Unit::MilliStrk; // TODO: This should come from request
    let payment_request = request.quote.clone(); // TODO: This should be the actual payment request

    // Convert HTTP types to internal types
    let inputs = request.inputs.into_iter()
        .map(convert_proof_from_http)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let response = app_state.inner_melt(method, unit, payment_request, &inputs)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let melt_response = MeltResponse {
        quote: response.quote.to_string(),
        amount: response.amount.into(),
        fee: response.fee.into(),
        state: match response.state {
            nuts::nut05::MeltQuoteState::Unpaid => "UNPAID".to_string(),
            nuts::nut05::MeltQuoteState::Pending => "PENDING".to_string(),
            nuts::nut05::MeltQuoteState::Paid => "PAID".to_string(),
        },
        expiry: response.expiry,
        transfer_ids: response.transfer_ids.clone().unwrap_or_default(),
    };

    // Store in cache (convert back to gRPC format for caching)
    let grpc_response = GrpcMeltResponse {
        quote: response.quote.to_string(),
        amount: response.amount.into(),
        fee: response.fee.into(),
        state: node::MeltState::from(response.state).into(),
        expiry: response.expiry,
        transfer_ids: response.transfer_ids.clone().unwrap_or_default(),
    };
    
    if let Err(e) = app_state.cache_response(cache_key, CachedResponse::Melt(grpc_response)) {
        tracing::warn!("Failed to cache melt response: {}", e);
    }

    Ok(Json(melt_response))
}

/// NUT-06: Get node info
#[cfg(feature = "http")]
async fn get_node_info(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let nuts_config = {
        let nuts_read_lock = app_state.nuts.read().await;
        nuts_read_lock.clone()
    };
    let pub_key = app_state
        .signer
        .clone()
        .get_root_pub_key(Request::new(signer::GetRootPubKeyRequest {}))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
        .into_inner()
        .root_pubkey;
    
    let node_info = nuts::nut06::NodeInfo {
        name: Some("Paynet Test Node".to_string()),
        pubkey: Some(
            PublicKey::from_str(&pub_key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
        ),
        version: Some(nuts::nut06::NodeVersion {
            name: "some_name".to_string(),
            version: "0.0.0".to_string(),
        }),
        description: Some("A test node".to_string()),
        description_long: Some("This is a longer description of the test node.".to_string()),
        contact: Some(vec![nuts::nut06::ContactInfo {
            method: "some_method".to_string(),
            info: "some_info".to_string(),
        }]),
        nuts: nuts_config,
        icon_url: Some("http://example.com/icon.png".to_string()),
        urls: Some(vec!["http://example.com".to_string()]),
        motd: Some("Welcome to the node!".to_string()),
        time: Some(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()),
    };

    let node_info_json = serde_json::to_value(&node_info)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(Json(node_info_json))
}

/// NUT-07: Check proof states
#[cfg(feature = "http")]
async fn check_state(
    State(app_state): State<AppState>,
    Json(request): Json<CheckStateRequest>,
) -> Result<Json<CheckStateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ys: Vec<PublicKey> = request.ys.iter()
        .map(|y_hex| hex::decode(y_hex)
            .map_err(|e| format!("Invalid hex: {}", e))
            .and_then(|bytes| PublicKey::from_slice(&bytes)
                .map_err(|e| format!("Invalid public key: {}", e))))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let proof_state = app_state.inner_check_state(ys)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let states = proof_state.proof_check_states.iter()
        .map(|state| ProofState {
            y: hex::encode(state.y.to_bytes()),
            state: match state.state {
                nuts::nut07::ProofState::Unspent => "UNSPENT".to_string(),
                nuts::nut07::ProofState::Pending => "PENDING".to_string(),
                nuts::nut07::ProofState::Spent => "SPENT".to_string(),
                nuts::nut07::ProofState::Unspecified => "UNSPECIFIED".to_string(),
            },
        })
        .collect();

    Ok(Json(CheckStateResponse { states }))
}

#[cfg(feature = "http")]
pub fn create_router() -> Router<AppState> {
    Router::new()
        // NUT-01: Keys
        .route("/v1/keys", get(get_keys))
        .route("/v1/keys/:keyset_id", get(|State(app_state): State<AppState>, Path(keyset_id): Path<String>| async move {
            get_keys(State(app_state), Some(Path(keyset_id))).await
        }))
        // NUT-03: Swap
        .route("/v1/swap", post(swap))
        // NUT-04: Mint
        .route("/v1/mint/quote/:method", post(mint_quote))
        .route("/v1/mint/quote/:method/:quote_id", get(mint_quote_state))
        .route("/v1/mint/:method", post(mint))
        // NUT-05: Melt
        .route("/v1/melt/quote/:method", post(melt_quote))
        .route("/v1/melt/quote/:method/:quote_id", get(melt_quote_state))
        .route("/v1/melt/:method", post(melt))
        // NUT-06: Info
        .route("/v1/info", get(get_node_info))
        // NUT-07: Check state
        .route("/v1/checkstate", post(check_state))
}