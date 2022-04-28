use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

async fn hello(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{:#?}", req);
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.status_mut() = StatusCode::OK;
            *response.body_mut() = Body::from("Hello World!");
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Ok(response)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(hello)) });
    let addr = ([0, 0, 0, 0], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://{}", addr);
    server
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("");
        })
        .await?;
    Ok(())
}

wit_bindgen_wasmtime::import!("wit/http.wit");

impl From<http::Method> for Method {
    fn from(method: http::Method) -> Self {
        use http::Method as M;
        match method {
            M::Delete => Method::DELETE,
            M::Get => Method::GET,
            M::Head => Method::HEAD,
            M::Post => Method::POST,
            M::Put => Method::PUT,
            M::Patch => Method::PATCH,
            M::Options => Method::OPTIONS,
        }
    }
}
