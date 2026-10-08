use sea_orm::entity::prelude::*;

/// 短链接表 `short_url` 的实体映射（与原 Kotlin/Exposed 表结构保持一致）
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "short_url")]
pub struct Model {
    /// 主键ID（雪花ID，应用侧生成，非自增）
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// 短链接 KEY
    #[sea_orm(column_name = "url_key")]
    pub url_key: String,
    /// 原始链接
    #[sea_orm(column_name = "url_raw")]
    pub url_raw: String,
    /// 原始链接的 SHA256 值
    #[sea_orm(column_name = "url_sha256_hex")]
    pub url_sha256_hex: String,
    /// 短链接过期时间（当前未启用）
    #[sea_orm(column_name = "expired_time")]
    pub expired_time: String,
    /// 短链接被访问的次数
    #[sea_orm(column_name = "visit_num")]
    pub visit_num: i32,
    /// 创建IP
    #[sea_orm(column_name = "created_ip")]
    pub created_ip: String,
    /// 创建UA
    #[sea_orm(column_name = "created_ua")]
    pub created_ua: String,
    /// 创建时间
    #[sea_orm(column_name = "created_time")]
    pub created_time: String,
    /// 更新时间
    #[sea_orm(column_name = "updated_time")]
    pub updated_time: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
