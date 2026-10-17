#[macro_export]
macro_rules! path {
    (@segment $s:literal) => {
        $crate::PathSegment::Literal($s)
    };
    (@segment $i:ident) => {
        $crate::PathSegment::Dynamic($crate::PathParam::new(
            stringify!($i),
            $crate::PathToken::Any,
        ))
    };
    (@segment [$i:ident ~ $re:literal]) => {{
        $crate::PathSegment::Dynamic($crate::PathParam::new(
            stringify!($i),
            $crate::PathToken::Regex($crate::regex::Regex::new($re).unwrap()),
        ))
    }};

    // Catch-all patterns
    (@collect [$($acc:expr,)*], * [$i:ident ~ $re:literal]) => {
        vec![
            $($acc,)*
            $crate::PathSegment::Dynamic($crate::PathParam::catch_all(
                stringify!($i),
                $crate::PathToken::Regex($crate::regex::Regex::new($re).unwrap()),
            ))
        ]
    };
    (@collect [$($acc:expr,)*], * [$i:ident ~ $re:literal] / $($rest:tt)*) => {
        compile_error!("catch-all segment (*[" stringify!($i) " ~ ...]) must be the last segment in path");
    };
    (@collect [$($acc:expr,)*], * $i:ident) => {
        vec![
            $($acc,)*
            $crate::PathSegment::Dynamic($crate::PathParam::catch_all(
                stringify!($i),
                $crate::PathToken::Any,
            ))
        ]
    };
    (@collect [$($acc:expr,)*], * $i:ident / $($rest:tt)*) => {
        compile_error!("catch-all segment (*" stringify!($i) ") must be the last segment in path");
    };

    // Single/Multiple segment patterns
    (@collect [$($acc:expr,)*], [$i:ident ~ $re:literal]) => {
        vec![$($acc,)* $crate::path!(@segment [$i ~ $re])]
    };
    (@collect [$($acc:expr,)*], [$i:ident ~ $re:literal] / $($rest:tt)*) => {
        $crate::path!(@collect [$($acc,)* $crate::path!(@segment [$i ~ $re]),], $($rest)*)
    };
    (@collect [$($acc:expr,)*], $s:literal) => {
        vec![$($acc,)* $crate::path!(@segment $s)]
    };
    (@collect [$($acc:expr,)*], $s:literal / $($rest:tt)*) => {
        $crate::path!(@collect [$($acc,)* $crate::path!(@segment $s),], $($rest)*)
    };
    (@collect [$($acc:expr,)*], $i:ident) => {
        vec![$($acc,)* $crate::path!(@segment $i)]
    };
    (@collect [$($acc:expr,)*], $i:ident / $($rest:tt)*) => {
        $crate::path!(@collect [$($acc,)* $crate::path!(@segment $i),], $($rest)*)
    };

    () => {{
        $crate::Path(vec![])
    }};

    ($($tokens:tt)+) => {{
        $crate::Path($crate::path!(@collect [], $($tokens)+))
    }};
}
