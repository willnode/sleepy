use lazy_static::lazy_static;
use std::env;
use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::proxy::proxy_handler;

mod bucket;
mod proxy;

lazy_static! {
    static ref TARGET: SocketAddr = get_env_adr("TARGET", SocketAddr::from(([127, 0, 0, 1], 3000)));
    static ref UPSTREAM: SocketAddr =
        get_env_adr("UPSTREAM", SocketAddr::from(([127, 0, 0, 1], 4000)));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(*TARGET).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(proxy_handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn get_env_adr(key: &str, default: SocketAddr) -> SocketAddr {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
