use std::{
    convert::Infallible,
    io,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use actix_web::{get, middleware::Logger, web, App, HttpRequest, HttpServer, Responder};
use actix_web_lab::{respond::Html, sse};
use futures_util::{stream, StreamExt};
use rabbitmq_stream_client::Consumer;
use sse_rabbitmq::messaging_service;
use tokio::time::sleep;
use tracing::info;

struct AppState {
    consumer: Arc<Mutex<Consumer>>,
}

#[get("/")]
async fn index() -> impl Responder {
    Html(include_str!("../assets/sse.html").to_string())
}

/// Countdown event stream starting from 8.
#[get("/sync")]
async fn sync_status(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // note: a more production-ready implementation might want to use the lastEventId header
    // sent by the reconnecting browser after the _retry_ period
    tracing::debug!("lastEventId: {:?}", req.headers().get("Last-Event-ID"));

    let mut consumer = data.consumer.lock().unwrap();
    get_sync_status(consumer).await
}

// todo
// make sure it can't be closed from client 
async fn get_sync_status(mut consumer: MutexGuard<'_, Consumer>) -> impl Responder {
    let delivery = consumer.next().await.unwrap().unwrap();
    info!(
        "Got message : {:?} with offset {}",
        delivery
            .message()
            .data()
            .map(|data| String::from_utf8(data.to_vec())),
        delivery.offset()
    );

    let message = delivery
        .message()
        .data()
        .map(|data| String::from_utf8(data.to_vec()))
        .unwrap()
        .unwrap();
    // handle if message is empty

    // handle message to stream
    let message_stream = stream::unfold((false, message.into_bytes()), |(state,message)| async move {
        let id = 1;
        if state {
            sleep(Duration::from_secs(3)).await;
        }
        let string_message = String::from_utf8(message.to_vec()).unwrap();
        let data = sse::Data::new(string_message).event("countdown").id(id.to_string());

        Some((Ok::<_, Infallible>(sse::Event::Data(data)), (true,message)))
    });

    consumer.handle().close().await.unwrap();
    sse::Sse::from_stream(message_stream).with_retry_duration(Duration::from_secs(5))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8080");
    println!("log par, {}", std::thread::available_parallelism()?.get());

    let env_rb = messaging_service::consumer::setup()
        .await
        .expect("can't create environment");
    let c = messaging_service::consumer::get_consumer(&env_rb).await;

    HttpServer::new(move || {
        let var_name = AppState {
            consumer: c.clone(),
        };
        App::new()
            .service(index)
            .service(sync_status)
            .app_data(var_name)
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
