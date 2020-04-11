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

    ($($segment:tt) / *) => {{
    $crate::Path(vec![
        $($crate::path!(@segment $segment) ),*
        ])
    }};
}
