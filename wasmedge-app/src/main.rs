use bytecodec::bytes::BytesEncoder;
use bytecodec::bytes::RemainingBytesDecoder;
use bytecodec::io::IoEncodeExt;
use bytecodec::DecodeExt;
use bytecodec::Encode;
use httparse::Status;
use httpcodec::BodyEncoder;
use httpcodec::{
    HttpVersion, ReasonPhrase, Request, RequestDecoder, Response, ResponseEncoder, StatusCode,
};
use std::io::{Read, Write};
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn handle_http(req: Request<Vec<u8>>) -> bytecodec::Result<Response<Vec<u8>>> {
    Ok(Response::new(
        req.http_version(),
        StatusCode::new(200)?,
        ReasonPhrase::new("")?,
        req.body().clone(),
    ))
}

/// streamからHTTPリクエストのヘッダーとボディの全てを受けとって返す
fn recv_all(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buff = [0u8; 1024];
    let mut data = Vec::new();

    loop {
        let n = stream.read(&mut buff)?;
        data.extend_from_slice(&buff[..n]);

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(&data).unwrap() {
            Status::Complete(body_offset) => {
                let content_length: usize =
                    if let Some(h) = req.headers.iter().find(|h| h.name.eq("Content-Length")) {
                        std::str::from_utf8(h.value).unwrap().parse().unwrap()
                    } else {
                        panic!("cannot find header `Content-Length`");
                    };

                if content_length == data.len() - body_offset {
                    break;
                }
            }
            Status::Partial => {
                println!("partial");
            }
        }
    }

    Ok(data)
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let data = recv_all(&mut stream)?;
    let mut decoder = RequestDecoder::<httpcodec::BodyDecoder<RemainingBytesDecoder>>::default();

    let req = match decoder.decode_from_bytes(data.as_slice()) {
        Ok(req) => handle_http(req),
        Err(e) => Err(e),
    };

    let r = match req {
        Ok(r) => r,
        Err(e) => {
            let err = format!("{:?}", e);
            Response::new(
                HttpVersion::V1_1,
                StatusCode::new(500).unwrap(),
                ReasonPhrase::new(err.clone().as_str()).unwrap(),
                err.into_bytes(),
            )
        }
    };

    let mut encoder = ResponseEncoder::new(BodyEncoder::new(BytesEncoder::new()));
    encoder.start_encoding(r).unwrap();

    let mut data = Vec::new();
    encoder.encode_all(&mut data).unwrap();

    stream.write(&data)?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1234".to_string());
    println!("new connection at {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port), false)?;
    loop {
        let _ = handle_client(listener.accept(false)?.0);
    }
}
