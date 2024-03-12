use std::{
    convert::Infallible,
    io,
    time::Duration,
};

use actix_web::{get, middleware::Logger, App, HttpRequest, HttpServer, Responder};
use actix_web_lab::{respond::Html, sse};
use futures_util::{stream, StreamExt};
use rabbitmq_stream_client::Consumer;
use sse_rabbitmq::messaging_service::rabbit_mq;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[get("/")]
async fn index() -> impl Responder {
    Html(include_str!("../assets/sse.html").to_string())
}

/// Countdown event stream starting from 8.
#[get("/sync")]
async fn sync_status(req: HttpRequest) -> impl Responder {
    // note: a more production-ready implementation might want to use the lastEventId header
    // sent by the reconnecting browser after the _retry_ period
    tracing::debug!("lastEventId: {:?}", req.headers().get("Last-Event-ID"));

    let env_rb = rabbit_mq::consumer::setup()
        .await
        .expect("can't create environment");
    // rust get timestamp i64
    let time_now = chrono::Utc::now().timestamp();
    let c = rabbit_mq::consumer::get_consumer(&env_rb, time_now, 1).await;
    get_sync_status(c).await
}

async fn get_sync_status(consumer: Consumer) -> impl Responder {
    // handle message to stream
    let message_stream = stream::unfold(
        (false, consumer),
        |(state, mut consumer)| async move {
            let next_item = consumer.next().await;
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
                    let data = sse::Data::new(message).event("countdown").id("sync");

                    Some((
                        Ok::<_, Infallible>(sse::Event::Data(data)),
                        (true, consumer),
                    ))
                }
                None => {
                    consumer.handle().close().await.unwrap();
                    None
                }
            }
        },
    );

    sse::Sse::from_stream(message_stream).with_retry_duration(Duration::from_secs(5))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8080");
    println!("log par, {}", std::thread::available_parallelism()?.get());

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(sync_status)
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
