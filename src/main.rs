use std::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Binding failed");
    println!("Serving on 127.0.0.1:{}", listener.local_addr().unwrap().port());
    run(listener)?.await
}

