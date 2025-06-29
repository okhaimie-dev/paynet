#[cfg(all(feature = "mock", feature = "starknet"))]
compile_error!("Only one of the features 'mock' and 'starknet' can be enabled at the same time");
#[cfg(not(any(feature = "mock", feature = "starknet")))]
compile_error!("At least one liquidity feature should be provided during compilation");

use core::panic;
use std::time::Duration;

use errors::Error;
use gauge::DbMetricsObserver;
use initialization::{
    connect_to_db_and_run_migrations, connect_to_signer, create_app_state, launch_tonic_server_task,
    read_env_variables,
};
#[cfg(feature = "http")]
use initialization::launch_http_server_task;
use tracing::{info, trace};

mod app_state;
mod errors;
mod gauge;
mod grpc_service;
#[cfg(feature = "http")]
mod http_service;
mod initialization;
mod keyset_cache;
#[cfg(feature = "keyset-rotation")]
mod keyset_rotation;
mod liquidity_sources;
mod logic;
mod methods;
mod response_cache;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    let (meter_provider, subscriber) = open_telemetry_tracing::init(PKG_NAME, PKG_VERSION);

    tracing::subscriber::set_global_default(subscriber).unwrap();
    opentelemetry::global::set_meter_provider(meter_provider);

    info!("Initializing node...");
    let args = <initialization::ProgramArguments as clap::Parser>::parse();

    // Read args and env
    let env_variables = read_env_variables()?;

    // Connect to db
    let pg_pool = connect_to_db_and_run_migrations(&env_variables.pg_url).await?;
    info!("Connected to node database.");

    // Lauch the database metrics polling task
    let meter = opentelemetry::global::meter("business");
    let gauge = meter.u64_gauge("stock").build();
    let observer = DbMetricsObserver::new(
        pg_pool.clone(),
        vec![starknet_types::Unit::MilliStrk],
        gauge,
    );
    let _handle = tokio::spawn(gauge::run_metrics_polling(
        observer,
        Duration::from_secs(60),
    ));

    // Connect to the signer service
    let signer_client = connect_to_signer(env_variables.signer_url.clone()).await?;
    info!("Connected to signer server.");

    let liquidity_sources =
        liquidity_sources::LiquiditySources::init(pg_pool.clone(), args).await?;

    // Create shared AppState
    let app_state = create_app_state(
        pg_pool.clone(),
        signer_client,
        liquidity_sources,
        env_variables.quote_ttl,
    ).await?;

    // Launch servers based on enabled features
    match (cfg!(feature = "grpc"), cfg!(feature = "http")) {
        (true, true) => {
            // Both gRPC and HTTP enabled
            info!("Starting both gRPC and HTTP servers");
            let (grpc_address, grpc_future) = launch_tonic_server_task(
                app_state.clone(),
                env_variables.grpc_port,
            ).await?;
            trace!(name: "grpc-listen", port = grpc_address.port());

            #[cfg(feature = "http")]
            let http_future = launch_http_server_task(
                app_state,
                env_variables.http_port,
            );

#[cfg(feature = "http")]
            tokio::select! {
                grpc_res = grpc_future => match grpc_res {
                    Ok(()) => eprintln!("gRPC task should never return"),
                    Err(err) => eprintln!("gRPC task failed: {}", err),
                },
                http_res = http_future => match http_res {
                    Ok(()) => eprintln!("HTTP task should never return"),
                    Err(err) => eprintln!("HTTP task failed: {}", err),
                },
                sig = tokio::signal::ctrl_c() => match sig {
                    Ok(()) => info!("Servers terminated"),
                    Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
                }
            };
            
            #[cfg(not(feature = "http"))]
            tokio::select! {
                grpc_res = grpc_future => match grpc_res {
                    Ok(()) => eprintln!("gRPC task should never return"),
                    Err(err) => eprintln!("gRPC task failed: {}", err),
                },
                sig = tokio::signal::ctrl_c() => match sig {
                    Ok(()) => info!("Servers terminated"),
                    Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
                }
            };
        }
        (true, false) => {
            // Only gRPC enabled
            info!("Starting gRPC server only");
            let (grpc_address, grpc_future) = launch_tonic_server_task(
                app_state,
                env_variables.grpc_port,
            ).await?;
            trace!(name: "grpc-listen", port = grpc_address.port());

            tokio::select! {
                grpc_res = grpc_future => match grpc_res {
                    Ok(()) => eprintln!("gRPC task should never return"),
                    Err(err) => eprintln!("gRPC task failed: {}", err),
                },
                sig = tokio::signal::ctrl_c() => match sig {
                    Ok(()) => info!("gRPC task terminated"),
                    Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
                }
            };
        }
        (false, true) => {
            // Only HTTP enabled
            #[cfg(feature = "http")]
            {
                info!("Starting HTTP server only");
                let http_future = launch_http_server_task(
                    app_state,
                    env_variables.http_port,
                );

                tokio::select! {
                    http_res = http_future => match http_res {
                        Ok(()) => eprintln!("HTTP task should never return"),
                        Err(err) => eprintln!("HTTP task failed: {}", err),
                    },
                    sig = tokio::signal::ctrl_c() => match sig {
                        Ok(()) => info!("HTTP task terminated"),
                        Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
                    }
                };
            }
        }
        (false, false) => {
            panic!("At least one server feature (grpc or http) must be enabled");
        }
    }

    Ok(())
}
