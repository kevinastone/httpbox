use uri_path::{path, Path};

macro_rules! matches {
    ($($name:ident: $path:expr => $expected:tt),* $(,)?) => {
        $(
            paste::item! {
                #[test]
                fn [<test_matches_ $name>]() {
                    let path: Path = $path;
                    __assert_matches!(@internal path, is_some, $expected);
                }
            }
        )*
    };
}

macro_rules! non_matches {
    ($($name:ident: $path:expr => $expected:tt),* $(,)?) => {
        $(
            paste::item! {
                #[test]
                fn [<test_non_matches_ $name>]() {
                    let path: Path = $path;
                    __assert_matches!(@internal path, is_none, $expected);
                }
            }
        )*
    };
}

macro_rules! __assert_matches {
    (@internal $path:ident, $assertion:ident, $expected:literal) => {{
        assert!($path.matches($expected).$assertion());
    }};
    (@internal $path:ident, $assertion:ident, [$($expected:literal),* $(,)?]) => {{
        $(
            assert!($path.matches($expected).$assertion());
        )*

    }};
}

matches! {
    literal: "/test".into() => "/test",
    regex_digits: path!("test" / [param ~= r"\d+"]) => ["/test/1", "/test/123", "/test/000"],
    regex_digits_prefixed: path!("test" / [param ~= r"user-\d+"]) => ["/test/user-1", "/test/user-123", "/test/user-000"],
}

non_matches! {
    literal: "/test".into() => ["/", "/other", "/test/other"],
    regex_digits: path!("test" / [param ~= r"\d+"]) => ["/test", "/test/", "/test/abc"],
}
