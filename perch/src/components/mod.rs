//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component and an Echo component for fullstack apps to be used in our app.

mod logo;
pub use logo::Logo;

mod footer;
pub use footer::Footer;

mod theme_controller;
pub use theme_controller::ThemeController;

// mod ory_form_builder;
// pub use ory_form_builder::FormBuilder;

// mod ory_error;
// pub use ory_error::DisplayError;

// mod ory_log_out;
// pub use ory_log_out::OryLogOut;
