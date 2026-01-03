use std::convert::Infallible;
use std::task::{Context, Poll};

use axum::Router;
use tower::Service;
use tower_http::services::ServeDir;

use crate::config::AppConfig;

fn create_error_response(status: axum::http::StatusCode) -> axum::http::Response<axum::body::Body> {
  axum::http::Response::builder()
    .status(status)
    .body(axum::body::Body::empty())
    .unwrap_or_else(|e| {
      log::error!("Failed to create error response: {:?}", e);
      axum::http::Response::new(axum::body::Body::empty())
    })
}

#[derive(Clone)]
struct HtmlFallbackService {
  inner: ServeDir,
}

impl Service<axum::http::Request<axum::body::Body>> for HtmlFallbackService {
  type Response = axum::http::Response<axum::body::Body>;
  type Error = Infallible;
  type Future = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
  >;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    match <ServeDir as Service<axum::http::Request<axum::body::Body>>>::poll_ready(
      &mut self.inner,
      cx,
    ) {
      Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
      Poll::Ready(Err(_e)) => Poll::Ready(Ok(())), // Convert IO errors to 500 responses
      Poll::Pending => Poll::Pending,
    }
  }

  fn call(&mut self, request: axum::http::Request<axum::body::Body>) -> Self::Future {
    let path = request.uri().path().to_string();
    let mut inner = self.inner.clone();

    Box::pin(async move {
      let response =
        match <ServeDir as tower::ServiceExt<axum::http::Request<axum::body::Body>>>::ready(
          &mut inner,
        )
        .await
        {
          Ok(s) => match s.call(request).await {
            Ok(r) => r,
            Err(e) => {
              tracing::error!("Error serving static file: {:?}", e);
              return Ok(create_error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
              ));
            }
          },
          Err(e) => {
            tracing::error!("Error preparing static file service: {:?}", e);
            return Ok(create_error_response(
              axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
          }
        };

      if response.status() == axum::http::StatusCode::NOT_FOUND
        && !path.contains('.')
        && !path.ends_with('/')
      {
        let html_path = format!("{}.html", path);
        if let Ok(new_uri) = html_path.parse::<axum::http::Uri>() {
          match axum::http::Request::builder()
            .uri(new_uri)
            .method(axum::http::Method::GET)
            .body(axum::body::Body::empty())
          {
            Ok(html_request) => {
              let mut inner = inner.clone();
              match <ServeDir as tower::ServiceExt<axum::http::Request<axum::body::Body>>>::ready(
                &mut inner,
              )
              .await
              {
                Ok(s) => match s.call(html_request).await {
                  Ok(html_response) => {
                    return Ok(html_response.map(axum::body::Body::new));
                  }
                  Err(e) => {
                    tracing::warn!("Error serving HTML fallback: {:?}", e);
                  }
                },
                Err(e) => {
                  tracing::warn!("Error preparing HTML fallback service: {:?}", e);
                }
              }
            }
            Err(e) => {
              tracing::warn!(
                "Failed to build HTML fallback request for {}: {:?}",
                html_path,
                e
              );
            }
          }
        }

        if let Ok(index_uri) = "/index.html".parse::<axum::http::Uri>() {
          match axum::http::Request::builder()
            .uri(index_uri)
            .method(axum::http::Method::GET)
            .body(axum::body::Body::empty())
          {
            Ok(index_request) => {
              let mut inner = inner.clone();
              match <ServeDir as tower::ServiceExt<axum::http::Request<axum::body::Body>>>::ready(
                &mut inner,
              )
              .await
              {
                Ok(s) => match s.call(index_request).await {
                  Ok(index_response) if index_response.status().is_success() => {
                    return Ok(index_response.map(axum::body::Body::new));
                  }
                  Ok(_) => {}
                  Err(e) => tracing::warn!("Error serving index fallback: {:?}", e),
                },
                Err(e) => tracing::warn!("Error preparing index fallback service: {:?}", e),
              }
            }
            Err(e) => tracing::warn!("Failed to build index fallback request: {:?}", e),
          }
        }
      }

      Ok(response.map(axum::body::Body::new))
    })
  }
}

pub fn create_router(config: &AppConfig, prefix_optional: Option<&str>) -> Router {
  let serve_dir = ServeDir::new(&config.static_dir)
    .append_index_html_on_directories(true)
    .precompressed_gzip()
    .precompressed_br()
    .precompressed_deflate()
    .precompressed_zstd();

  let fallback_service = HtmlFallbackService { inner: serve_dir };

  if let Some(prefix) = prefix_optional {
    Router::new().nest_service(prefix, fallback_service)
  } else {
    Router::new().fallback_service(fallback_service)
  }
}
