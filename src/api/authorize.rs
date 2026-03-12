use ory_kratos_client_wasm::apis::{configuration::Configuration, frontend_api::to_session};
use worker::{Env, Request, console_debug, console_warn};

pub async fn authenticate_browser(
  req: &Request,
  env: &Env,
) -> worker::Result<ory_kratos_client_wasm::models::Session> {
  let cookie_header = req.headers().get("Cookie")?;

  match cookie_header {
    None => {
      console_warn!("Request missing Cookie Header");
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
          console_debug!("{session:?}");
          if let Some(active) = session.active
            && active
          {
            return Ok(session);
          }
        }
        Err(e) => {
          console_debug!("Error: {e:?}");
        }
      }

      Err("Unauthorized".into())
    }
  }
}

// const SESSION_COOKIE_NAME: &str = "ory_kratos_session";

// pub async fn authenticate_browser(
//   req: &Request,
//   env: &Env,
// ) -> worker::Result<ory_kratos_client_wasm::models::Session> {
//   const SESSION_COOKIE_NAME: &str = "ory_kratos_session";
//   let cookie_header = req.headers().get("Cookie")?;

//   // console_warn!("{:?}", req.headers());

//   match cookie_header {
//     None => {
//       console_warn!("Request missing Cookie Header");
//       Err("Unauthorized".into())
//     }
//     Some(ch) => {
//       let cookies: std::str::Split<'_, char> = ch.split(';');

//       for entry in cookies {
//         if entry.contains(SESSION_COOKIE_NAME) {
//           let c = entry.split_once('=');
//           if let Some((_, cookie)) = c {
//             let cors = Headers::new();

//             // if let Ok(Some(origin)) = req.headers().get("Origin") {
//             // console_warn!("url: {}", req.url()?.origin().ascii_serialization());
//             // cors.append("Origin", &req.url()?.origin().ascii_serialization())?;
//             // }

//             if let Ok(Some(origin)) = req.headers().get("Origin") {
//               console_warn!("Origin: {}", origin);
//               cors.append("Origin", &origin)?;
//             }

//             if let Ok(Some(method)) = req.headers().get("Access-Control-Request-Method") {
//               console_debug!("Access-Control-Request-Method: {}", method);
//               cors.append("Access-Control-Request-Method", &method)?;
//             }

//             if let Ok(Some(allowed_headers)) = req.headers().get("Access-Control-Request-Headers") {
//               console_debug!("Access-Control-Request-Headers: {}", allowed_headers);
//               cors.append("Access-Control-Request-Headers", &allowed_headers)?;
//             }

//             let conf = Configuration {
//               base_path: env.var("KRATOS_BROWSER_URL")?.to_string(),
//               user_agent: None,
//               basic_auth: None,
//               oauth_access_token: None,
//               bearer_access_token: None,
//               api_key: None,
//               cors: Some(cors),
//             };

//             match to_session(&conf, None, Some(&ch), None).await {
//               Ok(session) => {
//                 console_debug!("{session:?}");
//                 if let Some(active) = session.active
//                   && active
//                 {
//                   return Ok(session);
//                 }
//               }
//               Err(e) => {
//                 console_debug!("Error: {e:?}");
//                 break;
//               }
//             }
//           }
//         }
//       }

//       Err("Unauthorized".into())
//     }
//   }
// }
