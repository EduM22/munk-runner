use axum::{http::StatusCode, response::Html, routing::get, Router};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", get(handler)).fallback(fallback);
    // run it
    let addr = &SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 8080);
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
