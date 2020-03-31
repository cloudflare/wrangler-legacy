use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

pub struct Host;

impl Host {
    pub async fn run() -> Result<(), failure::Error> {
        // Start HTTP echo server that prints whatever is posted to it.
        let addr = ([127, 0, 0, 1], 8080).into();

        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(print_logs)) });

        let server = Server::bind(&addr).serve(service);

        server.await?;

        Ok(())
    }
}

async fn print_logs(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;
            println!(
                "{}",
                std::str::from_utf8(&whole_body).expect("failed to deserialize tail log body")
            );

            Ok(Response::new(Body::from("Success")))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
