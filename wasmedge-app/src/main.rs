use bytecodec::{
    bytes::{BytesEncoder, RemainingBytesDecoder},
    io::IoEncodeExt,
    DecodeExt, Encode,
};
use httparse::Status;
use httpcodec::{
    BodyEncoder, HttpVersion, ReasonPhrase, Request, RequestDecoder, Response, ResponseEncoder,
    StatusCode,
};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn file_server(path: &str) -> Option<Vec<u8>> {
    let mut target = PathBuf::from(format!(".{}", urlencoding::decode(path).unwrap()));
    println!("target: {:?}", target);
    if !target.exists() {
        println!("does not exists");
        None
    } else {
        if target.is_dir() {
            target.push("index.html");
        }

        println!("target: {:?}", target);
        if target.is_file() {
            let mut f = File::open(target.as_path()).unwrap();
            let mut buf = vec![];
            f.read_to_end(&mut buf).unwrap();
            Some(buf)
        } else {
            None
        }
    }
}

fn handle_http(req: Request<Vec<u8>>) -> Response<Vec<u8>> {
    match req.method().as_str() {
        "GET" => match file_server(req.request_target().as_str()) {
            Some(v) => Response::new(
                req.http_version(),
                StatusCode::new(200).unwrap(),
                ReasonPhrase::new("").unwrap(),
                v,
            ),
            None => Response::new(
                req.http_version(),
                StatusCode::new(404).unwrap(),
                ReasonPhrase::new("NOT FOUND").unwrap(),
                b"404 Not Found".to_vec(),
            ),
        },
        _ => Response::new(
            req.http_version(),
            StatusCode::new(500).unwrap(),
            ReasonPhrase::new("NOT ALLOWED HTTP METHOD").unwrap(),
            Vec::with_capacity(0),
        ),
    }
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
                        break;
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
        Ok(req) => Ok(handle_http(req)),
        Err(e) => Err(e),
    };

    let r = match req {
        Ok(r) => r,
        Err(e) => {
            let err = format!("{:?}", e);
            Response::new(
                HttpVersion::V1_1,
                StatusCode::new(500).unwrap(),
                ReasonPhrase::new(&e.to_string()).unwrap(),
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
