use std::sync::Arc;

use metrics_exporter_prometheus::PrometheusHandle;
use minijinja::Environment;

use crate::config::AppConfig;
use crate::service::short_url::ShortUrlService;

/// 应用共享状态
pub struct AppState {
    pub service: ShortUrlService,
    pub tmpl: Environment<'static>,
    pub config: AppConfig,
    pub metrics: PrometheusHandle,
}

pub type SharedState = Arc<AppState>;
