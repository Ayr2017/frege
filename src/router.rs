use hyper::{Body, Request, Response, Method, StatusCode};
use futures::future::BoxFuture;
use std::sync::Arc;
use crate::middleware::IntoMiddlewares;

// Тип для обработчиков
pub type Handler = Arc<dyn Fn(Request<Body>) -> BoxFuture<'static, Response<Body>> + Send + Sync>;
pub type Middleware = Arc<dyn Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync>;

#[derive(Clone)]
struct Route {
    method: Method,
    path: String,
    handler: Handler,
    middlewares: Vec<Middleware>,
}

/// Основной роутер
#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn get<F>(&mut self, path: &str, handler: F) -> RouteBuilder
    where
        F: Fn(Request<Body>) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    {
        RouteBuilder::new(self, Method::GET, path, handler)
    }

    pub fn post<F>(&mut self, path: &str, handler: F) -> RouteBuilder
    where
        F: Fn(Request<Body>) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    {
        RouteBuilder::new(self, Method::POST, path, handler)
    }

    pub fn put<F>(&mut self, path: &str, handler: F) -> RouteBuilder
    where
        F: Fn(Request<Body>) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    {
        RouteBuilder::new(self, Method::PUT, path, handler)
    }

    pub fn delete<F>(&mut self, path: &str, handler: F) -> RouteBuilder
    where
        F: Fn(Request<Body>) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    {
        RouteBuilder::new(self, Method::DELETE, path, handler)
    }

    /// Обрабатывает входящий запрос
    pub async fn handle(&self, req: Request<Body>) -> Response<Body> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        if let Some(route) = self.routes.iter().find(|r| r.method == method && r.path == path) {
            let mut handler = route.handler.clone();
            for mw in route.middlewares.iter().rev() {
                let next = handler.clone();
                let mw = mw.clone();
                handler = Arc::new(move |req| {
                    let next = next.clone();
                    let mw = mw.clone();
                    Box::pin(async move {
                        mw(req, next).await
                    })
                });
            }
            handler(req).await
        } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    }
}

/// Билдер для маршрута
pub struct RouteBuilder<'a> {
    router: &'a mut Router,
    route: Route,
    committed: bool,
}

impl<'a> RouteBuilder<'a> {
    fn new<F>(router: &'a mut Router, method: Method, path: &str, handler: F) -> Self
    where
        F: Fn(Request<Body>) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    {
        Self {
            router,
            route: Route {
                method,
                path: path.to_string(),
                handler: Arc::new(handler),
                middlewares: Vec::new(),
            },
            committed: false,
        }
    }

    /// Добавляет middleware к маршруту
    pub fn middlewares<M: IntoMiddlewares>(mut self, middlewares: M) -> Self {
        self.route.middlewares = middlewares.into_middlewares().into_iter().collect();
        self
    }

    /// Явно регистрирует маршрут (альтернатива Drop)
    pub fn register(mut self) {
        if !self.committed {
            self.router.routes.push(self.route.clone());
            self.committed = true;
        }
    }
}

impl<'a> Drop for RouteBuilder<'a> {
    fn drop(&mut self) {
        if !self.committed {
            self.router.routes.push(self.route.clone());
            self.committed = true;
        }
    }
}
