use std::path::Path;

use serde::Deserialize;

/// 应用配置，对应原 Kotlin 项目的 application.yaml
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DbConfig,
    pub action: ActionConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionConfig {
    pub delete: DeleteConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteConfig {
    #[serde(default)]
    pub password: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

impl AppConfig {
    /// 从指定路径读取 TOML 配置；路径为空时使用环境变量 X302_CONFIG 或默认 config.toml
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => std::env::var("X302_CONFIG")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("config.toml")),
        };
        let text = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError(format!("读取配置文件 {} 失败: {e}", path.display())))?;
        let mut config = Self::parse(&text)?;
        config.apply_env_overrides();
        Ok(config)
    }

    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        toml::from_str(text).map_err(|e| ConfigError(format!("解析配置失败: {e}")))
    }

    /// 使用环境变量覆盖敏感/环境相关配置，避免将真实值提交到仓库
    pub fn apply_env_overrides(&mut self) {
        if let Ok(password) = std::env::var("X302_DELETE_PASSWORD") {
            self.action.delete.password = password;
        }
    }
}

#[derive(Debug)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}
