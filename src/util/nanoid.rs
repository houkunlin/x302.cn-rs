//! NanoID 生成器，算法与 JNanoID / 原 Kotlin 项目保持一致。

use rand::RngCore;

/// 默认字母表（URL 友好，共 64 个符号）
pub const DEFAULT_ALPHABET: &[u8] =
    b"_-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// 默认长度
pub const DEFAULT_SIZE: usize = 29;

/// 使用默认字母表与长度生成 NanoID
pub fn random_nanoid() -> String {
    random_nanoid_size(DEFAULT_SIZE)
}

/// 使用默认字母表与指定长度生成 NanoID
pub fn random_nanoid_size(size: usize) -> String {
    generate(size, DEFAULT_ALPHABET)
}

/// 按 NanoID 官方算法生成字符串
pub fn generate(size: usize, alphabet: &[u8]) -> String {
    assert!(size > 0, "size must be greater than zero");
    assert!(
        !alphabet.is_empty() && alphabet.len() < 256,
        "alphabet must contain between 1 and 255 symbols"
    );

    let mask = (2u32 << ((alphabet.len() as f64 - 1.0).ln() / 2f64.ln()).floor() as u32) - 1;
    let step = ((1.6 * mask as f64 * size as f64) / alphabet.len() as f64).ceil() as usize;

    let mut id = String::with_capacity(size);
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; step];

    while id.len() < size {
        rng.fill_bytes(&mut bytes);
        for &byte in &bytes {
            let index = (byte as u32) & mask;
            if (index as usize) < alphabet.len() {
                id.push(alphabet[index as usize] as char);
                if id.len() == size {
                    return id;
                }
            }
        }
    }
    id
}
