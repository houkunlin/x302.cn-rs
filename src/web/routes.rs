//! 路由定义，对应原 Ktor `configureRouting`。

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use url::Url;

use crate::util::ip::request_ip;
use crate::web::errors::AppError;
use crate::web::state::SharedState;
use crate::web::templates::{IndexContext, UrlRow, VoView};

/// 首页查询参数
#[derive(Debug, Deserialize)]
struct IndexParams {
    url: Option<String>,
}

/// 删除接口查询参数
#[derive(Debug, Deserialize)]
struct DeleteParams {
    action: Option<String>,
    password: Option<String>,
}

/// 构建路由
pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(index))
        .route("/s/{key}", get(redirect_short_url).post(delete_short_url))
        .route("/metrics-micrometer", get(metrics))
        .fallback(crate::web::errors::fallback)
}

/// GET / 首页；可选 url 参数生成短链
async fn index(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<IndexParams>,
) -> Result<Html<String>, AppError> {
    let params_url = params.url.unwrap_or_default();
    let url_obj = if params_url.trim().is_empty() {
        None
    } else {
        Some(
            Url::parse(&params_url)
                .map_err(|_| AppError::internal("URL解析错误，请传入正确的URL"))?,
        )
    };

    let current_ip = request_ip(&headers, Some(addr));
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let vo = match &url_obj {
        Some(url) => Some(state.service.create(url.as_str(), &current_ip, &ua).await?),
        None => None,
    };

    let host = base_url(&headers);
    let jump_url = format!("{host}s/");

    let rows = state.service.latest_list(100).await?;
    let data: Vec<UrlRow> = rows.iter().map(UrlRow::from).collect();

    let context = IndexContext {
        data,
        vo: vo.as_ref().map(VoView::from),
        url: params_url,
        host,
        jump_url,
        current_ip,
    };

    let rendered = state
        .tmpl
        .get_template("index.html")?
        .render(context)?;

    Ok(Html(rendered))
}

/// GET /s/{key} 跳转并累计访问次数
async fn redirect_short_url(
    State(state): State<SharedState>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    match state.service.get_by_url_key(&key).await? {
        Some(model) => {
            state.service.visit(model.id).await?;
            let location = HeaderValue::from_str(&model.url_raw)
                .map_err(|_| AppError::internal("原始链接无法作为跳转地址"))?;
            let mut response = StatusCode::FOUND.into_response();
            response.headers_mut().insert(header::LOCATION, location);
            Ok(response)
        }
        None => Err(AppError::NotFound),
    }
}

/// POST /s/{key}?action=delete[&password=...] 删除短链接
async fn delete_short_url(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(key): Path<String>,
    Query(params): Query<DeleteParams>,
) -> Result<StatusCode, AppError> {
    if params.action.as_deref() != Some("delete") {
        return Ok(StatusCode::OK);
    }

    let password = state.config.action.delete.password.as_str();
    if !password.is_empty() && params.password.as_deref() == Some(password) {
        state.service.delete(&key).await?;
    } else {
        let current_ip = request_ip(&headers, Some(addr));
        state.service.delete_by_ip(&key, &current_ip).await?;
    }

    Ok(StatusCode::OK)
}

/// GET /metrics-micrometer 返回 Prometheus 指标
async fn metrics(State(state): State<SharedState>) -> Response {
    let body = state.metrics.render();
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4"),
        )],
        body,
    )
        .into_response()
}

/// 计算服务基础地址 scheme://host/
fn base_url(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    format!("{scheme}://{host}/")
}
