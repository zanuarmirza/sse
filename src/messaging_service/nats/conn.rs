use async_nats::jetstream::{self, Context};
use std::env;

pub async fn connect() -> Result<Context, Box<dyn std::error::Error>> {
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string());
    let client = async_nats::connect(nats_url).await?;
    Ok(jetstream::new(client))
}
pub async fn conn_and_create_stream(stream_name: &str) -> Result<Context, Box<dyn std::error::Error>> {
    let jetstream = connect().await?;
    // jetstream
    //     .get_or_create_stream(jetstream::stream::Config {
    //         name: stream_name.to_string(),
    //         ..Default::default()
    //     })
    //     .await?;
    Ok(jetstream)
}
