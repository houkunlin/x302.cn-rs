//! MiniJinja 模板环境与渲染上下文（对应原 FreeMarker 模板）。

use minijinja::{Environment, Error as MiniError};
use serde::Serialize;

use crate::db::entity::short_url::Model;
use crate::service::short_url::format_display_time;

const INDEX_TEMPLATE: &str = "index.html";

/// 构建模板环境（编译期嵌入模板）
pub fn build() -> Result<Environment<'static>, MiniError> {
    let mut env = Environment::new();
    env.add_template(INDEX_TEMPLATE, include_str!("../../templates/index.html"))?;
    Ok(env)
}

/// 列表行视图
#[derive(Debug, Serialize)]
pub struct UrlRow {
    pub created_time: String,
    pub url_key: String,
    pub url_raw: String,
    pub visit_num: i32,
    pub created_ip: String,
}

impl From<&Model> for UrlRow {
    fn from(model: &Model) -> Self {
        Self {
            created_time: format_display_time(&model.created_time),
            url_key: model.url_key.clone(),
            url_raw: model.url_raw.clone(),
            visit_num: model.visit_num,
            created_ip: model.created_ip.clone(),
        }
    }
}

/// 新创建短链接的视图
#[derive(Debug, Serialize)]
pub struct VoView {
    pub url_key: String,
    pub url_raw: String,
}

impl From<&Model> for VoView {
    fn from(model: &Model) -> Self {
        Self {
            url_key: model.url_key.clone(),
            url_raw: model.url_raw.clone(),
        }
    }
}

/// 首页渲染上下文
#[derive(Debug, Serialize)]
pub struct IndexContext {
    pub data: Vec<UrlRow>,
    pub vo: Option<VoView>,
    pub url: String,
    pub host: String,
    pub jump_url: String,
    pub current_ip: String,
}
