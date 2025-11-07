//! Partnerfy - Covenant-based voucher management on Liquid Testnet
//!
//! A desktop app for issuing, managing, and redeeming Simplicity covenant-based vouchers.

use dioxus::prelude::*;

use views::{Promoter as PromoterPage, Participant as ParticipantPage, Partner as PartnerPage, P2MS as P2MSPage, Voucher as VoucherPage, Navbar, Landing as LandingPage, Instructions as InstructionsPage};
use app_core::{ElementsRPC, HalWrapper, Settings};

/// Define a components module that contains all shared components for our app.
mod components;
/// Define a views module that contains the UI for all Layouts and Routes for our app.
mod views;
/// Define core modules for Elements RPC, transaction building, and covenant handling.
mod app_core;

/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
/// 
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great for wrapping
    // many routes with a common UI like a navbar.
    // Landing page (no navbar)
    #[route("/")]
    LandingPage {},
    #[route("/instructions")]
    InstructionsPage {},
    // Role-based pages (with navbar)
    #[layout(Navbar)]
        #[route("/promoter")]
        PromoterPage {},
        #[route("/participant")]
        ParticipantPage {},
        #[route("/partner")]
        PartnerPage {},
        #[route("/p2ms")]
        P2MSPage {},
        #[route("/voucher")]
        VoucherPage {},
}

// We can import assets in dioxus with the `asset!` macro. This macro takes a path to an asset relative to the crate root.
// The macro returns an `Asset` type that will display as the path to the asset in the browser or a local path in desktop bundles.
const FAVICON: Asset = asset!("/assets/favicon.ico");
// The asset macro also minifies some assets like CSS and JS to make bundled smaller
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // The `launch` function is the main entry point for a dioxus app. It takes a component and renders it with the platform feature
    // you have enabled
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    use std::sync::Arc;
    
    // Initialize settings (default to Liquid Testnet)
    let settings = Settings::default();
    
    // Initialize RPC client context
    let rpc_client = match ElementsRPC::new(settings.clone()) {
        Ok(rpc) => Arc::new(rpc),
        Err(e) => {
            eprintln!("Failed to initialize RPC client: {}", e);
            return rsx! {
                div { "Failed to initialize RPC client. Please check your Elements node connection." }
            };
        }
    };
    provide_context(rpc_client);
    
    // Initialize hal-simplicity wrapper context
    let hal_wrapper = Arc::new(HalWrapper::new(None));
    provide_context(hal_wrapper);
    
    // Provide settings context
    provide_context(settings);
    
    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx! {
        // In addition to element and text (which we will see later), rsx can contain other components. In this case,
        // we are using the `document::Link` component to add a link to our favicon and main CSS file into the head of our app.
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        // The router component renders the route enum we defined above. It will handle synchronization of the URL and render
        // the layouts and components for the active route.
        Router::<Route> {}
    }
}
