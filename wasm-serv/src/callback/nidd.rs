// use thingspace_sdk::models::NiddCallback;
use worker::{Request, Response, RouteContext};

pub async fn receive_nidd_msg(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
  let ctype = req.headers().get("Content-Type");

  let Ok(ctype) = ctype else {
    return Response::error("Missing 'Content-Type' header", 400);
  };
  let Some(ctype) = ctype else {
    return Response::error("Bad 'Content-Type' header", 400);
  };

  if ctype == "application/json" {
    // let body = req.json::<NiddCallback>().await;

    let namespace = ctx.durable_object("PIGEON")?;
    let stub = namespace.id_from_name("A")?.get_stub()?;
    let _ = stub.fetch_with_request(req).await;

    // match body {
    //   Ok(b) => {
    //     console_log!("{b:?}");
    //     // worker::console_log!("{:?}", b.nidd_response);
    //   }
    //   Err(e) => {
    //     console_error!("{e}");
    //     console_debug!("{:?}", req.text().await);
    //   }
    // }
  } else {
    return Response::error("'Content-Type' must be 'application/json'", 400);
    // let body = req.text().await;
    // match body {
    //   Ok(b) => worker::console_log!("{b}"),
    //   Err(e) => worker::console_error!("{e}"),
    // }
  }

  Response::empty()
}
