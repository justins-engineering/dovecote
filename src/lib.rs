use worker::{Context, Env, Request, Response, Result, Router, event};

mod api;
mod cache;
mod callback;
mod pigeon;

#[event(fetch, respond_with_errors)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  match api::authenticate_browser(&req, &env).await {
    Ok(_session) => {
      Router::new()
        .get_async("/api/callback_listener", api::list_listeners)
        .post_async("/api/callback_listener", api::create_listeners)
        .delete_async("/api/callback_listener/:name", api::delete_listeners)
        .post_async("/api/device", api::list_devices)
        .post_async("/api/send_nidd", api::send_nidd_msg)
        .or_else_any_method_async("/api", log_request)
        .run(req, env)
        .await
    }
    Err(_) => {
      Router::new()
        .post_async("/vzw/nidd", callback::receive_nidd_msg)
        .post_async("/vzw/update_tables", callback::update_tables)
        .get_async("/vzw/update_tables", callback::update_tables)
        .get_async("/connect", websocket)
        .or_else_any_method_async("/vzw", log_request)
        .run(req, env)
        .await
    }
  }
}

// #[event(fetch, respond_with_errors)]
// async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
//   Router::new()
//     .get_async("/api/callback_listener", api::list_listeners)
//     .post_async("/api/callback_listener", api::create_listeners)
//     .delete_async("/api/callback_listener/:name", api::delete_listeners)
//     .post_async("/api/device", api::list_devices)
//     .post_async("/api/send_nidd", api::send_nidd_msg)
//     .or_else_any_method_async("/api", log_request)
//     .post_async("/vzw/nidd", callback::receive_nidd_msg)
//     .post_async("/vzw/update_tables", callback::update_tables)
//     .get_async("/vzw/update_tables", callback::update_tables)
//     .get_async("/connect", websocket)
//     .or_else_any_method_async("/vzw", log_request)
//     .run(req, env)
//     .await
// }

pub async fn log_request(
  mut req: Request,
  _ctx: worker::RouteContext<()>,
) -> worker::Result<Response> {
  let body = req.text().await;

  match body {
    Ok(b) => worker::console_log!("{b}"),
    Err(e) => worker::console_error!("{e}"),
  }

  // Response::error("Unauthorized", 401)
  Response::empty()
}

use futures::StreamExt;

pub async fn websocket(req: Request, _ctx: worker::RouteContext<()>) -> worker::Result<Response> {
  if let Ok(Some(upgrade_header)) = req.headers().get("Upgrade")
    && upgrade_header != "websocket"
  {
    return worker::Response::error("Expected Upgrade: websocket", 426);
  }

  let ws = worker::WebSocketPair::new()?;
  let client = ws.client;
  let server = ws.server;
  server.accept()?;
  let _ = server.send_with_bytes("Hello from server");

  worker::wasm_bindgen_futures::spawn_local(async move {
    let mut event_stream = server.events().expect("could not open stream");

    while let Some(event) = event_stream.next().await {
      match event.expect("received error in websocket") {
        worker::WebsocketEvent::Message(msg) => {
          worker::console_log!("{:?}", str::from_utf8(&msg.bytes().unwrap()));
          server.send_with_bytes(msg.bytes().unwrap()).unwrap()
        }
        worker::WebsocketEvent::Close(_event) => worker::console_log!("Closed!"),
      }
    }
  });

  worker::Response::from_websocket(client)
}
