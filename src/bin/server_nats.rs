use std::{sync::Mutex, time::Duration};

use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use async_nats::jetstream::Context;
use serde::Deserialize;
use sse_rabbitmq::messaging_service::nats::conn::conn_and_create_stream;
use tokio::time::sleep;

struct AppState {
    stream: Mutex<Context>,
}

#[derive(Deserialize)]
struct Message {
    id: i32,
}

#[post("/message")]
async fn send_message(info: web::Json<Message>, data: web::Data<AppState>) -> impl Responder {
    let stream = data.stream.lock().unwrap();
    println!("processing document in 2 seconds");
    println!("stream: {:?}", stream);
    sleep(Duration::from_secs(5)).await;
    stream
        .publish(format!("progress.{}", info.id), "Hello".to_string().into())
        .await
        .unwrap();

    "success"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8081");
    println!("log par, {}", std::thread::available_parallelism()?.get());

    let stream = conn_and_create_stream().await.expect("can't create stream");

    HttpServer::new(move || {
        let var_name = AppState {
            stream: Mutex::new(stream.clone()),
        };
        App::new()
            .service(send_message)
            .app_data(web::Data::new(var_name))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
