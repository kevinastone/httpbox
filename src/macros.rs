macro_rules! path {
    ($($segment:tt) / *) => {{
    $crate::path::Path(vec![
        $(__path_segment!($segment) ),*
        ])
    }};
}

#[doc(hidden)]
macro_rules! __path_segment {
    ($s:literal) => {{
        $crate::path::PathSegment::Literal($s)
    }};
    ($i:ident) => {{
        $crate::path::PathSegment::Param(stringify!($i))
    }};
}
