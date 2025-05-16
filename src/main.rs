use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
};

use deno_core::{
    serde_v8::{self, from_v8},
    v8, JsRuntime, RuntimeOptions,
};
use deno_core::futures::executor::block_on;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", any(handler_adapter));
    //.route("/:param", any(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handler_adapter() -> impl IntoResponse {
    handle_js().await
}

struct JsResponseData {
    status: u64,
    body: String,
    headers: Option<serde_json::Value>,
}

fn run_js_logic(/* method: String, uri: String */) -> Result<JsResponseData, String> {
    // For this example, keep hardcoded values. In a real app, pass them.
    let local_method = "GET".to_string();
    let uri_path = "/".to_string();

    let mut runtime = match setup_runtime() {
        Ok(rt) => rt,
        Err(e) => return Err(format!("Runtime setup error: {e}")),
    };

    if let Err(e) = runtime.execute_script(
        "<user_script>",
        // Ensure your JS code is correct and uses the passed `req` object
        "Munk.serve((req) => `Hello ${req.method} from ${req.url}`)",
    ) {
        return Err(format!("User script error: {e}"));
    }

    let full_url = format!("http://localhost:3000{}", uri_path);
    let js_req = serde_json::json!({
        "method": local_method,
        "url": full_url,
        "headers": {},
        "body": "",
        "query": ""
    });

    let handler_call_script = format!(
        r#"
        (async function() {{
            const req = {req_json};
            const resp = await globalThis.__munkHandler__(req);
            /*const headersObj = {{}};
            for (const [key, value] of resp.headers.entries()) {{
                headersObj[key] = value;
            }}*/
            //const bodyText = await resp.text();
            globalThis.__response__ = resp;
            
            //{{
            //    status: resp.status,
            //    headers: headersObj,
            //    body: bodyText
            //}};
        }})()
        "#,
        req_json = js_req
    );

    // Define an inner async function to manage JS async operations
    async fn execute_handler_and_pump_event_loop(
        rt: &mut JsRuntime,
        script: String,
    ) -> Result<(), deno_core::anyhow::Error> {
        rt.execute_script("<call_handler>", script)?;
        rt.run_event_loop(Default::default()).await?;
        Ok(())
    }

    if let Err(e) = block_on(execute_handler_and_pump_event_loop(
        &mut runtime,
        handler_call_script,
    )) {
        return Err(format!("JS handler execution or event loop error: {e}"));
    }

    let (status, body, headers_json) = {
        let scope = &mut runtime.handle_scope();
        let context = scope.get_current_context();
        let global = context.global(scope);

        let response_key = v8::String::new(scope, "__response__").unwrap();
        let js_value = global
            .get(scope, response_key.into())
            .unwrap_or_else(|| v8::undefined(scope).into());

        if js_value.is_undefined() {
            return Err("JS did not set '__response__' global variable.".to_string());
        }

        let deserialized: serde_json::Value = match from_v8(scope, js_value) {
            Ok(val) => val,
            Err(e) => return Err(format!("Failed to deserialize JS response: {e}")),
        };

        let a = deserialized.to_string();
        let b = a;

        let status = deserialized
            .get("status")
            .and_then(|s| s.as_u64())
            .unwrap_or(200);
        let body = deserialized
            .get("body")
            .and_then(|b| b.as_str())
            .map(String::from)
            .unwrap_or_default();
        let headers_json = deserialized.get("headers").cloned();
        (status, body, headers_json)
    };

    Ok(JsResponseData {
        status,
        body,
        headers: headers_json,
    })
}

async fn handle_js(/* method: String, uri: String */) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || run_js_logic(/* method, uri */)).await {
        Ok(Ok(js_response_data)) => {
            let mut headers = HeaderMap::new();
            if let Some(serde_json::Value::Object(hdrs)) = js_response_data.headers {
                for (key, value) in hdrs {
                    if let Some(val_str) = value.as_str() {
                        match header::HeaderName::from_bytes(key.as_bytes()) {
                            Ok(header_name) => match header::HeaderValue::from_str(val_str) {
                                Ok(header_value) => {
                                    headers.insert(header_name, header_value);
                                }
                                Err(e) => {
                                    eprintln!("Invalid header value for key {key}: {val_str} ({e})")
                                }
                            },
                            Err(e) => eprintln!("Invalid header name: {key} ({e})"),
                        }
                    }
                }
            }
            (
                StatusCode::from_u16(js_response_data.status as u16)
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                headers,
                js_response_data.body,
            )
                .into_response()
        }
        Ok(Err(e_str)) => {
            eprintln!("Error from JS execution thread: {e_str}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e_join) => {
            eprintln!("Spawn_blocking task failed: {e_join}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn setup_runtime() -> Result<JsRuntime, deno_core::anyhow::Error> {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        ..Default::default()
    });

    runtime.execute_script(
        "<munk_api>",
        r#"
        globalThis.Munk = {
            serve(handler) {
                if (typeof handler !== 'function') {
                    throw new Error("Munk.serve expects a function");
                }
                // This handler will be called by the Rust side
                globalThis.__munkHandler__ = handler;
            }
        };
        "#,
    )?;

    Ok(runtime)
}