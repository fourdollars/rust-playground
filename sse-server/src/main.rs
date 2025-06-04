use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web::body::BodyStream;
use futures_util::stream::{self, StreamExt};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::env::args;

async fn sse_handler() -> HttpResponse {
    let mut res_builder = HttpResponse::Ok();
    res_builder.insert_header(("Content-Type", "text/event-stream"))
               .insert_header(("Cache-Control", "no-cache"))
               .insert_header(("Connection", "keep-alive"));

    let initial_data_lines = vec![
        "data: YHOO",
        "data: +2",
        "data: 10",
        "",
        ": test stream",
        "",
        "data: first event",
        "id: 1",
        "",
        "data:second event",
        "id",
        "",
        "data:  third event",
        "",
        "data",
        "",
        "data",
        "data",
        "",
        "data:",
        "",
        "data:test",
        "",
        "data: test",
        "",
        "id: 123",
        "event: ping",
        "data: ping data",
        "retry: 1000",
        "",
    ];

    let initial_event_string = initial_data_lines.into_iter()
        .map(|line| format!("{}\n", line))
        .collect::<String>()
        + "\n";
    let initial_stream = stream::once(async move {
        Ok::<_, actix_web::Error>(web::Bytes::from(initial_event_string))
    });
    let start_time = Instant::now();
    let dynamic_event_stream = stream::repeat(())
        .enumerate()
        .take(15)
        .map(move |(i, _)| {
            let event_id = format!("{}", i + 1);
            let event_type = if (i + 1) % 2 == 0 { "update" } else { "notification" };
            let data_payload = format!("{{\"message\": \"Hello from Rust! Event {}\", \"timestamp\": \"{}\"}}",
                                       i + 1, Instant::now().duration_since(start_time).as_millis());
            let retry_time_ms = 2000;

            let event_string = format!(
                "id: {}\nevent: {}\nretry: {}\ndata: {}\n\n",
                event_id, event_type, retry_time_ms, data_payload
            );

            let mut full_event = event_string.clone();
            if (i + 1) % 3 == 0 {
                full_event += &format!("data: Just a simple data message at {}\n\n",
                                        Instant::now().duration_since(start_time).as_millis());
            }

            async move {
                sleep(Duration::from_secs(1)).await;
                Ok::<_, actix_web::Error>(web::Bytes::from(full_event))
            }
        })
        .flat_map(|fut| stream::once(fut));
    let combined_event_stream = initial_stream.chain(dynamic_event_stream);

    res_builder.body(BodyStream::new(combined_event_stream))
               .into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = args().nth(1).unwrap_or("127.0.0.1".to_string());
    let port = args().nth(2).unwrap_or("8080".to_string());
    println!("Starting SSE server at http://{}:{}{}", host, port, "/events");
    HttpServer::new(|| {
        App::new().route("/events", web::get().to(sse_handler))
    })
    .bind((host, port.parse().unwrap()))?
    .run()
    .await
}
