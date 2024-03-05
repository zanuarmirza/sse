use std::{
    convert::Infallible,
    io,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use actix_web::{get, middleware::Logger, web::{self, Data}, App, HttpRequest, HttpServer, Responder};
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

    get_sync_status(&data.consumer).await
}

async fn get_sync_status(consumer: &Arc<Mutex<Consumer>>) -> impl Responder {
    // handle message to stream
    let message_stream = stream::unfold((false, consumer.to_owned()), |(state, mut consumer)| async move {
        let next_item = consumer.lock().unwrap().next().await;
        match next_item {
            Some(delivery) => {
                let delivery = delivery.unwrap();
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

                if state {
                    sleep(Duration::from_secs(3)).await;
                }
                let data = sse::Data::new(message)
                    .event("countdown")
                    .id("sync");

                Some((
                    Ok::<_, Infallible>(sse::Event::Data(data)),
                    (true, consumer),
                ))
            }
            None => {
                consumer.lock().unwrap().handle().close().await.unwrap();
                None
            }
        }
    });

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
            .app_data(Data::new(var_name))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
