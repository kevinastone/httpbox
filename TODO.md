# Rust Fundamentals Learning Roadmap (`TODO.md`)

This document outlines hands-on refactoring exercises and feature additions designed to deepen understanding of core Rust fundamentals using `httpbox` as a practical testbed.

---

## 1. Lifetimes, Borrowing & Custom Trait Implementations

**Target:** `PathMatch`, `PathToken`, and `Router` in `uri_path/src/lib.rs`, `uri_path/src/router.rs`, and `src/http/request.rs`

### Current State
Every matched route parameter allocates a new `String` on the heap (`HashMap<&'static str, String>`).

### Exercises
- [ ] **Zero-Copy Route Matching (`PathMatch<'a>`):** Explore parameterizing `PathMatch<'a>` with `Cow<'a, str>` (borrowed `&'a str` for single-segment matches, owned `String` only when needed) and threading the lifetime through `Router::at<'r, 'p>(&'r self, path: &'p str) -> Option<Match<'r, 'p, T>>`.

---

## 2. Advanced Trait System: Type-Safe Request Extractors

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

## 3. Async Concurrency, Task Tracking & Backpressure

**Target:** `Server::serve` in `src/server.rs`

### Current State
- `Server::serve` tracks connection tasks using `tokio_util::task::TaskTracker` during graceful shutdown.
- It drives the listener stream using `.map(Ok).forward(futures::sink::drain())`.
- There is no upper bound on concurrent open connections.

### Exercises
- [ ] **Explicit Accept Loop with `tokio::select!`:** Replace `.forward(futures::sink::drain())` with an explicit `loop` using `tokio::select!` over `conn_stream.next()` and `shutdown_signal` to drive connection acceptance.
- [ ] **Connection Backpressure (`Semaphore`):** Add a configurable `max_connections` limit using `Arc<tokio::sync::Semaphore>` and `acquire_owned().await`, tying the permit's lifetime to the spawned connection task via RAII.

