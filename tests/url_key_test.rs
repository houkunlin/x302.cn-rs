use x302_cn::util::{nanoid, url_key};

const LONG_URL: &str = "http://video.weibo.com/show?fid=1034:c775dfcdd18c16eff10665ff567a9853";

#[test]
fn short_url_vectors_match_kotlin() {
    assert_eq!(
        url_key::short_url1(LONG_URL),
        vec!["UNvyYv", "rYZVfq", "IBfaUz", "vM3aym"]
    );

    assert_eq!(
        url_key::short_url2(LONG_URL),
        vec![
            "IBj6vm", "y6nmYv", "Q3EVRn", "Jvq63m", "VneyIf", "miYNNb", "mMJzqy", "2ymuEn"
        ]
    );

    assert_eq!(
        url_key::short_url3(LONG_URL),
        vec![
            "qymIVv", "bu2yam", "faM3q2", "2iIVb2", "j6R3a2", "Ffeium", "JvmmI3", "zuuUb2",
            "zaEZje", "bQNbYr", "jmIvii", "2yqAFb", "E7bMjy", "QBZfue", "6Znyea", "zeIZzy"
        ]
    );

    assert_eq!(
        url_key::sha512_hex(LONG_URL),
        "2bb633521c0d7ac33d2ec847381db15c7c2edc0b196518ebfac632ef3c3d52db4cbea81967e0c6c1508bb18b42b34b5cf0b8046a8942f4b6804c3fa0b59eb899"
    );
}

#[test]
fn random_key_lengths() {
    assert_eq!(url_key::uuid_random6().len(), 6);
    assert_eq!(url_key::uuid_random8().len(), 8);
    assert_eq!(url_key::uuid_random22().len(), 22);
    assert_eq!(nanoid::random_nanoid().len(), 29);
}

#[test]
fn hex_short_keys_are_deterministic() {
    let a = url_key::hex_short_keys(&url_key::sha256_hex(LONG_URL));
    let b = url_key::hex_short_keys(&url_key::sha256_hex(LONG_URL));
    assert_eq!(a, b);
    assert_eq!(a.len(), 8);
}
