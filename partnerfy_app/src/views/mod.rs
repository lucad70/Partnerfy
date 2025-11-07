//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod landing;
pub use landing::Landing;

mod instructions;
pub use instructions::Instructions;

mod p2ms;
pub use p2ms::P2MS;

mod voucher;
pub use voucher::Voucher;

mod navbar;
pub use navbar::Navbar;
