# x302.cn 短链接服务（Rust 版）

原 Kotlin/Ktor 版短链接服务的 Rust 重写版本，行为与原服务保持兼容。

技术栈：
- 语言: Rust (edition 2024)
- Web: axum 0.8 + tower / tower-http
- 模板: minijinja（对应原 FreeMarker）
- ORM: sea-orm（SQLite / PostgreSQL）
- 配置: TOML

## 运行

```bash
# 从原 Kotlin 项目复制一份 SQLite 数据库用于本地测试
copy ..\x302.cn\x302.cn.sqlite.db .\x302.cn.sqlite.db

cargo run
```

服务默认监听 `http://0.0.0.0:8080`。

## 配置

见 [config.toml](config.toml)：

| 配置项 | 说明 |
| --- | --- |
| `server.host` / `server.port` | 监听地址与端口 |
| `db.url` | 数据库连接串（`sqlite://` 或 `postgres://`） |
| `action.delete.password` | 使用该密码可删除任意短链接；建议留空，改用环境变量提供 |

敏感配置建议通过环境变量提供，例如：

```bash
X302_DELETE_PASSWORD=你的密码 cargo run
```

## 接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/` | 首页；可选 `?url=` 生成短链，并展示最新 100 条 |
| GET | `/s/{key}` | 302 跳转到原始链接，并累加访问次数 |
| POST | `/s/{key}?action=delete[&password=...]` | 删除短链接；带正确密码删除任意，否则仅可删除本 IP 创建的 |
| GET | `/metrics-micrometer` | Prometheus 指标 |
