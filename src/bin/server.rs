use std::{sync::{Arc, Mutex}, time::Duration};

use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use rabbitmq_stream_client::{types::Message, Dedup, Producer};
use sse_rabbitmq::messaging_service::publisher;
use tokio::time::sleep;

struct AppState {
    producer: Arc<Mutex<Producer<Dedup>>>,
}

// need one endpoint to trigger publish message
//
#[post("/message")]
async fn send_message(data: web::Data<AppState>) -> impl Responder {
    let mut producer = data.producer.lock().unwrap();
    println!("sending message");
    //todo adding sleep
    sleep(Duration::from_secs(5)).await;

    producer
        .send_with_confirm(Message::builder().body(format!("message{}", 1)).build())
        .await
        .expect("can't send message");
    "success"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8080");
    println!("log par, {}", std::thread::available_parallelism()?.get());

    let env_rb = publisher::setup().await.expect("can't create environment");
    let p = publisher::get_producer(&env_rb).await;

    HttpServer::new(move || {
        let var_name = AppState {
            producer: p.clone(),
        };
        App::new()
            .service(send_message)
            .app_data(web::Data::new(var_name))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
