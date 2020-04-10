#[macro_export]
macro_rules! path {
    ($($segment:tt) / *) => {{
    $crate::Path(vec![
        $($crate::__path_segment!($segment) ),*
        ])
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __path_segment {
    ($s:literal) => {{
        $crate::PathSegment::Literal($s)
    }};
    ($i:ident) => {{
        $crate::PathSegment::Dynamic($crate::PathParam {
            name: stringify!($i),
            token: $crate::PathToken::Any,
        })
    }};
    ([$i:ident ~= $re:literal]) => {{
        $crate::PathSegment::Dynamic($crate::PathParam {
            name: stringify!($i),
            token: $crate::PathToken::Regex(regex::Regex::new($re).unwrap()),
        })
    }};
}
