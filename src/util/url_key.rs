//! 短链接 KEY 生成工具，算法与原 Kotlin `UrlKey` 完全一致。

use md5::Md5;
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

/// 62 进制字符表（a-z0-9A-Z）
pub const CHARS1: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// 64 进制字符表（a-zA-Z0-9_- 顺序为 a-z0-9A-Z-_）
pub const CHARS2: &[u8; 64] =
    b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ-_";

/// 生成 6 位随机 KEY
pub fn uuid_random6() -> String {
    uuid_random(6, CHARS1, 0x3E)
}

/// 生成 8 位随机 KEY
pub fn uuid_random8() -> String {
    uuid_random(8, CHARS1, 0x3E)
}

fn uuid_random(len: usize, chars: &[u8], modulus: usize) -> String {
    let uuid = Uuid::new_v4().to_string().replace('-', "");
    let bytes = uuid.as_bytes();
    let mut out = String::with_capacity(len);
    for i in 0..len {
        let segment = std::str::from_utf8(&bytes[i * 4..i * 4 + 4]).expect("uuid hex ascii");
        let x = u32::from_str_radix(segment, 16).expect("uuid segment is hex");
        out.push(chars[(x as usize) % modulus] as char);
    }
    out
}

/// 生成 22 位随机 KEY（每 3 个十六进制字符转换为 2 个字符）
pub fn uuid_random22() -> String {
    let uuid = Uuid::new_v4().to_string().replace('-', "");
    let bytes = uuid.as_bytes();
    let mut out = String::with_capacity(22);
    for i in 0..10 {
        let segment = std::str::from_utf8(&bytes[i * 3..i * 3 + 3]).expect("uuid hex ascii");
        let x = u32::from_str_radix(segment, 16).expect("uuid segment is hex");
        out.push(CHARS2[(x as usize) / 0x40] as char);
        out.push(CHARS2[(x as usize) % 0x40] as char);
    }
    out.push(bytes[30] as char);
    out.push(bytes[31] as char);
    out
}

/// MD5 摘要后生成 6 位短码
pub fn short_url1(url: &str) -> Vec<String> {
    hex_short_keys(&md5_hex(url))
}

/// SHA256 摘要后生成 6 位短码
pub fn short_url2(url: &str) -> Vec<String> {
    hex_short_keys(&sha256_hex(url))
}

/// SHA512 摘要后生成 6 位短码
pub fn short_url3(url: &str) -> Vec<String> {
    hex_short_keys(&sha512_hex(url))
}

pub fn md5_hex(input: &str) -> String {
    hex::encode(Md5::digest(input.as_bytes()))
}

pub fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

pub fn sha512_hex(input: &str) -> String {
    hex::encode(Sha512::digest(input.as_bytes()))
}

/// 把摘要的十六进制字符串按 8 位一组生成 6 位短码
pub fn hex_short_keys(digest_hex: &str) -> Vec<String> {
    let bytes = digest_hex.as_bytes();
    let mut result = Vec::with_capacity(digest_hex.len() / 8);

    for i in 0..(digest_hex.len() / 8) {
        let segment = std::str::from_utf8(&bytes[i * 8..i * 8 + 8]).expect("digest ascii");
        // 与 0x3FFFFFFF 进行位与运算，取得 30 位有效值
        let mut value = (u32::from_str_radix(segment, 16).expect("digest is hex") & 0x3FFF_FFFF) as i64;
        let mut out = String::with_capacity(6);
        for _ in 0..6 {
            let index = (0x3D & value) as usize;
            out.push(CHARS1[index] as char);
            value >>= 5;
        }
        result.push(out);
    }
    result
}
