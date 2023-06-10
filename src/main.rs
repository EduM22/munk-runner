use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use wasmtime::*;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .route("/wasm", get(handler_wasm))
        .fallback(fallback);
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

async fn handler_wasm() -> impl IntoResponse {
    let engine = Engine::default();

    let wat = r#"
        (module
            (func (export "run") (result i32)
                i32.const 42
            )
        )
    "#;

    let module_res = Module::new(&engine, wat);

    let module = match module_res {
        Ok(module) => module,
        Err(_) => return (StatusCode::NOT_FOUND, "Not Found").into_response(),
    };

    let mut store = Store::new(&engine, ());
    let instance_res = Instance::new(&mut store, &module, &[]);

    let instance = match instance_res {
        Ok(instance) => instance,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    let function = instance.get_func(&mut store, "run");

    if function.is_none() {
        return (StatusCode::FAILED_DEPENDENCY, "No Run function").into_response();
    }

    let typed_function_res = function.unwrap().typed::<(), i32>(&store);

    let typed_function = match typed_function_res {
        Ok(typed_function) => typed_function,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    let result = typed_function.call(&mut store, ());

    let result = match result {
        Ok(result) => result,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    let body = format!("<h1>{}</h1>", result);

    (
        StatusCode::OK,
        [("x-app-version", "v0.1.0"), ("content-type", "text/html")],
        body,
    )
        .into_response()
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
