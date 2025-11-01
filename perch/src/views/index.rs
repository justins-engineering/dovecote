use crate::{Configuration, Create, api};
use dioxus::prelude::*;
use ory_kratos_client_wasm::apis::metadata_api::is_ready;

#[component]
pub fn Index() -> Element {
  let ready = use_resource(move || async move { is_ready(&Configuration::create()).await });

  let mut factor1 = use_signal(|| 1i32);
  let mut factor2 = use_signal(|| 1i32);

  #[cfg(feature = "api")]
  let answer = {
    let multiplication =
      use_resource(move || async move { api::multiply(factor1(), factor2()).await });
    let mut answer = use_signal(|| "= ?".to_string());
    use_effect(move || {
      answer.set(match &*multiplication.read() {
        Some(Ok(product)) => format!("= {product}"),
        Some(Err(err)) => err.to_string(),
        None => "= ?".to_string(),
      });
    });
    answer
  };

  return match &*ready.read() {
    Some(result) => match result {
      Ok(status) => {
        rsx! {
          p { {format!("{status:?}")} }
          div {
            display: "flex",
            flex: "1 1 auto",
            justify_content: "center",
            div {
              display: "grid",
              grid_template_columns: "2cm 50px 2cm 4cm",

              align_items: "center",
              justify_items: "center",

              // Top row
              div {
                button {
                  onclick: move |_| {
                      factor1 += 1;
                  },
                  "+"
                }
              }
              div {}
              div {
                button {
                  onclick: move |_| {
                      factor2 += 1;
                  },
                  "+"
                }
              }
              div {}

              // Middle row
              div { "{factor1}" }
              div { dangerous_inner_html: "&times;" }
              div { "{factor2}" }
              div { "{answer}" }

              // Bottom row
              div {
                button {
                  onclick: move |_| {
                      factor1 -= 1;
                  },
                  "-"
                }
              }
              div {}
              div {
                button {
                  onclick: move |_| {
                      factor2 -= 1;
                  },
                  "-"
                }
              }
              div {}
            }
          }
        }
      }
      Err(err) => {
        rsx! {
          p { {format!("{err:?}")} }
        }
      }
    },
    None => rsx! {
      p { "Waiting..." }
    },
  };
}
