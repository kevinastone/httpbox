macro_rules! try_or_error_response {
    ($state:expr, $result:expr) => (
        try_or_error_response!(
            $crate::app::response::bad_request, $state, $result
        )
    );
    ($response:path, $state:expr, $result:expr) => (match $result {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(_) => return $response($state)
    })
}

macro_rules! expect_or_error_response {
    ($state:expr, $option:expr) => (
        expect_or_error_response!(
            $crate::app::response::bad_request, $state, $option
        )
    );
    ($response:path, $state:expr, $option:expr) => (match $option {
        ::std::option::Option::Some(val) => val,
        ::std::option::Option::None => return $response($state)
    })
}

macro_rules! future_try_or_error_response {
    ($state:expr, $result:expr) => (match $result {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(e) =>
            return ::futures::future::err(($state, e.into_handler_error()))
    });
    ($status:path, $state:expr, $result:expr) => (match $result {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(e) =>
            return ::futures::future::err(
                ($state, e.into_handler_error().with_status($status))
            )
    })
}
