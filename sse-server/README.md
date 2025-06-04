# A dummy Rust SSE server

## Server

```bash
$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/sse-server`
Starting SSE server at http://127.0.0.1:8080/events
$ cargo run 0.0.0.0 8765
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/sse-server 0.0.0.0 8765`
Starting SSE server at http://0.0.0.0:8765/events
```

## Client

```bash
$ time curl -v http://127.0.0.1:8765/events
*   Trying 127.0.0.1:8765...
* Connected to 127.0.0.1 (127.0.0.1) port 8765
> GET /events HTTP/1.1
> Host: 127.0.0.1:8765
> User-Agent: curl/8.5.0
> Accept: */*
>
< HTTP/1.1 200 OK
< transfer-encoding: chunked
< cache-control: no-cache
< content-type: text/event-stream
< date: Wed, 28 May 2025 07:45:59 GMT
<
data: YHOO
data: +2
data: 10

: test stream

data: first event
id: 1

data:second event
id

data:  third event

data

data
data

data:

data:test

data: test

id: 1
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 1", "timestamp": "0"}

id: 2
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 2", "timestamp": "1000"}

id: 3
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 3", "timestamp": "2002"}

data: Just a simple data message at 2002

id: 4
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 4", "timestamp": "3004"}

id: 5
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 5", "timestamp": "4005"}

id: 6
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 6", "timestamp": "5007"}

data: Just a simple data message at 5007

id: 7
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 7", "timestamp": "6008"}

id: 8
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 8", "timestamp": "7010"}

id: 9
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 9", "timestamp": "8012"}

data: Just a simple data message at 8012

id: 10
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 10", "timestamp": "9014"}

id: 11
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 11", "timestamp": "10015"}

id: 12
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 12", "timestamp": "11017"}

data: Just a simple data message at 11017

id: 13
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 13", "timestamp": "12018"}

id: 14
event: update
retry: 2000
data: {"message": "Hello from Rust! Event 14", "timestamp": "13019"}

id: 15
event: notification
retry: 2000
data: {"message": "Hello from Rust! Event 15", "timestamp": "14020"}

data: Just a simple data message at 14020

* Connection #0 to host 127.0.0.1 left intact

real    0m15.033s
user    0m0.004s
sys     0m0.009s
```

If you use my forked [sse_client](https://github.com/fourdollars/sse-client) crate, you can do the following:

```rust
use sse_client::EventSource;

fn main() {
    let event_source = EventSource::new("http://127.0.0.1:8765/events").unwrap();

    event_source.add_event_listener("message", |event| {
        println!("Message event {:?}", event);
    });

    event_source.add_event_listener("update", |event| {
        println!("update event {:?}", event);
    });

    event_source.add_event_listener("notification", |event| {
        println!("notification event {:?}", event);
    });

    event_source.add_event_listener("error", |error| {
        println!("Error {:?}", error);
        std::process::exit(1);
    });

    loop {
    }
}
```

Then you can get the following output:

```
Message event Event { event: Some("message"), data: Some("YHOO\n+2\n10"), id: None, retry: None }
Message event Event { event: Some("message"), data: Some("first event"), id: Some("1"), retry: None }
Message event Event { event: Some("message"), data: Some("second event"), id: Some(""), retry: None }
Message event Event { event: Some("message"), data: Some(" third event"), id: None, retry: None }
Message event Event { event: Some("message"), data: Some(""), id: None, retry: None }
Message event Event { event: Some("message"), data: Some("\n"), id: None, retry: None }
Message event Event { event: Some("message"), data: Some(""), id: None, retry: None }
Message event Event { event: Some("message"), data: Some("test"), id: None, retry: None }
Message event Event { event: Some("message"), data: Some("test"), id: None, retry: None }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 1\", \"timestamp\": \"0\"}"), id: Some("1"), retry: Some("2000") }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 2\", \"timestamp\": \"1001\"}"), id: Some("2"), retry: Some("2000") }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 3\", \"timestamp\": \"2002\"}"), id: Some("3"), retry: Some("2000") }
Message event Event { event: Some("message"), data: Some("Just a simple data message at 2002"), id: None, retry: None }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 4\", \"timestamp\": \"3004\"}"), id: Some("4"), retry: Some("2000") }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 5\", \"timestamp\": \"4005\"}"), id: Some("5"), retry: Some("2000") }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 6\", \"timestamp\": \"5007\"}"), id: Some("6"), retry: Some("2000") }
Message event Event { event: Some("message"), data: Some("Just a simple data message at 5007"), id: None, retry: None }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 7\", \"timestamp\": \"6008\"}"), id: Some("7"), retry: Some("2000") }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 8\", \"timestamp\": \"7010\"}"), id: Some("8"), retry: Some("2000") }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 9\", \"timestamp\": \"8012\"}"), id: Some("9"), retry: Some("2000") }
Message event Event { event: Some("message"), data: Some("Just a simple data message at 8012"), id: None, retry: None }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 10\", \"timestamp\": \"9013\"}"), id: Some("10"), retry: Some("2000") }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 11\", \"timestamp\": \"10015\"}"), id: Some("11"), retry: Some("2000") }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 12\", \"timestamp\": \"11016\"}"), id: Some("12"), retry: Some("2000") }
Message event Event { event: Some("message"), data: Some("Just a simple data message at 11016"), id: None, retry: None }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 13\", \"timestamp\": \"12018\"}"), id: Some("13"), retry: Some("2000") }
update event Event { event: Some("update"), data: Some("{\"message\": \"Hello from Rust! Event 14\", \"timestamp\": \"13019\"}"), id: Some("14"), retry: Some("2000") }
notification event Event { event: Some("notification"), data: Some("{\"message\": \"Hello from Rust! Event 15\", \"timestamp\": \"14021\"}"), id: Some("15"), retry: Some("2000") }
Message event Event { event: Some("message"), data: Some("Just a simple data message at 14021"), id: None, retry: None }
Message event Event { event: Some("message"), data: None, id: None, retry: None }
Error Event { event: Some("error"), data: Some("connection closed by server"), id: None, retry: None }
```

## References

* https://html.spec.whatwg.org/multipage/server-sent-events.html

## License

MIT
