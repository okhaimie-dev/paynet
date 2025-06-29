use tonic::transport::Channel;
use tower_otel::trace;
use tracing::Level;

use crate::app_state::SignerClient;

use super::Error;

pub async fn connect_to_signer(signer_url: String) -> Result<SignerClient, Error> {
    let endpoint = Channel::from_shared(signer_url).expect("Invalid signer URL");
    let channel = endpoint
        .connect()
        .await
        .map_err(Error::SignerConnection)?;
    let channel = tower::ServiceBuilder::new()
        .layer(trace::GrpcLayer::client(Level::INFO))
        .service(channel);

    Ok(signer::SignerClient::new(channel))
}
