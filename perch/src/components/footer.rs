use dioxus::prelude::*;
#[cfg(not(feature = "web"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "web")]
use web_time::{SystemTime, UNIX_EPOCH};
#[component]
pub fn Footer() -> Element {
  rsx! {
    footer { class: "footer sm:footer-horizontal footer-center bg-base-200 text-base-content p-4",
      aside {
        p {
          "Copyright © {1970 + (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()/31556926)} Justin's Engineering Services, LLC"
        }
      }
    }
  }
}
