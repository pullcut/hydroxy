//! Docs server: renders every route to HTML and serves the hydration bundle.

#![recursion_limit = "256"] // Leptos view types nest deeply; release builds need the headroom.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use hydroxy_docs::app::{App, shell};

    let conf = get_configuration(None).expect("reads [package.metadata.leptos] from Cargo.toml");
    let options = conf.leptos_options;
    let addr = options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&options, routes, {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(options)
        .layer(axum::middleware::from_fn(cache_static))
        .layer(tower_http::compression::CompressionLayer::new());

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("docs port is free");
    println!("hydroxy docs on http://{addr}");
    axum::serve(listener, app.into_make_service()).await.expect("server runs");
}

/// Hashed build output and the fonts never change at a given URL, so browsers may keep
/// them for a year without asking again. Pages themselves stay uncached.
#[cfg(feature = "ssr")]
async fn cache_static(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    use axum::http::{HeaderValue, header};
    let path = req.uri().path();
    // Dev builds (`cargo leptos watch`) reuse unhashed `/pkg` names, so they must not be cached.
    let immutable = (path.starts_with("/pkg/") && !cfg!(debug_assertions)) || path.starts_with("/fonts/");
    let mut res = next.run(req).await;
    if immutable && res.status().is_success() {
        res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
    }
    res
}

/// The server needs the `ssr` feature; `cargo leptos` builds it with that set.
#[cfg(not(feature = "ssr"))]
fn main() {}
