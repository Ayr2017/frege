use async_trait::async_trait;
use hyper::{Body, Request, Response};
use futures::future::BoxFuture;
use super::router::{Router, Handler};
use std::sync::Arc;

/// Трейт для CRUD-ресурсов
#[async_trait]
pub trait Resource: Send + Sync + 'static {
    async fn get(&self, id: i32) -> Response<Body>;
    async fn create(&self, data: String) -> Response<Body>;
    async fn update(&self, id: i32, data: String) -> Response<Body>;
    async fn delete(&self, id: i32) -> Response<Body>;
}

impl Router {
    /// Регистрирует CRUD-ресурс
    pub fn resource<R: Resource>(&mut self, path: &str, resource: R) -> &mut Self {
        let resource = Arc::new(resource);

        // GET /path/:id
        self.get(&format!("{}/:id", path), {
            let resource = resource.clone();
            move |req| {
                let resource = resource.clone();
                Box::pin(async move {
                    let id = req.uri().path().split('/').last().unwrap().parse().unwrap();
                    resource.get(id).await
                })
            }
        });

        // POST /path
        self.post(path, {
            let resource = resource.clone();
            move |req| {
                let resource = resource.clone();
                Box::pin(async move {
                    let data = extract_body(req).await.unwrap_or_default();
                    resource.create(data).await
                })
            }
        });

        // PUT /path/:id
        self.put(&format!("{}/:id", path), {
            let resource = resource.clone();
            move |req| {
                let resource = resource.clone();
                Box::pin(async move {
                    let id = req.uri().path().split('/').last().unwrap().parse().unwrap();
                    let data = extract_body(req).await.unwrap_or_default();
                    resource.update(id, data).await
                })
            }
        });

        // DELETE /path/:id
        self.delete(&format!("{}/:id", path), {
            let resource = resource.clone();
            move |req| {
                let resource = resource.clone();
                Box::pin(async move {
                    let id = req.uri().path().split('/').last().unwrap().parse().unwrap();
                    resource.delete(id).await
                })
            }
        });

        self
    }
}

/// Вспомогательная функция для извлечения тела запроса
async fn extract_body(req: Request<Body>) -> Result<String, hyper::Error> {
    let bytes = hyper::body::to_bytes(req.into_body()).await?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}
