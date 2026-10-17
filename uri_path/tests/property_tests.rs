use proptest::prelude::*;
use std::collections::BTreeMap;
use uri_path::path;

// Generates valid single path segments without slashes or empty strings
fn valid_segment() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,32}"
}

// Generates arbitrary string inputs including edge cases:
// slashes, percent signs, nulls, whitespace, unicode, long paths, etc.
fn arbitrary_path_input() -> impl Strategy<Value = String> {
    prop_oneof![
        // Completely arbitrary unicode strings
        "\\PC{0,100}",
        // Strings with multiple leading/trailing/consecutive slashes
        r"/+([a-zA-Z0-9_-]+/+)*[a-zA-Z0-9_-]*",
        // Percent-encoded variations
        r"(/%[0-9a-fA-F]{2})+",
        // Query strings and fragments attached
        r"/[a-z]+(\?[a-z]+=[a-z]+)?(#[a-z]+)?",
    ]
}

proptest! {
    /// Invariant: `Path::matches` must NEVER panic on any input string,
    /// regardless of special characters, consecutive slashes, null bytes, or length.
    #[test]
    fn matches_never_panics_literal(input in arbitrary_path_input()) {
        let p = path!("api" / "v1" / "users");
        let _ = p.matches(&input);
    }

    #[test]
    fn matches_never_panics_dynamic(input in arbitrary_path_input()) {
        let p = path!("users" / id / "profile");
        let _ = p.matches(&input);
    }

    #[test]
    fn matches_never_panics_catch_all(input in arbitrary_path_input()) {
        let p = path!("static" / *tail);
        let _ = p.matches(&input);
    }

    #[test]
    fn matches_never_panics_regex(input in arbitrary_path_input()) {
        let p = path!("items" / [id ~ r"\d+"] / *[rest ~ r"[a-z]+"]);
        let _ = p.matches(&input);
    }

    /// Invariant: Parameter roundtrip (replace -> matches)
    /// If valid segments are provided via replace(), matching the generated path
    /// MUST always recover the exact original parameter values.
    #[test]
    fn dynamic_param_roundtrip(
        user_id in valid_segment(),
        profile in valid_segment()
    ) {
        let p = path!("users" / id / "profile" / section);

        let mut params = BTreeMap::new();
        params.insert("id", user_id.as_str());
        params.insert("section", profile.as_str());

        let rendered = p.replace(&params).expect("replace should succeed with all params").to_string();
        let matched = p.matches(&rendered).expect("rendered path should match");

        prop_assert_eq!(matched.get("id"), Some(&user_id));
        prop_assert_eq!(matched.get("section"), Some(&profile));
    }

    /// Invariant: Catch-all tail segment roundtrip
    #[test]
    fn catch_all_roundtrip(
        tail_segments in prop::collection::vec(valid_segment(), 1..6)
    ) {
        let p = path!("files" / *tail);
        let tail_str = tail_segments.join("/");

        let mut params = BTreeMap::new();
        params.insert("tail", tail_str.as_str());

        let rendered = p.replace(&params).expect("replace should succeed").to_string();
        let matched = p.matches(&rendered).expect("rendered path should match");

        prop_assert_eq!(matched.get("tail"), Some(&tail_str));
    }

    /// Invariant: Exact literal matching consistency
    /// A path consisting of literal segments must match its exact representation
    /// (with any number of redundant leading/trailing/repeated slashes normalized).
    #[test]
    fn literal_matching_slashes(
        prefix_slashes in 0..5usize,
        middle_slashes in 1..5usize,
        trailing_slashes in 0..5usize,
    ) {
        let p = path!("foo" / "bar");

        let input = format!(
            "{}{}{}{}",
            "/".repeat(prefix_slashes),
            "foo",
            "/".repeat(middle_slashes),
            "bar".to_string() + &"/".repeat(trailing_slashes)
        );

        prop_assert!(p.matches(&input).is_some());
    }

    /// Invariant: Regex segment constraint
    /// Numeric regex route MUST match when given digits, and MUST reject non-digits.
    #[test]
    fn regex_digits_matching(
        digits in "[0-9]{1,16}",
        non_digits in "[a-zA-Z]{1,16}"
    ) {
        let p = path!("id" / [val ~ r"^\d+$"]);

        let match_url = format!("/id/{digits}");
        prop_assert!(p.matches(&match_url).is_some());

        let reject_url = format!("/id/{non_digits}");
        prop_assert!(p.matches(&reject_url).is_none());
    }
}
