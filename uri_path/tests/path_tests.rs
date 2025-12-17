use uri_path::{Path, path};

macro_rules! path_test {
    (@assertion $path:ident, matches, $expected:literal) => {
        assert!($path.matches($expected).is_some());
    };

    (@assertion $path:ident, non_matches, $expected:literal) => {
        assert!($path.matches($expected).is_none());
    };

    (@assertion $path:ident, $assertion:ident, [$($expected:literal),* $(,)?]) => {
        $(
            path_test!(@assertion $path, $assertion, $expected);
        )*
    };

    (@map $ctor:path, {$($name:ident : $value:expr),* $(,)?}) => {{
        #[allow(unused_mut)]
        let mut map = $ctor();
        $(
            map.insert(stringify!($name), $value);
        )*
        map
    }};

    (@assertion $path:ident, replace, $map:tt => None) => {{
        let params = path_test!(@map ::std::collections::BTreeMap::new, $map);
        assert!(
            $path.replace(&params).is_none()
        );
    }};

    (@assertion $path:ident, replace, $map:tt => $expected:literal) => {{
        let params = path_test!(@map ::std::collections::BTreeMap::new, $map);
        assert_eq!(
            $path.replace(&params).unwrap().to_string(),
            $expected
        );
    }};

    (@assertion $path:ident, params, $input:literal => None) => {{
        assert!(
            $path.matches($input).is_none()
        );
    }};

    (@assertion $path:ident, params, $input:literal => $map:tt) => {{
        let params: ::std::collections::HashMap<&'static str, String> = path_test!(@map ::std::collections::HashMap::new, $map)
            .into_iter()
            .map(|(k, v): (&'static str, &str)| (k, v.to_string()))
            .collect();
        assert_eq!(
            $path.matches($input).unwrap(),
            params.into()
        );
    }};

    (@assertion $path:ident, $assertion:ident, {$($input:tt => $output:tt),* $(,)?}) => {
        $(
            path_test!(@assertion $path, $assertion, $input => $output);
        )*
    };

    (@assertions $path:ident, {$($assertion:ident: $tests:tt),* $(,)?}) => {
        $(
            path_test!(@assertion $path, $assertion, $tests);
        )*
    };

    ($($label:ident ( $path:expr ) $assertions:tt),* $(,)?) => {
        $(
            paste::item! {
                #[test]
                fn [<test_matches_ $label>]() {
                    let path: Path = $path;
                    path_test!(@assertions path, $assertions);
                }
            }
        )*
    };
}

path_test! {
    literal("/test".into()) {
        matches: "/test",
        non_matches: ["/", "/other", "/test/other"],
        params: {
            "/test" => {},
        },
        replace: {
            {} => "/test",
            {first: "value"} => "/test?first=value",
            {first: "value", second: "another"} => "/test?first=value&second=another",
        },
    },
    segmented(path!("test" / param)) {
        matches: ["/test/123", "/test/abc"],
        non_matches: ["/", "/test", "/test/abc/whatever"],
        params: {
            "/test" => None,
            "/test/abc" => {param: "abc"},
            "/test/abc/whatever" => None,
        },
        replace: {
            {} => None,
            {param: "value"} => "/test/value",
            {param: "value", first: "other", second: "another"} => "/test/value?first=other&second=another",
        },
    }
}

#[cfg(feature = "regex")]
path_test! {
    regex_digits(path!("test" / [param ~ r"\d+"])) {
        matches: ["/test/1", "/test/123", "/test/000"],
        non_matches: ["/test", "/test/", "/test/abc"],
        params: {
            "/test" => None,
            "/test/abc" => None,
            "/test/1" => {param: "1"},
            "/test/123" => {param: "123"},
        },
        replace: {
            {} => None,
            {param: "123"} => "/test/123",
            {param: "123", first: "other", second: "another"} => "/test/123?first=other&second=another",
        },
    },
    regex_digits_prefixed(path!("test" / [param ~ r"user-\d+"])) {
        matches: ["/test/user-1", "/test/user-123", "/test/user-000"],
        non_matches: ["/test", "/test/user", "/test/user-", "/test/user-abc"],
        params: {
            "/test" => None,
            "/test/user" => None,
            "/test/user-abc" => None,
            "/test/user-1" => {param: "user-1"},
            "/test/user-123" => {param: "user-123"},
        },
        replace: {
            {} => None,
            {param: "user-123"} => "/test/user-123",
            {param: "user-123", first: "other", second: "another"} => "/test/user-123?first=other&second=another",
        },
    },
}
