use async_nats::jetstream::{self, consumer::{PushConsumer, DeliverPolicy}};

pub async fn get_consumer(
    jetstream: &jetstream::Context,
    stream_name: &str,
    name:&str,
) -> Result<PushConsumer, Box<dyn std::error::Error>> {
    let stream = jetstream
        .get_stream(stream_name.to_owned().as_str())
        .await?;
    // jetstream.publish("events", "data".into()).await?;
    let stream_name = format!("consumer-{}", name);
    let consumer = stream
        .get_or_create_consumer(
            stream_name.to_owned().as_str(),
            jetstream::consumer::push::Config {
                name: Some(stream_name.to_owned()),
                durable_name: None,
                deliver_policy:DeliverPolicy::LastPerSubject,
                filter_subject: format!("progress.{}", name),
                deliver_subject: format!("deliver-{}", name),
                ..Default::default()
            },
        )
        .await?;
    Ok(consumer)
}
