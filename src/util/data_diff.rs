//! 集合差异对比，对应原 Kotlin 项目的 `DataDiff`。

use std::collections::HashSet;
use std::hash::Hash;

/// 差异对比结果
#[derive(Debug, Clone)]
pub struct DiffResult<T> {
    /// 公共数据
    pub common_keys: HashSet<T>,
    /// 需要从 DB 删除的数据
    pub delete_keys: HashSet<T>,
    /// 需要保存的新 KEY
    pub new_keys: HashSet<T>,
}

/// 对比集合差异
pub fn set<T>(db_keys: &HashSet<T>, new_keys: &HashSet<T>) -> DiffResult<T>
where
    T: Eq + Hash + Clone,
{
    let common_keys: HashSet<T> = db_keys.intersection(new_keys).cloned().collect();

    let delete_keys = if db_keys.len() > common_keys.len() {
        db_keys.difference(&common_keys).cloned().collect()
    } else {
        HashSet::new()
    };

    let new_only = if new_keys.len() > common_keys.len() {
        new_keys.difference(&common_keys).cloned().collect()
    } else {
        HashSet::new()
    };

    DiffResult {
        common_keys,
        delete_keys,
        new_keys: new_only,
    }
}
