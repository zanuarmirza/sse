use std::{convert::Infallible, io, time::Duration};

use actix_web::{get, middleware::Logger, post, App, HttpRequest, HttpServer, Responder};
use actix_web_lab::{extract::Path, respond::Html, sse};
use futures_util::stream;
use time::format_description::well_known::Rfc3339;
use tokio::{sync::mpsc, time::sleep};

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

    get_sync_status()
}

fn get_sync_status() -> impl Responder {
    let countdown_stream = stream::unfold((false), |(started)| async move {
        // allow first countdown value to yield immediately
        if started {
            sleep(Duration::from_secs(1)).await;
        }

        let n = 1;

        let data = sse::Data::new(n.to_string())
            .event("sync_status")
            .id(n.to_string());

        Some((Ok::<_, Infallible>(sse::Event::Data(data)), (true)))
    });

    sse::Sse::from_stream(countdown_stream).with_retry_duration(Duration::from_secs(5))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    tracing::info!("starting HTTP server at http://localhost:8080");
    println!("log par, {}", std::thread::available_parallelism()?.get());

    HttpServer::new(|| {
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
