# Rust Fundamentals Learning Roadmap (`TODO.md`)

This document outlines hands-on refactoring exercises and feature additions designed to deepen understanding of core Rust fundamentals using `httpbox` as a practical testbed.

---

## 1. Pinning & Structural Projections (`Pin<&mut Self>`)

**Target:** `ChunksBytes` in `chunk_bytes/src/lib.rs`

### Current State
`impl<S> Stream for ChunksBytes<S>` requires `S: Stream + Unpin` so that it can call `Pin::new(&mut self.stream).poll_next(cx)`.
If you pass an `async` stream created via `futures::stream::unfold` or an async generator (which are `!Unpin` because they hold self-referential state across `.await` points), `.chunks_bytes()` will not compile unless the caller heap-pins it first (`Box::pin`).

### Exercises
- [ ] **Manual Structural Pinning:** Remove the `S: Unpin` bound on `ChunksBytes<S>` and implement structural pin projection for `stream: S` while keeping `pending: BytesMut` and `capacity: NonZeroUsize` unpinned. Document the `unsafe` invariants (specifically regarding `Drop` and moving fields out of `Self`).
- [ ] **Safe Pin Projection (`pin-project-lite`):** Refactor `ChunksBytes<S>` to use `pin_project_lite::pin_project!` and compare the generated projection type with your manual implementation.
- [ ] **Test with `!Unpin` Streams:** Add a unit test using `futures::stream::unfold` (without `Box::pin`) piped directly into `.chunks_bytes(...)`.

```rust
// Example goal: allow !Unpin streams directly
let stream = futures::stream::unfold(0u8, |state| async move {
    if state < 5 {
        Some((vec![state, state + 1], state + 1))
    } else {
        None
    }
});
let chunks: Vec<Bytes> = stream.chunks_bytes(capacity).collect().await;
```

---

## 2. Lifetimes, Borrowing & Custom Trait Implementations

**Target:** `PathMatch`, `PathToken`, and `Router` in `uri_path/src/lib.rs`, `uri_path/src/router.rs`, and `src/http/request.rs`

### Current State
1. Every matched route parameter allocates a new `String` on the heap (`HashMap<&'static str, String>`).
2. `Request::params` in `src/http/request.rs` clones the entire `PathMatch` map just to deserialize it (`de::deserialize(self.params.clone())`).
3. `Router::insert` in `uri_path/src/router.rs` compares tokens via `p.token().to_string() == param.token().to_string()`, allocating two `String`s because `regex::Regex` does not implement `PartialEq`.

### Exercises
- [ ] **Custom `PartialEq` & `Eq` for `PathToken`:** Implement `PartialEq` and `Eq` on `PathToken` by comparing `re1.as_str() == re2.as_str()` for the `Regex` variant, then derive/implement `PartialEq` on `PathParam` and remove the `.to_string()` allocation in `Router::insert`.
- [ ] **Zero-Clone Deserialization (`IntoDeserializer` for `&'a PathMatch`):** Implement `serde::de::IntoDeserializer<'de, E>` for `&'a PathMatch` (yielding `BorrowedStrDeserializer` or references) so `Request::params` can deserialize directly from `&self.params` without cloning the `HashMap`.
- [ ] **Zero-Copy Route Matching (`PathMatch<'a>`):** Explore parameterizing `PathMatch<'a>` with `Cow<'a, str>` (borrowed `&'a str` for single-segment matches, owned `String` only when needed) and threading the lifetime through `Router::at<'r, 'p>(&'r self, path: &'p str) -> Option<Match<'r, 'p, T>>`.

---

## 3. Idiomatic Error Handling & `IntoResponse`

**Target:** `src/http/error.rs`, `src/http/response.rs`, `src/service/method/body.rs`, and `hyper_body/src/lib.rs`

### Current State
- `http::Error` (`src/http/error.rs`) is an enum of `HyperError(hyper::http::Error)` and `Failure(Box<Response>)` used for early-returning HTTP error statuses via `?`.
- `Error::into_result` is marked `async fn` despite doing no asynchronous work.
- `http::Error` does not implement `std::fmt::Display` or `std::error::Error`.
- `hyper_body::Error` wraps a `Box<dyn Error + Send + Sync>`, but does not expose `.downcast_ref::<T>()` or `.is::<T>()` helper methods.

