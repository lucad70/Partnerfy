use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
/// 
/// This layout component wraps the UI of role-based routes in a common navbar.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div {
            id: "navbar",
            Link {
                to: Route::LandingPage {},
                style: "margin-right: auto;",
                "🏠 Home"
            }
            Link {
                to: Route::PromoterPage {},
                "Promoter"
            }
            Link {
                to: Route::ParticipantPage {},
                "Participant"
            }
            Link {
                to: Route::PartnerPage {},
                "Partner"
            }
            Link {
                to: Route::P2MSPage {},
                "P2MS"
            }
        }

        // The `Outlet` component is used to render the next component inside the layout.
        Outlet::<Route> {}
    }
}
