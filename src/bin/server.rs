use std::time::Duration;

use actix_web::{get, middleware::Logger, App, HttpServer, Responder};
use actix_web_lab::sse;
use rabbitmq_stream_client::types::Message;
use sse_rabbitmq::broker::{publisher};


// need one endpoint to trigger publish message
//
#[get("/send")]
async fn send_message() -> impl Responder {
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let _ = tx.send(sse::Event::Comment("my comment".into())).await;
    let _ = tx
        .send(sse::Data::new("my data").event("chat_msg").into())
        .await;
    sse::Sse::from_infallible_receiver(rx).with_retry_duration(Duration::from_secs(5))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8080");
    println!("log par, {}", std::thread::available_parallelism()?.get());
    let mut rb_producer = publisher::setup().await.expect("can't create a pool");

    for i in 0..10 {
        rb_producer
            .send_with_confirm(Message::builder().body(format!("message{}", i)).build())
            .await.expect("can't send message");
    }

    rb_producer.close().await.expect("can't close producer");
    HttpServer::new(|| App::new().service(send_message).wrap(Logger::default()))
        .workers(2)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
    // todo setup rabbit server
}
