use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use std::{convert::Infallible, net::SocketAddr};

async fn mirror(req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, hyper::Error> {
    let ip = addr.to_string();
    let mut output = format!(
        "{}\n{}\n\r{} {} {}\n",
        ip,
        std::iter::repeat("-").take(ip.len()).collect::<String>(),
        req.method(),
        if let Some(r) = req.uri().path_and_query() {
            r.to_string()
        } else {
            "".to_string()
        },
        match req.version() {
            hyper::Version::HTTP_09 => "HTTP/0.9",
            hyper::Version::HTTP_10 => "HTTP/1.0",
            hyper::Version::HTTP_11 => "HTTP/1.1",
            hyper::Version::HTTP_2 => "HTTP/2.0",
            hyper::Version::HTTP_3 => "HTTP/3.0",
            _ => "Invalid",
        }
    );
    for (key, value) in req.headers() {
        output.push_str(&format!(
            "{}: {}\r\n",
            key,
            match value.to_str() {
                Ok(value) => value,
                Err(_) => "",
            }
        ));
    }
    let body = hyper::body::to_bytes(req).await;
    match body {
        Ok(d) => output.push_str(&format!(
            "\r\n{}\r\n",
            if let Ok(d) = std::str::from_utf8(&d) {
                d
            } else {
                ""
            }
        )),
        _ => (),
    };
    Ok(Response::new(Body::from(format!("{}", output))))
}
#[tokio::main]
async fn main() {
    let addr = ([0, 0, 0, 0], 3000).into();
    use hyper::server::conn::AddrStream;

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let addr = conn.remote_addr();
        async move {
            let addr = addr.clone();
            Ok::<_, Infallible>(service_fn(move |req| mirror(req, addr.clone())))
        }
    });

    let server = Server::bind(&addr).serve(make_service);
    println!("server running on {}", addr);
    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
