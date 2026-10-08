//! HTTP 中间件装配，对应原 Ktor `configureHTTP` 与 `configureMonitoring`。

use std::time::Instant;

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, Method, header};
use axum::middleware::{Next, from_fn};
use axum::response::Response;
use axum::Router;
use metrics::{counter, histogram};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::Span;

const X_ENGINE: &str = "x302-rs";
const HSTS_VALUE: &str = "max-age=31536000; includeSubDomains";

fn x_request_id() -> HeaderName {
    HeaderName::from_static("x-request-id")
}

/// 为请求创建带 call-id 的追踪 Span
#[derive(Clone, Copy)]
struct MakeSpanWithId;

impl<B> MakeSpan<B> for MakeSpanWithId {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> Span {
        let call_id = request
            .headers()
            .get(x_request_id())
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        tracing::info_span!(
            "request",
            "call-id" = %call_id,
            method = %request.method(),
            uri = %request.uri()
        )
    }
}

/// 应用所有中间件层
pub fn apply(app: Router) -> Router {
    app
        // 最内层：指标统计
        .layer(from_fn(track_metrics))
        // 静态资源缓存头
        .layer(from_fn(caching_headers))
        // 默认响应头
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-engine"),
            HeaderValue::from_static(X_ENGINE),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static(HSTS_VALUE),
        ))
        // 压缩
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        // CORS
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::OPTIONS,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                ])
                .allow_headers(Any),
        )
        // 请求 ID 回写响应
        .layer(PropagateRequestIdLayer::new(x_request_id()))
        // 追踪（在设置请求 ID 之后，便于记录 call-id）
        .layer(TraceLayer::new_for_http().make_span_with(MakeSpanWithId))
        // 最外层：生成/读取请求 ID
        .layer(SetRequestIdLayer::new(x_request_id(), MakeRequestUuid))
}

/// 记录请求计数与耗时
async fn track_metrics(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed().as_secs_f64();

    counter!(
        "x302_http_requests_total",
        "status" => response.status().as_u16().to_string()
    )
    .increment(1);
    histogram!("x302_http_request_duration_seconds").record(elapsed);

    response
}

/// 为 CSS / JS 响应设置 24 小时缓存
async fn caching_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let is_static = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|content_type| {
            let content_type = content_type.to_ascii_lowercase();
            content_type.starts_with("text/css")
                || content_type.starts_with("application/javascript")
                || content_type.starts_with("text/javascript")
        })
        .unwrap_or(false);

    if is_static {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("max-age=86400"),
        );
    }

    response
}
