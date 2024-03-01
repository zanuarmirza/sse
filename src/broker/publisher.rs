use rabbitmq_stream_client::{types::{ByteCapacity}, Environment, Dedup};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub async fn setup() -> Result<rabbitmq_stream_client::Producer<Dedup>, Box<dyn std::error::Error>> {
    let environment = Environment::builder()
        .host("172.17.0.3")
        .port(5552)
        .build()
        .await?;

    environment
        .stream_creator()
        .max_length(ByteCapacity::GB(2))
        .create("test")
        .await?;

    let producer = environment
        .producer()
        .name("test_producer")
        .build("test")
        .await?;

    Ok(producer)
}
