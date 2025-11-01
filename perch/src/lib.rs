#![forbid(unsafe_code)]

use dioxus::prelude::*;
use ory_kratos_client_wasm::apis::configuration::Configuration;
use views::{Index, PageNotFound, Wrapper};
mod components;
mod views;

#[cfg(feature = "api")]
pub mod api;

const KRATOS_BROWSER_URL: &str = "http://127.0.0.1:4433";
trait Create {
  fn create() -> Configuration;
}

impl Create for Configuration {
  fn create() -> Configuration {
    Configuration {
      base_path: KRATOS_BROWSER_URL.to_owned(),
      user_agent: None,
      basic_auth: None,
      oauth_access_token: None,
      bearer_access_token: None,
      api_key: None,
    }
  }
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
  #[layout(Wrapper)]
    #[route("/")]
    Index {},
    #[end_layout]
  #[route("/:..route")]
  PageNotFound { route: Vec<String> },
}

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

// The server function at the endpoint "static_routes" will be called by the CLI to generate the list of static
// routes. You must explicitly set the endpoint to `"static_routes"` in the server function attribute instead of
// the default randomly generated endpoint.
#[server(endpoint = "static_routes", output = server_fn::codec::Json)]
async fn static_routes() -> Result<Vec<String>, ServerFnError> {
  // The `Routable` trait has a `static_routes` method that returns all static routes in the enum
  Ok(
    Route::static_routes()
      .iter()
      .map(ToString::to_string)
      .collect(),
  )
}

#[component]
pub fn App() -> Element {
  rsx! {
    document::Link { rel: "stylesheet", href: MAIN_CSS }
    document::Link {
      rel: "icon",
      href: asset!("/assets/images/icon-light.ico"),
      sizes: "32x32",
    }
    document::Link {
      rel: "icon",
      href: asset!("/assets/images/icon-light.ico"),
      sizes: "32x32",
      media: "prefers-color-scheme: light",
    }
    document::Link {
      rel: "icon",
      href: asset!("/assets/images/icon-dark.ico"),
      sizes: "32x32",
      media: "prefers-color-scheme: dark",
    }
    document::Link {
      rel: "icon",
      r#type: "image/svg+xml",
      href: asset!("/assets/images/icon-light.svg"),
    }
    document::Link {
      rel: "icon",
      r#type: "image/svg+xml",
      href: asset!("/assets/images/icon-light.svg"),
      media: "prefers-color-scheme: light",
    }
    document::Link {
      rel: "icon",
      r#type: "image/svg+xml",
      href: asset!("/assets/images/icon-dark.svg"),
      media: "prefers-color-scheme: dark",
    }

    Router::<Route> {}
  }
}
