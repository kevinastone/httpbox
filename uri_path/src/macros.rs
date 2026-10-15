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

    // Internal TT-muncher implementation
    (@parse $v:ident,) => {};
    (@parse $v:ident, * [$i:ident ~ $re:literal]) => {{
        $v.push($crate::PathSegment::Dynamic($crate::PathParam::catch_all(
            stringify!($i),
            $crate::PathToken::Regex($crate::regex::Regex::new($re).unwrap()),
        )));
    }};
    (@parse $v:ident, * [$i:ident ~ $re:literal] / $($rest:tt)*) => {
        compile_error!("catch-all segment (*[" stringify!($i) " ~ ...]) must be the last segment in path");
    };
    (@parse $v:ident, * $i:ident) => {
        $v.push($crate::PathSegment::Dynamic($crate::PathParam::catch_all(
            stringify!($i),
            $crate::PathToken::Any,
        )));
    };
    (@parse $v:ident, * $i:ident / $($rest:tt)*) => {
        compile_error!("catch-all segment (*" stringify!($i) ") must be the last segment in path");
    };
    (@parse $v:ident, [$i:ident ~ $re:literal]) => {
        $v.push($crate::path!(@segment [$i ~ $re]));
    };
    (@parse $v:ident, [$i:ident ~ $re:literal] / $($rest:tt)*) => {
        $v.push($crate::path!(@segment [$i ~ $re]));
        $crate::path!(@parse $v, $($rest)*);
    };
    (@parse $v:ident, $s:literal) => {
        $v.push($crate::path!(@segment $s));
    };
    (@parse $v:ident, $s:literal / $($rest:tt)*) => {
        $v.push($crate::path!(@segment $s));
        $crate::path!(@parse $v, $($rest)*);
    };
    (@parse $v:ident, $i:ident) => {
        $v.push($crate::path!(@segment $i));
    };
    (@parse $v:ident, $i:ident / $($rest:tt)*) => {
        $v.push($crate::path!(@segment $i));
        $crate::path!(@parse $v, $($rest)*);
    };

    () => {{
        $crate::Path(vec![])
    }};

    ($($tokens:tt)+) => {{
        let mut segments = vec![];
        $crate::path!(@parse segments, $($tokens)+);
        $crate::Path(segments)
    }};
}
