use std::collections::BTreeMap;
use uri_path::{path, Path};

#[test]
fn test_replace_literal() {
    let path: Path = "/test".into();
    let params = BTreeMap::new();
    assert_eq!(path.replace(&params).unwrap().to_string(), "/test");
}

#[test]
fn test_replace_literal_with_single_param() {
    let path: Path = "/test".into();
    let mut params = BTreeMap::new();
    params.insert("first", "value");
    assert_eq!(
        path.replace(&params).unwrap().to_string(),
        "/test?first=value"
    );
}
#[test]
fn test_replace_literal_with_multiple_params() {
    let path: Path = "/test".into();
    let mut params = BTreeMap::new();
    params.insert("first", "value");
    params.insert("second", "another");
    assert_eq!(
        path.replace(&params).unwrap().to_string(),
        "/test?first=value&second=another"
    );
}

#[test]
fn test_replace_segmented_missing_param() {
    let path: Path = path!("test" / param);
    let params = BTreeMap::new();
    assert!(path.replace(&params).is_none());
}

#[test]
fn test_replace_segmented_with_path_param() {
    let path: Path = path!("test" / param);
    let mut params = BTreeMap::new();
    params.insert("param", "value");
    assert_eq!(path.replace(&params).unwrap().to_string(), "/test/value");
}

#[test]
fn test_replace_segmented_with_extra_params() {
    let path: Path = path!("test" / param);
    let mut params = BTreeMap::new();
    params.insert("param", "value");
    params.insert("first", "value");
    params.insert("second", "another");
    assert_eq!(
        path.replace(&params).unwrap().to_string(),
        "/test/value?first=value&second=another"
    );
}

#[test]
fn test_replace_regex_missing_param() {
    let path: Path = path!("test" / [param ~= r"\d+"]);
    let params = BTreeMap::new();
    assert!(path.replace(&params).is_none());
}

#[test]
fn test_replace_regex_with_path_param() {
    let path: Path = path!("test" / [param ~= r"\d+"]);
    let mut params = BTreeMap::new();
    params.insert("param", "123");
    assert_eq!(path.replace(&params).unwrap().to_string(), "/test/123");
}

#[test]
fn test_replace_regex_with_extra_params() {
    let path: Path = path!("test" / [param ~= r"\d+"]);
    let mut params = BTreeMap::new();
    params.insert("param", "123");
    params.insert("first", "value");
    params.insert("second", "another");
    assert_eq!(
        path.replace(&params).unwrap().to_string(),
        "/test/123?first=value&second=another"
    );
}