### Exercises
- [ ] **Standard Error Traits:** Make `Error::into_result` a synchronous `fn`, and implement `std::fmt::Display` and `std::error::Error` (including `source()`) for `crate::http::Error`.
- [ ] **Downcasting on `hyper_body::Error`:** Add `pub fn is<T: StdError + 'static>(&self) -> bool` and `pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T>` to `hyper_body::Error` and add unit tests for them.
- [ ] **`IntoResponse` Pattern:** Design an `IntoResponse` trait implemented for `Response`, `StatusCode`, `String`, `&'static str`, `(StatusCode, R)`, and `Result<T, E>` (where `T: IntoResponse, E: IntoResponse`). Refactor handlers to return domain-specific error enums (e.g., `BadRequest`, `PayloadTooLarge`) instead of raw `Box<Response>` wrappers.

```rust
pub trait IntoResponse {
    fn into_response(self) -> crate::http::Result;
}
```

---

## 4. Advanced Trait System: Type-Safe Request Extractors

**Target:** `Handler` in `src/handler.rs` and `Request` in `src/http/request.rs`

### Current State
Every handler has the fixed signature `async fn(Request) -> Result` and manually extracts path/query parameters and headers inside the function body:
```rust
pub async fn bytes(req: Request) -> Result {
    let n = req.param::<u32>("n").ok_or_else(bad_request)?;
    let query = req.query::<BytesQueryParams>().map_err(|_| bad_request())?;
    // ...
}
```

### Exercises
- [ ] **`FromRequestParts` & `FromRequest` Traits:** Define traits using native `async fn` in traits (or Generic Associated Types) to extract typed data from a `Request`:
  - `Path<T>`: Deserializes route parameters via `serde` or `FromStr`.
  - `Query<T>`: Deserializes URL query parameters via `serde_urlencoded`.
  - `TypedHeader<H>`: Extracts a typed HTTP header.
- [ ] **Variadic Tuple `Handler<T>` Implementations:** Parameterize `pub trait Handler<T>: Send + Sync` and use a `macro_rules!` macro to implement `Handler<(E1, E2, ...)>` for async functions taking 1 to N extractors.

```rust
// Target ergonomic handler signature:
pub async fn bytes(
    Path(n): Path<u32>,
    Query(query): Query<BytesQueryParams>,
) -> Result {
    // ...
}
```

---

## 5. Async Concurrency, Task Tracking & Backpressure

**Target:** `Server::serve` in `src/server.rs`

### Current State
- `Server::serve` spawns detached tasks via `tokio::task::spawn` and relies on counting clones of a `watch::Receiver` (`close_tx.receiver_count()`) to wait for active connections during graceful shutdown.
- It drives the listener stream using `.map(Ok).forward(futures::sink::drain())`.
- There is no upper bound on concurrent open connections.

### Exercises
- [ ] **Structured Concurrency with `JoinSet`:** Replace the detached `tokio::task::spawn` + `watch::Receiver` reference-counting pattern with `tokio::task::JoinSet`. Log any connection tasks that terminate with a panic (`JoinError::is_panic`).
- [ ] **Explicit Accept Loop with `tokio::select!`:** Replace `.forward(futures::sink::drain())` with an explicit `loop` using `tokio::select!` over `conn_stream.next()`, `shutdown_signal`, and `join_set.join_next()` so completed connections are reaped continuously while the server runs.
- [ ] **Connection Backpressure (`Semaphore`):** Add a configurable `max_connections` limit using `Arc<tokio::sync::Semaphore>` and `acquire_owned().await`, tying the permit's lifetime to the spawned connection task via RAII.

---

## 6. Macros, `LazyLock` & Compile-Time Validation

**Target:** `path!` macro in `uri_path/src/macros.rs`

### Current State
Every time `path!("user" / [id ~ r"\d+"])` is evaluated, it calls `Regex::new($re).unwrap()` at runtime:
```rust
(@segment [$i:ident ~ $re:literal]) => {{
    $crate::PathSegment::Dynamic($crate::PathParam::new(
        stringify!($i),
        $crate::PathToken::Regex($crate::regex::Regex::new($re).unwrap()),
    ))
}};
```

### Exercises
- [ ] **One-Time Initialization with `std::sync::LazyLock`:** Update `PathToken::Regex` (or the `path!` macro expansion) to store or clone a `static RE: std::sync::LazyLock<regex::Regex>` so regex compilation happens at most once per macro call-site.
- [ ] **Procedural Macro (`proc-macro`):** Create a companion `uri_path_macros` proc-macro crate that parses the `path!(...)` tokens at compile time, validates the regular expression syntax during `cargo check` / `cargo build`, and emits a span-accurate `compile_error!` if a regex literal is invalid.
