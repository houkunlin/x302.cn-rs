//! Prometheus 指标，对应原 Micrometer 注册表与 `/metrics-micrometer`。

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

/// 安装 Prometheus recorder 并返回渲染句柄
pub fn install() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("install prometheus recorder")
}
