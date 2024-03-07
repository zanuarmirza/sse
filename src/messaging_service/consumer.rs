use std::sync::{Arc, Mutex};

use rabbitmq_stream_client::{
    error::StreamCreateError,
    types::{ByteCapacity, OffsetSpecification, ResponseCode},
    Consumer, Environment,
};

pub async fn setup() -> Result<Environment, Box<dyn std::error::Error>> {
    let environment = Environment::builder()
        .host("localhost")
        .port(5552)
        .build()
        .await?;

    // create stream if it doesn't exist
    match environment
        .stream_creator()
        .max_length(ByteCapacity::GB(2))
        .create("stream_a")
        .await
    {
        Ok(_) => Ok(environment),
        Err(err) => match err {
            StreamCreateError::Create {
                stream: _,
                status: ResponseCode::StreamAlreadyExists,
            } => Ok(environment),
            _ => Err(Box::new(err)),
        },
    }
}
pub async fn get_consumer(env_rb: &Environment, time: i64,id:i32) -> Consumer {

    tracing::debug!("time: {:?}", time);
    let consumer = env_rb
        .consumer()
        .offset(OffsetSpecification::Last)
        .name(format!("stream_b{}", id).as_str())
        .build("stream_a")
        .await
        .unwrap();

    consumer
}
