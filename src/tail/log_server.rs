use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::sync::oneshot::Receiver;

pub struct LogServer {
    server: Builder<AddrIncoming>,
    shutdown_rx: Receiver<()>,
}

/// LogServer is just a basic HTTP server running locally; it listens for POST requests on the root
/// path and simply prints the JSON body of each request as its own line to STDOUT.
impl LogServer {
    pub fn new(shutdown_rx: Receiver<()>) -> LogServer {
        // Start HTTP echo server that prints whatever is posted to it.
        let addr = ([127, 0, 0, 1], 8080).into();

        let server = Server::bind(&addr);

        LogServer {
            server,
            shutdown_rx,
        }
    }

    pub async fn run(self) -> Result<(), failure::Error> {
        let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(print_logs)) });

        let server = self.server.serve(service);

        // The shutdown receiver listens for a one shot message from our sigint handler as a signal
        // to gracefully shut down the hyper server.
        let shutdown_rx = self.shutdown_rx;

        let graceful = server.with_graceful_shutdown(async {
            shutdown_rx.await.ok();
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
