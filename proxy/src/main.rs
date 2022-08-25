use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::net::IpAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};

fn debug_request(_req: Request<Body>, n: u64) -> Result<Response<Body>, Infallible> {
    // let body_str = format!("{:?}", req);
    Ok(Response::new(Body::from(n.to_string())))
}

async fn handle(
    client_ip: IpAddr,
    req: Request<Body>,
    counter: Arc<AtomicU64>,
) -> Result<Response<Body>, Infallible> {
    if req.uri().path().starts_with("/target/first") {
        // will forward requests to port 13901
        match hyper_reverse_proxy::call(client_ip, "http://127.0.0.1:13901", req).await {
            Ok(response) => Ok(response),
            Err(_error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()),
        }
    } else if req.uri().path().starts_with("/target/second") {
        // will forward requests to port 13902
        match hyper_reverse_proxy::call(client_ip, "http://127.0.0.1:13902", req).await {
            Ok(response) => Ok(response),
            Err(_error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()),
        }
    } else {
        let count = counter.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        debug_request(req, count)
    }
}

#[tokio::main]
async fn main() {
    let bind_addr = "127.0.0.1:8000";
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let counter = Arc::new(AtomicU64::new(0));

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        let counter = counter.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle(remote_addr, req, counter.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
