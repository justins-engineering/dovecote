use ory_kratos_client_wasm::apis::{configuration::Configuration, frontend_api::to_session};
use worker::{Env, Request, console_debug};

// const SESSION_COOKIE_NAME: &str = "ory_kratos_session";

pub async fn authenticate_browser(
  req: &Request,
  env: &Env,
) -> worker::Result<ory_kratos_client_wasm::models::Session> {
  const SESSION_COOKIE_NAME: &str = "ory_kratos_session";
  let cookie_header = req.headers().get("Cookie")?;

  match cookie_header {
    None => {
      console_debug!("Missing Cookie Header");
      Err("Unauthorized".into())
    }
    Some(ch) => {
      let cookies = ch.split(';');

      for entry in cookies {
        if entry.contains(SESSION_COOKIE_NAME) {
          let mut c = entry.split('=');
          if let Some(cookie) = c.next_back() {
            console_debug!("{}: {}", SESSION_COOKIE_NAME, cookie);

            let conf = Configuration {
              base_path: env.var("KRATOS_BROWSER_URL")?.to_string(),
              user_agent: None,
              basic_auth: None,
              oauth_access_token: None,
              bearer_access_token: None,
              api_key: None,
            };

            match to_session(&conf, None, Some(cookie), None).await {
              Ok(session) => {
                if let Some(active) = session.active
                  && active
                {
                  return Ok(session);
                }
              }
              Err(e) => {
                console_debug!("Error: {e:?}");
                break;
              }
            }
          }
        }
      }

      Err("Unauthorized".into())
    }
  }
}
