use std::time::Duration;

use dashmap::DashMap;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use http_body_util::{Empty, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use once_cell::sync::Lazy;
use std::convert::Infallible;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::bucket::RateLimiter;

pub static VISITORS: Lazy<DashMap<String, RateLimiter>> = Lazy::new(|| DashMap::new());

pub async fn proxy_handler(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    let headers = req.headers();
    let mut set_cookie_header = None;

    let session_id = extract_cookie_or_set(headers, &mut set_cookie_header);

    let pool: Response<Incoming>;
    let mut duration = Duration::from_millis(0);
    let mut weight: u32 = 0;

    match do_the_proxy(req).await {
        Ok((p, e)) => {
            pool = p;
            if pool.status() < StatusCode::INTERNAL_SERVER_ERROR {
                let mut visitor = VISITORS.get_mut(&session_id);
                if visitor.is_none() {
                    VISITORS.insert(session_id.clone(), RateLimiter::new());
                    visitor = VISITORS.get_mut(&session_id);
                }
                let mut val = visitor.unwrap();
                let visitor = val.value_mut();
                visitor.on_response(e);
                duration = visitor.get_penalty_delay();
                weight = visitor.get_budget();
            }
        }
        Err(_) => {
            return Ok(Response::new(full("Bad gateway")));
        }
    }

    let mut resp = pool.map(|body| {
        let boxed: BoxBody<Bytes, hyper::Error> = BoxBody::new(body);
        boxed
    });

    if !duration.is_zero() {
        tokio::time::sleep(duration).await;
    }

     resp.headers_mut().insert(
            "X-Sleepy-Weight",
            hyper::header::HeaderValue::from_str(&weight.to_string()).unwrap(),
        );

    if let Some(set_cookie) = set_cookie_header {
        resp.headers_mut().insert(
            hyper::header::SET_COOKIE,
            hyper::header::HeaderValue::from_str(&set_cookie).unwrap(),
        );
    }

    Ok(resp)
}

fn extract_cookie_or_set(
    headers: &hyper::HeaderMap,
    set_cookie_header: &mut Option<String>,
) -> String {
    let session_id = headers
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|cookie| {
            cookie.split(';').find_map(|kv| {
                let kv = kv.trim();
                if kv.starts_with("sleepy-session=") {
                    Some(kv.trim_start_matches("sleepy-session=").to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| {
            // Generate a new session ID and set a cookie valid for 1 month
            let new_id = Uuid::new_v4().to_string();
            let one_month = 60 * 60 * 24 * 30; // seconds
            *set_cookie_header = Some(format!(
                "sleepy-session={}; Max-Age={}; Path=/; HttpOnly; SameSite=Strict",
                new_id, one_month
            ));
            new_id
        });
    session_id
}

async fn do_the_proxy(
    req: Request<hyper::body::Incoming>,
) -> Result<(Response<Incoming>, Duration), Error> {
    let start = tokio::time::Instant::now();

    let stream = TcpStream::connect("localhost:4000").await.unwrap();
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    // The authority of our URL will be the hostname of the httpbin remote
    let url = "http://localhost:4000".parse::<hyper::Uri>().unwrap();
    let authority = url.authority().unwrap().clone();

    // Create an HTTP request with an empty body and a HOST header

    let req = Request::builder()
        .uri(req.uri().clone())
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())
        .unwrap();

    // let mut proxied_req = req;
    // *proxied_req.uri_mut() = proxied_uri.parse().unwrap();

    let response_result = sender.send_request(req).await?;
    let elapsed = start.elapsed();

    Ok((response_result, elapsed))
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
