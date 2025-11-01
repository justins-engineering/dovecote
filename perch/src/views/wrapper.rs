use crate::Route;
use crate::components::Logo;
use crate::components::{Footer, ThemeController};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdMenu, LdSquareChevronLeft, LdSquareChevronRight};

// #[component]
// pub fn Wrapper() -> Element {
//   rsx! {
//     Navbar {}
//     div { class: "flex-1", Outlet::<Route> {} }
//     ThemeController {}
//     Footer {}
//   }
// }

#[component]
pub fn Wrapper() -> Element {
  rsx! {
    div { id: "sidebar", class: "drawer lg:drawer-open",
      input {
        class: "drawer-toggle",
        id: "nav-drawer",
        r#type: "checkbox",
      }
      div { class: "drawer-content flex flex-col max-w-none",
        header { class: "bg-base-100 lg:bg-base-100/90 border-base-300 sticky top-0 z-10 border-b data-[at-top=true]:border-transparent lg:backdrop-blur-sm",
          nav { class: "flex items-center justify-between py-2",
            div { class: "flex items-center gap-2",
              label {
                class: "px-2 swap swap-rotate max-lg:hidden",
                aria_label: "toggle sidebar",
                input { r#type: "checkbox" }
                Icon {
                  class: "swap-on",
                  icon: LdSquareChevronRight,
                }
                Icon {
                  class: "swap-off",
                  icon: LdSquareChevronLeft,
                }
              }
              label {
                class: "px-2 lg:hidden",
                r#for: "nav-drawer",
                aria_label: "open sidebar",
                class: "btn btn-square btn-ghost btn-sm",
                Icon { icon: LdMenu }
              }
            }
            div { class: "max-lg:hidden",
              ul { class: "menu menu-horizontal gap-2 px-1 text-base" }
            }
            div { class: "space-x-2", ThemeController {} }
          }
        }
        // span { class: "py-4",
        //   label {
        //     class: "btn btn-primary drawer-button py-2",
        //     r#for: "nav-drawer",
        //     "Menu"
        //   }
        // }
        div { class: "flex-1 p-4 lg:p-12", Outlet::<Route> {} }
        Footer {}
      }
      div { class: "drawer-side",
        label {
          aria_label: "close sidebar",
          class: "drawer-overlay",
          r#for: "nav-drawer",
        }
        ul { class: "menu menu-lg bg-base-200 text-base-content min-h-full w-80 p-4",
          li {
            Link { to: Route::Index {}, aria_label: "Home", Logo {} }
          }
          li {
            Link { to: Route::Index {}, "Home" }
          }
                // li {
        //   Link { to: Route::SessionInfo {}, "Session Information" }
        // }
        // li {
        //   h2 { class: "menu-title", "Default User Interfaces" }
        //   ul {
        //     li {
        //       Link { to: Route::Verify {}, "Account Verification" }
        //     }
        //     li {
        //       Link { to: Route::Settings {}, "Account Settings" }
        //     }
        //     li { OryLogOut {} }
        //   }
        // }
        }
      }
    }
  }
}

// #[component]
// fn Navbar() -> Element {
//   rsx! {
//     div { class: "drawer lg:drawer-open",
//       input {
//         class: "drawer-toggle",
//         id: "nav-drawer",
//         r#type: "checkbox",
//       }
//       div { class: "drawer-content flex flex-col max-w-none p-4 lg:p-12",
//         span { class: "py-4",
//           label {
//             class: "btn btn-primary drawer-button lg:hidden py-2",
//             r#for: "nav-drawer",
//             "Menu"
//           }
//         }
//         Outlet::<Route> {}
//       }
//       div { class: "drawer-side",
//         label {
//           aria_label: "close sidebar",
//           class: "drawer-overlay",
//           r#for: "nav-drawer",
//         }
//         ul { class: "menu menu-lg bg-base-200 text-base-content min-h-full w-80 p-4",
//           li { class: "menu-title text-primary", "Welcome to Ory" }
//           li {
//             Link { to: Route::Index {}, "Home" }
//           }
//                 // li {
//         //   Link { to: Route::SessionInfo {}, "Session Information" }
//         // }
//         // li {
//         //   h2 { class: "menu-title", "Default User Interfaces" }
//         //   ul {
//         //     li {
//         //       Link { to: Route::Verify {}, "Account Verification" }
//         //     }
//         //     li {
//         //       Link { to: Route::Settings {}, "Account Settings" }
//         //     }
//         //     li { OryLogOut {} }
//         //   }
//         // }
//         }
//       }
//     }
//   }
// }
