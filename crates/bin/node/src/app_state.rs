use std::sync::{Arc, atomic::AtomicU64};

use nuts::{
    Amount, QuoteTTLConfig,
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
    nut06::NutsSettings,
    nut19::{CacheResponseKey, Route},
};
use sqlx::PgPool;
use starknet_types::Unit;
use std::str::FromStr;
use tokio::sync::RwLock;
use tonic::{Status, transport::Channel};

use crate::{
    keyset_cache::{CachedKeysetInfo, KeysetCache},
    liquidity_sources::LiquiditySources,
    methods::Method,
    response_cache::{CachedResponse, InMemResponseCache, ResponseCache},
};

pub type NutsSettingsState = Arc<RwLock<NutsSettings<Method, Unit>>>;
pub type SignerClient = signer::SignerClient<tower_otel::trace::Grpc<Channel>>;

/// Quote Time To Live config
///
/// Specifies for how long, in seconds, the quote issued by the node will be valid.
///
/// We use AtomicU64 to share this easily between threads.
#[derive(Debug)]
pub struct QuoteTTLConfigState {
    mint_ttl: AtomicU64,
    melt_ttl: AtomicU64,
}

impl QuoteTTLConfigState {
    /// Returns the number of seconds a new mint quote is valid for
    pub fn mint_ttl(&self) -> u64 {
        self.mint_ttl.load(std::sync::atomic::Ordering::Relaxed)
    }
    /// Returns the number of seconds a new melt quote is valid for
    pub fn melt_ttl(&self) -> u64 {
        self.melt_ttl.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl From<QuoteTTLConfig> for QuoteTTLConfigState {
    fn from(value: QuoteTTLConfig) -> Self {
        Self {
            mint_ttl: value.mint_ttl.into(),
            melt_ttl: value.melt_ttl.into(),
        }
    }
}

/// Shared application state used by both gRPC and HTTP services
#[derive(Debug, Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub signer: SignerClient,
    pub keyset_cache: KeysetCache,
    pub nuts: NutsSettingsState,
    pub quote_ttl: Arc<QuoteTTLConfigState>,
    pub liquidity_sources: LiquiditySources<Unit>,
    pub response_cache: Arc<InMemResponseCache<(Route, u64), CachedResponse>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InitKeysetError {
    #[error(transparent)]
    Tonic(#[from] tonic::Status),
    #[error(transparent)]
    Nut01(#[from] nut01::Error),
    #[error(transparent)]
    Nut02(#[from] nut02::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl AppState {
    pub fn new(
        pg_pool: PgPool,
        signer_client: SignerClient,
        nuts_settings: NutsSettings<Method, Unit>,
        quote_ttl: QuoteTTLConfig,
        liquidity_sources: LiquiditySources<Unit>,
    ) -> Self {
        Self {
            pg_pool,
            keyset_cache: Default::default(),
            nuts: Arc::new(RwLock::new(nuts_settings)),
            quote_ttl: Arc::new(quote_ttl.into()),
            signer: signer_client,
            liquidity_sources,
            response_cache: Arc::new(InMemResponseCache::new(None)),
        }
    }

    pub async fn init_first_keysets(
        &self,
        units: &[Unit],
        index: u32,
        max_order: u32,
    ) -> Result<(), InitKeysetError> {
        let mut insert_keysets_query_builder = db_node::InsertKeysetsQueryBuilder::new();

        for unit in units {
            let response = self
                .signer
                .clone()
                .declare_keyset(signer::DeclareKeysetRequest {
                    unit: unit.to_string(),
                    index,
                    max_order,
                })
                .await?;
            let response = response.into_inner();
            let keyset_id = KeysetId::from_bytes(&response.keyset_id)?;

            insert_keysets_query_builder.add_row(keyset_id, unit, max_order, index);

            self.keyset_cache
                .insert_info(keyset_id, CachedKeysetInfo::new(true, *unit, max_order))
                .await;

            let keys = response
                .keys
                .into_iter()
                .map(|k| -> Result<(Amount, PublicKey), InitKeysetError> {
                    Ok((
                        Amount::from(k.amount),
                        PublicKey::from_str(&k.pubkey).map_err(InitKeysetError::Nut01)?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            self.keyset_cache
                .insert_keys(keyset_id, keys.into_iter())
                .await;
        }

        let mut conn = self.pg_pool.acquire().await?;
        insert_keysets_query_builder.execute(&mut conn).await?;

        Ok(())
    }

    pub fn get_cached_response(&self, cache_key: &CacheResponseKey) -> Option<CachedResponse> {
        if let Some(cached_response) = self.response_cache.get(cache_key) {
            return Some(cached_response);
        }

        None
    }

    pub fn cache_response(
        &self,
        cache_key: (Route, u64),
        response: CachedResponse,
    ) -> Result<(), Status> {
        self.response_cache
            .insert(cache_key, response)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(())
    }
}
