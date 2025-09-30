use std::env;
use tokio::net::UnixListener;
use tokio_fastcgi::{RequestResult, Requests};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let path = &args[1];
        eprintln!("Listening on {}...", path);
        if tokio::fs::metadata(path).await.is_ok() {
            tokio::fs::remove_file(path).await.unwrap();
        }
        let listener = UnixListener::bind(path).unwrap();

        loop {
            if let Ok((socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    eprintln!("Accepted connection");
                    let (reader, writer) = socket.into_split();
                    let mut requests = Requests::from_split_socket((reader, writer), 10, 10);

                    while let Ok(Some(request)) = requests.next().await {
                        if let Err(err) = request
                            .process(|req| async move {
                                let mut stdout = req.get_stdout();
                                let content = format!(
                                    "Status: 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                                    "Hello, world!"
                                );
                                stdout.write(content.as_bytes()).await.unwrap();
                                RequestResult::Complete(0)
                            })
                            .await
                        {
                            eprintln!("Processing request failed: {}", err);
                        }
                    }
                });
            }
        }
    } else {
        eprintln!("Reading from stdin...");
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut requests = Requests::from_split_socket((stdin, stdout), 10, 10);

        while let Ok(Some(request)) = requests.next().await {
            if let Err(err) = request
                .process(|req| async move {
                    let mut stdout = req.get_stdout();
                    let content = format!(
                        "Status: 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
                        "Hello, world!"
                    );
                    stdout.write(content.as_bytes()).await.unwrap();
                    RequestResult::Complete(0)
                })
                .await
            {
                eprintln!("Processing request failed: {}", err);
            }
        }
    }
}
