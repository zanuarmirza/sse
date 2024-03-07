use std::{sync::Mutex, time::Duration};

use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use rabbitmq_stream_client::{types::Message, Environment};
use sse_rabbitmq::messaging_service::rabbit_mq::publisher;
use tokio::time::sleep;

struct AppState {
    env_rb: Mutex<Environment>,
}

// need one endpoint to trigger publish message
//
#[post("/message")]
async fn send_message(data: web::Data<AppState>) -> impl Responder {
    let env_rb = data.env_rb.lock().unwrap();
    let mut producer = publisher::get_producer(&env_rb, 1).await;
    println!("sending message");
    //todo adding sleep
    sleep(Duration::from_secs(5)).await;

    let time_now = chrono::Utc::now().timestamp();
    producer
        .send_with_confirm(
            Message::builder()
                .body(format!("message{}", time_now))
                .build(),
        )
        .await
        .expect("can't send message");
    "success"
    // clear steram on close connection
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8080");
    println!("log par, {}", std::thread::available_parallelism()?.get());

    let env_rb = publisher::setup().await.expect("can't create environment");

    HttpServer::new(move || {
        let var_name = AppState {
            env_rb: Mutex::new(env_rb.clone()),
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
