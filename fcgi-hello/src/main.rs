use std::env;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;

fn main() {
    let args: Vec<String> = env::args().collect();

    let handler = |mut req: fastcgi::Request| {
        let mut stdout = req.stdout();
        write!(stdout, "Status: 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, world!").unwrap();
    };

    if args.len() > 1 {
        let path = &args[1];
        println!("Listening on {}...", path);
        if std::fs::metadata(path).is_ok() {
            std::fs::remove_file(path).unwrap();
        }
        let listener = UnixListener::bind(path).unwrap();
        fastcgi::run_raw(handler, listener.as_raw_fd());
    } else {
        println!("Reading from stdin...");
        let stdin_fd = io::stdin().as_raw_fd();
        fastcgi::run_raw(handler, stdin_fd);
    }
}
