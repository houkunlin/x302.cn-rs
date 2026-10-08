//! 真实 IP 提取，逻辑对应原 Kotlin `exts/Request.kt`。

use std::net::SocketAddr;

use axum::http::HeaderMap;
use once_cell::sync::Lazy;
use regex::Regex;

const IP_KEYS: [&str; 4] = [
    "X-Forwarded-For",
    "X-Real-IP",
    "Proxy-Client-IP",
    "WL-Proxy-Client-IP",
];

const DEFAULT_LOCAL_IP6: [&str; 2] = ["0:0:0:0:0:0:0:1", "::1"];

static IP_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^((2(5[0-5]|[0-4]\d))|[0-1]?\d{1,2})(\.((2(5[0-5]|[0-4]\d))|[0-1]?\d{1,2})){3}$")
        .expect("valid ip regex")
});

/// 从请求头与远端地址中提取真实 IP
pub fn request_ip(headers: &HeaderMap, remote: Option<SocketAddr>) -> String {
    let mut ip: Option<String> = None;
    let mut found = false;

    for key in IP_KEYS {
        if let Some(value) = headers.get(key).and_then(|v| v.to_str().ok()) {
            ip = Some(value.to_string());
            found = has_ip(value);
            if found {
                break;
            }
        }
    }

    if !found {
        ip = remote.map(|addr| addr.ip().to_string());
    }

    match ip {
        Some(value) => obtain_ip(&value),
        None => String::new(),
    }
}

fn has_ip(ip: &str) -> bool {
    !ip.trim().is_empty() && !ip.eq_ignore_ascii_case("unknown")
}

fn obtain_ip(ip: &str) -> String {
    let real_ip = if ip.contains(',') {
        ip.split(',').next().unwrap_or(ip)
    } else {
        ip
    };

    if DEFAULT_LOCAL_IP6.contains(&real_ip) {
        return "127.0.0.1".to_string();
    }

    if IP_PATTERN.is_match(real_ip) {
        return real_ip.to_string();
    }

    "0.0.0.0".to_string()
}
