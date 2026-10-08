//! 雪花 ID 生成器。
//!
//! 与原 Kotlin 项目保持同一纪元（epoch = 2020-01-01），机器号 1。
//! 采用标准布局：41 位时间戳 + 10 位机器号 + 12 位序列号。

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

const EPOCH: i64 = 1_577_808_000_000;
const WORKER_BITS: i64 = 10;
const SEQUENCE_BITS: i64 = 12;
const MAX_WORKER: i64 = (1 << WORKER_BITS) - 1;
const MAX_SEQUENCE: i64 = (1 << SEQUENCE_BITS) - 1;
const WORKER_SHIFT: i64 = SEQUENCE_BITS;
const TIMESTAMP_SHIFT: i64 = SEQUENCE_BITS + WORKER_BITS;

struct State {
    last_timestamp: i64,
    sequence: i64,
}

/// 线程安全的雪花 ID 生成器
pub struct Snowflake {
    worker_id: i64,
    state: Mutex<State>,
}

impl Snowflake {
    pub fn new(worker_id: i64) -> Self {
        Self {
            worker_id: worker_id & MAX_WORKER,
            state: Mutex::new(State {
                last_timestamp: -1,
                sequence: 0,
            }),
        }
    }

    pub fn next_id(&self) -> i64 {
        let mut state = self.state.lock().expect("snowflake mutex poisoned");
        let mut timestamp = now_millis();
        if timestamp < state.last_timestamp {
            // 时钟回拨，退回到上一次时间戳
            timestamp = state.last_timestamp;
        }

        if timestamp == state.last_timestamp {
            state.sequence = (state.sequence + 1) & MAX_SEQUENCE;
            if state.sequence == 0 {
                // 当前毫秒序列号用尽，等待下一毫秒
                while now_millis() <= state.last_timestamp {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                timestamp = now_millis();
            }
        } else {
            state.sequence = 0;
        }
        state.last_timestamp = timestamp;

        ((timestamp - EPOCH) << TIMESTAMP_SHIFT)
            | (self.worker_id << WORKER_SHIFT)
            | state.sequence
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis() as i64
}

static SNOWFLAKE: Lazy<Snowflake> = Lazy::new(|| Snowflake::new(1));

/// 生成下一个雪花 ID
pub fn next_id() -> i64 {
    SNOWFLAKE.next_id()
}
