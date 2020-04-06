use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::sync::oneshot::Receiver;

pub struct Host {
    server: Builder<AddrIncoming>,
    rx: Receiver<()>,
}

impl Host {
    pub fn new(rx: Receiver<()>) -> Host {
        // Start HTTP echo server that prints whatever is posted to it.
        let addr = ([127, 0, 0, 1], 8080).into();

        let server = Server::bind(&addr);

        Host { server, rx }
    }

    pub async fn run(self) -> Result<(), failure::Error> {
        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(print_logs)) });

        let server = self.server.serve(service);

        let rx = self.rx;

        let graceful = server.with_graceful_shutdown(async {
            rx.await.ok();
        });

        graceful.await?;

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
