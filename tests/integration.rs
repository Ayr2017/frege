use frege::{Router, Resource, Handler};
use hyper::{Body, Request, Response, StatusCode, Method};
use hyper::server::Server;
use hyper::client::Client;
use async_trait::async_trait;
use tokio::net::TcpListener;
use futures::future::BoxFuture;

struct TestResource;

#[async_trait]
impl Resource for TestResource {
    async fn get(&self, id: i32) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(format!("User {}", id)))
            .unwrap()
    }
    async fn create(&self, _data: String) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Created"))
            .unwrap()
    }
    async fn update(&self, _id: i32, _data: String) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Updated"))
            .unwrap()
    }
    async fn delete(&self, _id: i32) -> Response<Body> {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Deleted"))
            .unwrap()
    }
}

fn logging_middleware(req: Request<Body>, next: Handler) -> BoxFuture<'static, Response<Body>> {
    Box::pin(async move {
        next(req).await
    })
}

fn hello_handler(_req: Request<Body>) -> BoxFuture<'static, Response<Body>> {
    Box::pin(async move {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Hello"))
            .unwrap()
    })
}

#[tokio::test]
async fn test_router() {
    let mut router = Router::new();
    router.get("/hello", hello_handler)
        .middlewares(logging_middleware);
    router.resource("/users", TestResource);

    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = Server::bind(&addr)
        .serve(hyper::service::make_service_fn(move |_| {
            let router = router.clone();
            async move {
                Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                    let router = router.clone();
                    async move { Ok::<_, hyper::Error>(router.handle(req).await) }
                }))
            }
        }));

    tokio::spawn(server);
    let client = Client::new();

    let resp = client.get(format!("http://{}/hello", addr).parse().unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body, "Hello");

    let resp = client.get(format!("http://{}/users/1", addr).parse().unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body, "User 1");

    // PUT
    let req = Request::builder()
        .method(Method::PUT)
        .uri(format!("http://{}/users/1", addr))
        .body(Body::from("new data"))
        .unwrap();
    let resp = client.request(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body, "Updated");

    // DELETE
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("http://{}/users/1", addr))
        .body(Body::empty())
        .unwrap();
    let resp = client.request(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body, "Deleted");
}
