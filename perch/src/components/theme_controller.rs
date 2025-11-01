use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdMoon, LdSun};
use dioxus_free_icons::Icon;
use wasm_theme::theme_toggle;

#[component]
pub fn ThemeController() -> Element {
  use_effect(move || {
    theme_toggle();
  });
  rsx! {
    label { class: "swap swap-rotate",
      input { name: "theme-toggle", r#type: "checkbox", value: "dark" }
      Icon { class: "swap-on", icon: LdSun }
      Icon { class: "swap-off", icon: LdMoon }
    }
  }
}
