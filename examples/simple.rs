use frege::{Handler, Router};
use futures::future::BoxFuture;
use hyper::server::Server;
use hyper::{Body, Request, Response, StatusCode};

fn logging_middleware(req: Request<Body>, next: Handler) -> BoxFuture<'static, Response<Body>> {
    Box::pin(async move {
        println!("Request: {} {}", req.method(), req.uri().path());
        next(req).await
    })
}

fn hello_handler(_req: Request<Body>) -> BoxFuture<'static, Response<Body>> {
    Box::pin(async move {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Hello, world!"))
            .unwrap()
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();
    router
        .get("/hello", hello_handler)
        .middlewares(logging_middleware);

    let addr = ([127, 0, 0, 1], 3000).into();
    Server::bind(&addr)
        .serve(hyper::service::make_service_fn(|_| {
            let router = router.clone();
            async move {
                Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                    let router = router.clone();
                    async move { Ok::<_, hyper::Error>(router.handle(req).await) }
                }))
            }
        }))
        .await?;

    Ok(())
}
