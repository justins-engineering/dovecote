use worker::{Request, RequestInit, Response, RouteContext, console_error};

pub async fn receive_nidd_msg(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
  let ctype = req.headers().get("Content-Type");

  let Ok(ctype) = ctype else {
    return Response::error("Missing 'Content-Type' header", 400);
  };

  let Some(ctype) = ctype else {
    return Response::error("Bad 'Content-Type' header", 400);
  };

  if ctype == "application/json" {
    let namespace = ctx.durable_object("PIGEON")?;
    let stub = namespace.id_from_name("A")?.get_stub()?;
    let _ = stub.fetch_with_request(req).await;
  } else {
    return Response::error("'Content-Type' must be 'application/json'", 400);
  }

  Response::empty()
}

pub async fn update_tables(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
  let namespace = ctx.durable_object("PIGEON")?;
  let mut vzw_req = crate::api::list_devices(req, ctx).await?;

  let body = vzw_req.text().await?;

  let mut ri = RequestInit::new();
  ri.with_method(worker::Method::Post)
    .with_redirect(worker::RequestRedirect::Follow)
    .with_body(Some(worker::wasm_bindgen::JsValue::from_str(&body)));
  // let new_req = Request::new_with_init(&path, &ri)?;
  let new_req = Request::new_with_init("http://localhost:8787/vzw/update_tables", &ri)?;

  //
  // let json = vzw_req.json::<AccountDeviceListResponse>().await?;

  let stub = namespace.id_from_name("A")?.get_stub()?;
  match stub.fetch_with_request(new_req).await {
    Ok(r) => Ok(r),
    Err(e) => {
      console_error!("{e}");
      Response::error(e.to_string(), 500)
    }
  }
}
