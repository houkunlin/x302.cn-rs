//! 短链接业务逻辑，对应原 Kotlin `ShortUrlService`。

use std::collections::HashSet;

use chrono::{Duration, Local};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::db::entity::short_url::{ActiveModel, Column, Entity as ShortUrl, Model};
use crate::util::snowflake;
use crate::util::url_key;

/// URL 最大长度
pub const URL_MAX_LENGTH: usize = 8192;

/// 时间列格式（SQLite TEXT 存储，保留毫秒）
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// 短链接 Service
pub struct ShortUrlService {
    db: DatabaseConnection,
}

impl ShortUrlService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 获取最后几条数据
    pub async fn latest_list(&self, limit: u64) -> Result<Vec<Model>, DbErr> {
        ShortUrl::find()
            .order_by_desc(Column::Id)
            .limit(limit)
            .all(&self.db)
            .await
    }

    /// 创建一个短链接；若对应的原始链接已存在则直接返回已有记录
    pub async fn create(&self, url: &str, ip: &str, ua: &str) -> Result<Model, DbErr> {
        let url_str = truncate_url(url);
        let hex = url_key::sha256_hex(&url_str);

        if let Some(found) = ShortUrl::find()
            .filter(Column::UrlSha256Hex.eq(hex.clone()))
            .order_by_desc(Column::Id)
            .one(&self.db)
            .await?
        {
            return Ok(found);
        }

        let key = self.create_key(&hex).await?;

        let now = Local::now();
        let created_time = now.format(TIME_FORMAT).to_string();
        let expired_time = (now + Duration::days(60)).format(TIME_FORMAT).to_string();

        let model = ActiveModel {
            id: Set(snowflake::next_id()),
            url_key: Set(key),
            url_raw: Set(url_str),
            url_sha256_hex: Set(hex),
            expired_time: Set(expired_time),
            visit_num: Set(0),
            created_ip: Set(ip.to_string()),
            created_ua: Set(ua.to_string()),
            created_time: Set(created_time.clone()),
            updated_time: Set(created_time),
        };
        let id = model.id.clone().unwrap();

        model.insert(&self.db).await?;

        ShortUrl::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("short_url id {id} not found after insert")))
    }

    /// 创建一个可用的短链接 KEY
    pub async fn create_key(&self, url_sha256_hex: &str) -> Result<String, DbErr> {
        // 通过摘要生成若干候选 KEY（保持顺序，取第一个未被占用的）
        let candidates = url_key::hex_short_keys(url_sha256_hex);

        let existing: HashSet<String> = ShortUrl::find()
            .filter(Column::UrlKey.is_in(candidates.clone()))
            .order_by_desc(Column::Id)
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| m.url_key)
            .collect();

        for key in &candidates {
            if !existing.contains(key) {
                return Ok(key.clone());
            }
        }

        // 兜底：随机生成直到不冲突
        let mut index = 0usize;
        loop {
            index += 1;
            let key = if index <= 20 {
                url_key::uuid_random8()
            } else if index <= 40 {
                url_key::uuid_random22()
            } else {
                crate::util::nanoid::random_nanoid()
            };

            let count = ShortUrl::find()
                .filter(Column::UrlKey.eq(key.clone()))
                .count(&self.db)
                .await?;
            if count == 0 {
                return Ok(key);
            }
        }
    }

    /// 根据短链接 KEY 获取短链接信息
    pub async fn get_by_url_key(&self, key: &str) -> Result<Option<Model>, DbErr> {
        ShortUrl::find()
            .filter(Column::UrlKey.eq(key))
            .order_by_desc(Column::Id)
            .one(&self.db)
            .await
    }

    /// 访问短链接，访问次数 +1
    pub async fn visit(&self, id: i64) -> Result<(), DbErr> {
        ShortUrl::update_many()
            .col_expr(Column::VisitNum, Expr::col(Column::VisitNum).add(1))
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// 删除指定 KEY 的所有短链接
    pub async fn delete(&self, url_key: &str) -> Result<(), DbErr> {
        ShortUrl::delete_many()
            .filter(Column::UrlKey.eq(url_key))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// 删除指定 KEY 且由指定 IP 创建的短链接
    pub async fn delete_by_ip(&self, url_key: &str, ip: &str) -> Result<(), DbErr> {
        ShortUrl::delete_many()
            .filter(Column::UrlKey.eq(url_key))
            .filter(Column::CreatedIp.eq(ip))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}

/// 按原逻辑截断 URL：长度 >= 8192 时截取前 8191 个字符
fn truncate_url(url: &str) -> String {
    if url.len() < URL_MAX_LENGTH {
        return url.to_string();
    }
    let mut end = URL_MAX_LENGTH - 1;
    while end > 0 && !url.is_char_boundary(end) {
        end -= 1;
    }
    url[..end].to_string()
}

/// 将存储的时间字符串（含毫秒）格式化为秒级展示
pub fn format_display_time(value: &str) -> String {
    if value.len() >= 19 {
        value[..19].to_string()
    } else {
        value.to_string()
    }
}
