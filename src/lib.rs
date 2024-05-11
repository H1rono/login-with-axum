pub fn make_router() -> axum::Router {
    axum::Router::new().route("/ping", axum::routing::get(|| async { "pong" }))
}
