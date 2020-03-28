macro_rules! path {
    () => {{
        $crate::path::Path(vec![])
    }};
    ($first:tt $(/ $tail:tt)*) => {{
    $crate::path::Path(vec![
        __path_segment!($first)
        $( , __path_segment!($tail) )*
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
