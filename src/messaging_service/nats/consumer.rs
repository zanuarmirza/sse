use async_nats::jetstream::{self, consumer::PushConsumer};

pub async fn get_consumer(
    jetstream: &jetstream::Context,
    stream_name: &str,
    subject: &str,
) -> Result<PushConsumer, Box<dyn std::error::Error>> {
    let stream = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: stream_name.to_string(),
            subjects: vec![subject.to_string()],
             max_messages: 10_000,
            ..Default::default()
        })
        .await?;
    // jetstream.publish("events", "data".into()).await?;
    let consumer = stream
        .get_or_create_consumer(
            "consumerA",
            jetstream::consumer::push::Config {
                name: Some("consumer1".to_string()),
                durable_name: None,
                deliver_subject: "deliver-1".to_string(),
                ..Default::default()
            },
        )
        .await?;
    Ok(consumer)
}
