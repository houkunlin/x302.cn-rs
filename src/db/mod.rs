pub mod entity;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Schema};

use crate::db::entity::short_url::Entity as ShortUrl;

/// 建立数据库连接
pub async fn connect(url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(url).await
}

/// 确保 short_url 表存在。表已存在时忽略错误（幂等）。
pub async fn ensure_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let stmt = schema.create_table_from_entity(ShortUrl);
    match db.execute(backend.build(&stmt)).await {
        Ok(_) => Ok(()),
        Err(err) => {
            // 表已存在（SQLite 无 IF NOT EXISTS 支持路径）时忽略
            let already_exists = matches!(backend, DbBackend::Sqlite)
                && err.to_string().to_lowercase().contains("already exists");
            if already_exists {
                Ok(())
            } else {
                Err(err)
            }
        }
    }
}
