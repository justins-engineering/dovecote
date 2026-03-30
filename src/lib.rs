use ory_kratos_client_wasm::apis::{configuration::Configuration, frontend_api::to_session};
use worker::{
  Context, Env, Request, Response, Result, Router, console_error, console_log, console_warn, event,
};

pub mod models;
mod objects;

#[event(fetch, respond_with_errors)]
async fn main(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
  match authenticate_browser(&req, &env).await {
    Ok(session) => {
      let Ok(req) = req.clone_mut() else {
        return Response::error("Internal Server Error", 500);
      };

      if let Err(e) = req.headers().set("X-User-Id", &session.id.clone()) {
        console_error!("Failed to set X-User-Id header, Error: {e}");
        return Response::error("Internal Server Error", 500);
      }

      Router::new()
        // Flocks endpoints - all route to user's Flocks DO
        .on_async("/flocks/:sub_path", |req, ctx| async move {
          match validate_crud_request(req.clone()?).await {
            Ok(user_id) => {
              let namespace = ctx.durable_object("FLOCKS")?;
              let stub = namespace.id_from_name(&user_id)?.get_stub()?;
              match stub.fetch_with_request(req).await {
                Ok(r) => Ok(r),
                Err(e) => {
                  console_error!("{e}");
                  Response::error(e.to_string(), 500)
                }
              }
            }
            Err(err) => err,
          }
        })
        // Pigeons endpoints - all route to user's Pigeons DO
        .on_async("/pigeons/:sub_path", |req, ctx| async move {
          match validate_crud_request(req.clone()?).await {
            Ok(user_id) => {
              let namespace = ctx.durable_object("PIGEONS")?;
              let stub = namespace.id_from_name(&user_id)?.get_stub()?;
              match stub.fetch_with_request(req).await {
                Ok(r) => Ok(r),
                Err(e) => {
                  console_error!("{e}");
                  Response::error(e.to_string(), 500)
                }
              }
            }
            Err(err) => err,
          }
        })
        // Pigeon messages - route to specific Pigeon DO
        .on_async("/pigeon/:pigeon_id/messages", |req, ctx| async move {
          match validate_crud_request(req.clone()?).await {
            Ok(_user_id) => {
              let Some(pigeon_id) = ctx.param("pigeon_id") else {
                return Response::error("Bad Request", 400);
              };

              let namespace = ctx.durable_object("PIGEONS")?;
              let stub = namespace.id_from_name(pigeon_id)?.get_stub()?;
              match stub.fetch_with_request(req).await {
                Ok(r) => Ok(r),
                Err(e) => {
                  console_error!("{e}");
                  Response::error(e.to_string(), 500)
                }
              }
            }
            Err(err) => err,
          }
        })
        .on_async(
          "/pigeon/:pigeon_id/messages/:message_id",
          |req, ctx| async move {
            match validate_crud_request(req.clone()?).await {
              Ok(_user_id) => {
                let Some(pigeon_id) = ctx.param("pigeon_id") else {
                  return Response::error("Bad Request", 400);
                };
                let namespace = ctx.durable_object("PIGEONS")?;
                let stub = namespace.id_from_name(pigeon_id)?.get_stub()?;
                match stub.fetch_with_request(req).await {
                  Ok(r) => Ok(r),
                  Err(e) => {
                    console_error!("{e}");
                    Response::error(e.to_string(), 500)
                  }
                }
              }
              Err(err) => err,
            }
          },
        )
        .or_else_any_method_async("/", |mut req, _ctx| async move {
          match req.text().await {
            Ok(b) => console_log!("{b}"),
            Err(e) => console_error!("{e}"),
          }
          Response::error("Not Found", 404)
        })
        .run(req, env)
        .await
    }
    Err(_) => {
      match req.text().await {
        Ok(b) => console_log!("{b}"),
        Err(e) => console_error!("{e}"),
      }
      Response::error("Unauthorized", 401)
    }
  }
}

async fn validate_crud_request(req: Request) -> Result<String, worker::Result<Response>> {
  // let ctype = req.headers().get("Content-Type");

  // let Ok(ctype) = ctype else {
  //   console_error!("Missing 'Content-Type' header");
  //   return Err(Response::error("Missing 'Content-Type' header", 400));
  // };

  // let Some(ctype) = ctype else {
  //   console_error!("Bad 'Content-Type' header");
  //   return Err(Response::error("Bad 'Content-Type' header", 400));
  // };

  // if ctype != "application/json" {
  //   console_error!("'Content-Type' must be 'application/json'");
  //   return Err(Response::error(
  //     "'Content-Type' must be 'application/json'",
  //     400,
  //   ));
  // }

  let Ok(Some(user_id)) = req.headers().get("X-User-Id") else {
    console_error!("Request missing 'X-User-Id' header");
    return Err(Response::error("Unauthorized", 401));
  };

  Ok(user_id)
}

pub async fn authenticate_browser(
  req: &Request,
  env: &Env,
) -> worker::Result<ory_kratos_client_wasm::models::Session> {
  let cookie_header = req.headers().get("Cookie")?;

  match cookie_header {
    None => {
      console_error!("Request missing Cookie Header");
      Err("Unauthorized".into())
    }
    Some(ch) => {
      let conf = Configuration {
        base_path: env.var("KRATOS_BROWSER_URL")?.to_string(),
        user_agent: None,
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
      };

      match to_session(&conf, None, Some(&ch), None).await {
        Ok(session) => {
          if let Some(active) = session.active
            && active
          {
            return Ok(session);
          }
        }
        Err(e) => {
          console_warn!("Error: {e:?}");
        }
      }

      Err("Unauthorized".into())
    }
  }
}
