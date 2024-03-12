use std::{convert::Infallible, io, sync::Mutex, time::Duration};

use actix_web::{get, middleware::Logger, web, App, HttpRequest, HttpServer, Responder};
use actix_web_lab::{respond::Html, sse};
use async_nats::jetstream::{
    consumer::{push::Config, Consumer, PushConsumer},
    Context,
};
use futures_util::{stream, StreamExt};
use sse_rabbitmq::messaging_service::nats::{conn, consumer};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

struct AppState {
    stream: Mutex<Context>,
}

#[get("/")]
async fn index() -> impl Responder {
    Html(include_str!("../assets/sse.html").to_string())
}

#[get("/stream-2")]
async fn other() -> impl Responder {
    Html(include_str!("../assets/sse-other.html").to_string())
}

/// Countdown event stream starting from 8.
#[get("/sync/{id}")]
async fn sync_status(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let id: i32 = req.match_info().query("id").parse().unwrap();
    // note: a more production-ready implementation might want to use the lastEventId header
    // sent by the reconnecting browser after the _retry_ period
    tracing::debug!("lastEventId: {:?}", req.headers().get("Last-Event-ID"));

    let stream = data.stream.lock().unwrap();

    get_sync_status(stream.clone(), id).await
}

async fn get_sync_status(stream: Context, id: i32) -> impl Responder {
    println!("get sync status: {}", id);
    let consumer = consumer::get_consumer(
        &stream,
        format!("stream_dummy_{}", id).as_str(),
        format!("progress.{}", id).as_str(),
        id.to_string().as_str(),
    )
    .await
    .expect("can't create consumer");

    println!("waiting message");
    let message_stream = stream::unfold((consumer), |(mut consumer)| async move {
        let next_item = consumer.messages().await.unwrap().next().await;
        println!("next item: {:?}", next_item);
        match next_item {
            Some(message) => {
                let message = message.unwrap();
                let message_data = String::from_utf8(message.payload.to_vec()).unwrap();
                let data = sse::Data::new(message_data).event("countdown").id("sync");
                message.ack().await.unwrap();

                Some((Ok::<_, Infallible>(sse::Event::Data(data)), (consumer)))
            }
            None => None,
        }
    });

    sse::Sse::from_stream(message_stream).with_retry_duration(Duration::from_secs(10))
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

    let stream = conn::connect().await.expect("can't create stream");
    HttpServer::new(move || {
        let var_name = AppState {
            stream: Mutex::new(stream.clone()),
        };
        App::new()
            .service(index)
            .service(other)
            .service(sync_status)
            .app_data(web::Data::new(var_name))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
