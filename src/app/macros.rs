macro_rules! etry {
    ($state:expr, $result:expr) => {
        etry!($crate::app::response::bad_request, $state, $result)
    };
    ($response:path, $state:expr, $result:expr) => {
        match $result {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(_) => return $response($state),
        }
    };
}

macro_rules! eexpect {
    ($state:expr, $option:expr) => {
        eexpect!($crate::app::response::bad_request, $state, $option)
    };
    ($response:path, $state:expr, $option:expr) => {
        match $option {
            ::std::option::Option::Some(val) => val,
            ::std::option::Option::None => return $response($state),
        }
    };
}

macro_rules! future_etry {
    ($state:expr, $result:expr) => {
        match $result {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(e) => {
                return ::futures::future::err(($state, e.into_handler_error()));
            }
        }
    };
    ($status:path, $state:expr, $result:expr) => {
        match $result {
            ::std::result::Result::Ok(val) => val,
            ::std::result::Result::Err(e) => {
                return ::futures::future::err((
                    $state,
                    e.into_handler_error().with_status($status),
                ));
            }
        }
    };
}
