//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//!
//! The [`Home`] and [`Blog`] components will be rendered when the current route is [`Route::Home`] or [`Route::Blog`] respectively.
//!
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod index;
pub use index::Index;

mod error;
pub use error::PageNotFound;
// pub use error::ServerError;

mod wrapper;
pub use wrapper::Wrapper;

// mod settings;
// pub use settings::Settings;
// pub use settings::SettingsFlow;

// mod verification;
// pub use verification::VerificationFlow;
// pub use verification::Verify;

// mod recovery;
// pub use recovery::AccountRecovery;
// pub use recovery::RecoveryFlow;

// mod session;
// pub use session::SessionInfo;
