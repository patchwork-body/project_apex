use matchit::{Match, Router};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

type Handler = Box<
    dyn Fn(HashMap<String, String>) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync,
>;

pub struct ApexRouter {
    router: Router<Handler>,
}

impl ApexRouter {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    pub fn route<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let boxed_handler: Handler = Box::new(move |params| Box::pin(handler(params)));

        if let Err(e) = self.router.insert(path, boxed_handler) {
            panic!("Failed to insert route '{path}': {e}");
        }

        self
    }

    pub async fn handle_request(&self, path: &str) -> Option<String> {
        match self.router.at(path) {
            Ok(Match {
                value: handler,
                params,
            }) => {
                let params_map: HashMap<String, String> = params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                Some(handler(params_map).await)
            }
            Err(_) => None,
        }
    }
}

impl Default for ApexRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_routing() {
        let router = ApexRouter::new()
            .route("/", |_| async { "Home".to_string() })
            .route("/about", |_| async { "About".to_string() });

        assert_eq!(router.handle_request("/").await, Some("Home".to_string()));
        assert_eq!(
            router.handle_request("/about").await,
            Some("About".to_string())
        );
        assert_eq!(router.handle_request("/missing").await, None);
    }

    #[tokio::test]
    async fn test_router_builder_pattern() {
        async fn root_handler(_params: HashMap<String, String>) -> String {
            "Root page".to_string()
        }

        async fn about_handler(_params: HashMap<String, String>) -> String {
            "About page".to_string()
        }

        let router = ApexRouter::new()
            .route("/", root_handler)
            .route("/about", about_handler);

        assert_eq!(
            router.handle_request("/").await,
            Some("Root page".to_string())
        );
        assert_eq!(
            router.handle_request("/about").await,
            Some("About page".to_string())
        );
    }

    #[tokio::test]
    async fn test_path_parameters() {
        let router = ApexRouter::new()
            .route("/users/{id}", |params| async move {
                format!(
                    "User ID: {}",
                    params.get("id").unwrap_or(&"unknown".to_string())
                )
            })
            .route("/posts/{id}/comments/{comment_id}", |params| async move {
                format!(
                    "Post ID: {}, Comment ID: {}",
                    params.get("id").unwrap_or(&"unknown".to_string()),
                    params.get("comment_id").unwrap_or(&"unknown".to_string())
                )
            });

        assert_eq!(
            router.handle_request("/users/123").await,
            Some("User ID: 123".to_string())
        );

        assert_eq!(
            router.handle_request("/posts/456/comments/789").await,
            Some("Post ID: 456, Comment ID: 789".to_string())
        );

        assert_eq!(router.handle_request("/users").await, None);
    }

    #[tokio::test]
    async fn test_wildcard_routes() {
        let router = ApexRouter::new().route("/static/{*filepath}", |params| async move {
            format!(
                "Static file: {}",
                params.get("filepath").unwrap_or(&"index.html".to_string())
            )
        });

        assert_eq!(
            router.handle_request("/static/css/style.css").await,
            Some("Static file: css/style.css".to_string())
        );

        assert_eq!(
            router.handle_request("/static/js/app.js").await,
            Some("Static file: js/app.js".to_string())
        );
    }
}
