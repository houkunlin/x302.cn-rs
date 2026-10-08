//! 错误处理与 HTML 错误页，对应原 Ktor `StatusPages`。

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use sea_orm::DbErr;

/// 应用错误类型
#[derive(Debug)]
pub enum AppError {
    /// 404
    NotFound,
    /// 500（包含错误类型与错误信息）
    Internal {
        kind: String,
        message: String,
    },
}

impl AppError {
    pub fn internal(message: impl Into<String>) -> Self {
        AppError::Internal {
            kind: "InternalError".to_string(),
            message: message.into(),
        }
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::Internal {
            kind: "DatabaseError".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<minijinja::Error> for AppError {
    fn from(err: minijinja::Error) -> Self {
        AppError::Internal {
            kind: "TemplateError".to_string(),
            message: err.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, Html(not_found_html())).into_response(),
            AppError::Internal { kind, message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, Html(error_html(&kind, &message))).into_response()
            }
        }
    }
}

fn error_html(kind: &str, message: &str) -> String {
    format!(
        "<html><body>\
<h1>发现错误</h1>\
<p>错误类型：{kind}</p>\
<p>错误信息：{message}</p>\
<a href=\"/\">返回首页</a>\
</body></html>"
    )
}

fn not_found_html() -> &'static str {
    "<html><body>\
<h1>404</h1>\
<a href=\"/\">返回首页</a>\
</body></html>"
}

/// 未匹配路由的兜底处理器
pub async fn fallback() -> Response {
    (StatusCode::NOT_FOUND, Html(not_found_html())).into_response()
}
