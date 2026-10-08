use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{FixedOffset, Utc};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use x302_cn::config::AppConfig;
use x302_cn::db;
use x302_cn::service::short_url::ShortUrlService;
use x302_cn::web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = AppConfig::load(None)?;

    let db = db::connect(&config.db.url).await?;
    db::ensure_schema(&db).await?;

    let tmpl = web::templates::build()?;
    let metrics = web::metrics::install();
    let service = ShortUrlService::new(db);

    let state = Arc::new(web::state::AppState {
        service,
        tmpl,
        config: config.clone(),
        metrics,
    });

    let app = web::middleware::apply(web::routes::router().with_state(state));

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("x302.cn 短链接服务监听 http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_timer(ChinaTimer))
        .init();
}

/// 以中国时区（UTC+8）格式化日志时间，格式与原 logback 配置一致
struct ChinaTimer;

impl tracing_subscriber::fmt::time::FormatTime for ChinaTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> fmt::Result {
        let offset = FixedOffset::east_opt(8 * 3600).expect("valid +08:00 offset");
        let now = Utc::now().with_timezone(&offset);
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}
