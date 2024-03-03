use std::sync::{Arc, Mutex};

use rabbitmq_stream_client::{
    error::StreamCreateError,
    types::{ByteCapacity, ResponseCode},
    Environment, Producer, Dedup,
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
        .create("test")
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
pub async fn get_producer(env_rb: &Environment) -> Arc<Mutex<Producer<Dedup>>> {
    let producer = env_rb
        .producer()
        .name("test_producer")
        .build("test")
        .await
        .expect("can't create producer");

    Arc::new(Mutex::new(producer))
}
