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

macro_rules! async_response {
    ($future:expr) => {
        Box::new(
            $future
                .then(|(state, resp)| {
                    let r = ::gotham::handler::IntoResponse::into_response(
                        resp, &state,
                    );
                    future::ok((state, r))
                })
                .boxed()
                .compat(),
        )
    };
}
