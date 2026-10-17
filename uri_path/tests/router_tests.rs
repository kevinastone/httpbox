use proptest::prelude::*;
use uri_path::{Router, path};

#[test]
fn test_router_literal_matching() {
    let mut router = Router::new();
    router.insert(path!("ip"), "ip");
    router.insert(path!("user-agent"), "user-agent");
    router.insert(path!("cookies"), "cookies");
    router.insert(path!("cookies" / "set"), "cookies_set");

    assert_eq!(router.at("/ip").map(|m| *m.value), Some("ip"));
    assert_eq!(
        router.at("/user-agent").map(|m| *m.value),
        Some("user-agent")
    );
    assert_eq!(router.at("/cookies").map(|m| *m.value), Some("cookies"));
    assert_eq!(
        router.at("/cookies/set").map(|m| *m.value),
        Some("cookies_set")
    );
    assert_eq!(router.at("/cookies/other"), None);
    assert_eq!(router.at("/not-found"), None);
}

#[test]
fn test_router_dynamic_params() {
    let mut router = Router::new();
    router.insert(path!("status" / code), "status");
    router.insert(path!("user" / id / "profile"), "user_profile");

    let m1 = router.at("/status/418").unwrap();
    assert_eq!(*m1.value, "status");
    assert_eq!(m1.params.get("code").map(|s| s.as_str()), Some("418"));

    let m2 = router.at("/user/42/profile").unwrap();
    assert_eq!(*m2.value, "user_profile");
    assert_eq!(m2.params.get("id").map(|s| s.as_str()), Some("42"));

    assert_eq!(router.at("/user/42"), None);
}

#[test]
fn test_router_precedence_literal_over_dynamic() {
    let mut router = Router::new();
    router.insert(path!("user" / "me"), "me");
    router.insert(path!("user" / id), "id");

    let m1 = router.at("/user/me").unwrap();
    assert_eq!(*m1.value, "me");
    assert!(m1.params.is_empty());

    let m2 = router.at("/user/alice").unwrap();
    assert_eq!(*m2.value, "id");
    assert_eq!(m2.params.get("id").map(|s| s.as_str()), Some("alice"));
}

#[test]
fn test_router_catch_all() {
    let mut router = Router::new();
    router.insert(path!("assets" / *path), "assets");
    router.insert(path!("assets" / "favicon.ico"), "favicon");

    let m1 = router.at("/assets/favicon.ico").unwrap();
    assert_eq!(*m1.value, "favicon");

    let m2 = router.at("/assets/css/style.css").unwrap();
    assert_eq!(*m2.value, "assets");
    assert_eq!(
        m2.params.get("path").map(|s| s.as_str()),
        Some("css/style.css")
    );
}

#[cfg(feature = "regex")]
#[test]
fn test_router_regex() {
    let mut router = Router::new();
    router.insert(path!("user" / [id ~ r"^\d+$"]), "numeric_user");
    router.insert(path!("user" / [name ~ r"^[a-zA-Z]+$"]), "alpha_user");

    let m1 = router.at("/user/12345").unwrap();
    assert_eq!(*m1.value, "numeric_user");
    assert_eq!(m1.params.get("id").map(|s| s.as_str()), Some("12345"));

    let m2 = router.at("/user/kevin").unwrap();
    assert_eq!(*m2.value, "alpha_user");
    assert_eq!(m2.params.get("name").map(|s| s.as_str()), Some("kevin"));

    assert_eq!(router.at("/user/kevin123"), None);
}

fn valid_segment() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,32}"
}

fn arbitrary_path_input() -> impl Strategy<Value = String> {
    prop_oneof![
        "\\PC{0,100}",
        r"/+([a-zA-Z0-9_-]+/+)*[a-zA-Z0-9_-]*",
        r"(/%[0-9a-fA-F]{2})+",
        r"/[a-z]+(\?[a-z]+=[a-z]+)?(#[a-z]+)?",
    ]
}

fn build_test_router() -> Router<&'static str> {
    let mut router = Router::new();
    router.insert(path!("users" / "profile"), "user_profile_literal");
    router.insert(path!("users" / id), "user_id_dynamic");
    router.insert(path!("users" / id / "settings"), "user_settings");
    router.insert(path!("assets" / *tail), "assets_catch_all");
    router
}

proptest! {
    /// Invariant: `router.at(...)` must never panic on any arbitrary input string.
    #[test]
    fn router_never_panics_on_arbitrary_input(input in arbitrary_path_input()) {
        let router = build_test_router();
        let _ = router.at(&input);
    }

    /// Invariant: Router results are identical to single `Path::matches` when
    /// testing an isolated route pattern.
    #[test]
    fn router_matches_single_path_equivalence(
        seg1 in valid_segment(),
        seg2 in valid_segment(),
    ) {
        let p = path!("api" / version / "items" / item_id);
        let mut router = Router::new();
        router.insert(path!("api" / version / "items" / item_id), "handler");

        let test_path = format!("/api/{}/items/{}", seg1, seg2);

        let path_match = p.matches(&test_path);
        let router_match = router.at(&test_path);

        match (path_match, router_match) {
            (Some(expected), Some(actual)) => {
                prop_assert_eq!(*actual.value, "handler");
                prop_assert_eq!(actual.params, expected);
            }
            (None, None) => {}
            _ => prop_assert!(false, "Router and Path::matches outcome differed for {}", test_path),
        }
    }

    /// Invariant: Literal routes always take precedence over dynamic routes
    /// regardless of insertion order.
    #[test]
    fn router_precedence_order_independent(
        literal_first: bool,
        random_id in valid_segment()
    ) {
        let mut router = Router::new();
        if literal_first {
            router.insert(path!("users" / "profile"), "literal");
            router.insert(path!("users" / id), "dynamic");
        } else {
            router.insert(path!("users" / id), "dynamic");
            router.insert(path!("users" / "profile"), "literal");
        }

        // Literal query must always resolve to "literal"
        let lit_match = router.at("/users/profile").expect("should match literal");
        prop_assert_eq!(*lit_match.value, "literal");
        prop_assert!(lit_match.params.is_empty());

        // If random_id is not "profile", it must resolve to "dynamic" with id param
        if random_id != "profile" {
            let dynamic_path = format!("/users/{}", random_id);
            let dyn_match = router.at(&dynamic_path).expect("should match dynamic");
            prop_assert_eq!(*dyn_match.value, "dynamic");
            prop_assert_eq!(dyn_match.params.get("id").map(|s| s.as_str()), Some(random_id.as_str()));
        }
    }
}
