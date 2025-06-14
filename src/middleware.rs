use super::router::{Handler, Middleware};
use futures::future::BoxFuture;
use hyper::{Body, Request, Response};
use std::sync::Arc;

/// Трейт для преобразования кортежей в итерируемую коллекцию middleware
pub trait IntoMiddlewares {
    type IntoIter: IntoIterator<Item = Middleware>;
    fn into_middlewares(self) -> Self::IntoIter;
}

impl<F> IntoMiddlewares for F
where
    F: Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
{
    type IntoIter = std::vec::IntoIter<Middleware>;
    fn into_middlewares(self) -> Self::IntoIter {
        vec![Arc::new(self) as Middleware].into_iter()
    }
}

impl<F1, F2> IntoMiddlewares for (F1, F2)
where
    F1: Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    F2: Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
{
    type IntoIter = std::vec::IntoIter<Middleware>;
    fn into_middlewares(self) -> Self::IntoIter {
        vec![
            Arc::new(self.0) as Middleware,
            Arc::new(self.1) as Middleware,
        ].into_iter()
    }
}

impl<F1, F2, F3> IntoMiddlewares for (F1, F2, F3)
where
    F1: Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    F2: Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
    F3: Fn(Request<Body>, Handler) -> BoxFuture<'static, Response<Body>> + Send + Sync + 'static,
{
    type IntoIter = std::vec::IntoIter<Middleware>;
    fn into_middlewares(self) -> Self::IntoIter {
        vec![
            Arc::new(self.0) as Middleware,
            Arc::new(self.1) as Middleware,
            Arc::new(self.2) as Middleware,
        ].into_iter()
    }
}
