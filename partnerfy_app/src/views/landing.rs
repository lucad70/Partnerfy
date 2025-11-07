//! Landing page for Partnerfy

use dioxus::prelude::*;

#[component]
pub fn Landing() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #ffffff;",
            // Header
            header { style: "border-bottom: 2px solid #00090C;",
                div { style: "max-width: 1200px; margin: 0 auto; padding: 16px 24px; display: flex; align-items: center; justify-content: space-between;",
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        div { style: "width: 32px; height: 32px; background-color: #00090C; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                            "🔒"
                        }
                        span { style: "font-size: 1.25rem; font-weight: 700;", "Partnerfy" }
                    }
                    nav { style: "display: flex; align-items: center; gap: 24px;",
                        Link {
                            to: crate::Route::InstructionsPage {},
                            style: "text-decoration: none; color: #00090C; font-size: 0.9rem; transition: opacity 0.2s;",
                            "Instructions"
                        }
                        Link {
                            to: crate::Route::PromoterPage {},
                            class: "button",
                            style: "font-size: 0.9rem; padding: 8px 16px;",
                            "Get Started"
                        }
                    }
                }
            }

            // Hero Section
            section { style: "max-width: 1200px; margin: 0 auto; padding: 80px 24px; text-align: center;",
                div { style: "max-width: 800px; margin: 0 auto;",
                    div { style: "display: inline-flex; align-items: center; gap: 8px; padding: 4px 12px; border-radius: 9999px; background-color: rgba(0, 9, 12, 0.1); color: #00090C; font-size: 0.875rem; font-weight: 500; margin-bottom: 24px;",
                        "⚡"
                        "Powered by Liquid Network"
                    }
                    h1 { style: "font-size: 3rem; font-weight: 700; margin-bottom: 24px; line-height: 1.2; color: #00090C;",
                        "Simplify your Liquid Event"
                    }
                    p { style: "font-size: 1.25rem; color: #666; margin-bottom: 40px; max-width: 600px; margin-left: auto; margin-right: auto; line-height: 1.6;",
                        "Send restricted LBTC to event participants that can only be spent at pre-approved partners. Secure, transparent, and powered by covenants."
                    }
                    div { style: "display: flex; flex-direction: row; gap: 16px; align-items: center; justify-content: center; flex-wrap: wrap;",
                        Link {
                            to: crate::Route::PromoterPage {},
                            class: "button",
                            style: "font-size: 1.125rem; padding: 16px 32px;",
                            "Start as Promoter →"
                        }
                        Link {
                            to: crate::Route::ParticipantPage {},
                            class: "button",
                            style: "font-size: 1.125rem; padding: 16px 32px; background-color: transparent; border: 2px solid #00090C;",
                            "Start as Participant"
                        }
                    }
                }
            }

            // Features Section
            section { style: "max-width: 1200px; margin: 0 auto; padding: 80px 24px; border-top: 2px solid #00090C;",
                h2 { style: "font-size: 2.5rem; font-weight: 700; text-align: center; margin-bottom: 48px; color: #00090C;",
                    "Why Partnerfy?"
                }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 24px;",
                    div { class: "panel-section",
                        div { style: "width: 48px; height: 48px; background-color: rgba(0, 9, 12, 0.1); border-radius: 8px; display: flex; align-items: center; justify-content: center; margin-bottom: 16px;",
                            "🔒"
                        }
                        h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px;", "Restricted Spending" }
                        p { style: "color: #666; line-height: 1.6;",
                            "Vouchers can only be spent at pre-approved partner addresses using Simplicity covenant restrictions."
                        }
                    }
                    div { class: "panel-section",
                        div { style: "width: 48px; height: 48px; background-color: rgba(0, 9, 12, 0.1); border-radius: 8px; display: flex; align-items: center; justify-content: center; margin-bottom: 16px;",
                            "🔄"
                        }
                        h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px;", "Covenant Recursion" }
                        p { style: "color: #666; line-height: 1.6;",
                            "Change outputs automatically inherit the same covenant, ensuring perpetual compliance with spending rules."
                        }
                    }
                    div { class: "panel-section",
                        div { style: "width: 48px; height: 48px; background-color: rgba(0, 9, 12, 0.1); border-radius: 8px; display: flex; align-items: center; justify-content: center; margin-bottom: 16px;",
                            "⚡"
                        }
                        h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px;", "Liquid Testnet" }
                        p { style: "color: #666; line-height: 1.6;",
                            "Fast, confidential transactions on Liquid Testnet with low fees and instant settlement."
                        }
                    }
                }
            }

            // How It Works Section
            section { style: "max-width: 1200px; margin: 0 auto; padding: 80px 24px; border-top: 2px solid #00090C;",
                div { style: "max-width: 800px; margin: 0 auto;",
                    h2 { style: "font-size: 2.5rem; font-weight: 700; text-align: center; margin-bottom: 48px; color: #00090C;",
                        "How It Works"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 32px;",
                        div { style: "display: flex; gap: 16px;",
                            div { style: "flex-shrink: 0; width: 40px; height: 40px; background-color: #00090C; color: #ffffff; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700;",
                                "1"
                            }
                            div {
                                h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                    "Promoter Creates & Funds Covenant"
                                }
                                p { style: "color: #666; line-height: 1.6;",
                                    "Promoters compile a Simplicity covenant, fund it with LBTC, and split it into individual vouchers for participants."
                                }
                            }
                        }
                        div { style: "display: flex; gap: 16px;",
                            div { style: "flex-shrink: 0; width: 40px; height: 40px; background-color: #00090C; color: #ffffff; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700;",
                                "2"
                            }
                            div {
                                h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                    "Participant Redeems at Partner"
                                }
                                p { style: "color: #666; line-height: 1.6;",
                                    "Participants select a partner, build a redemption transaction, and sign it. Change automatically returns to the same covenant."
                                }
                            }
                        }
                        div { style: "display: flex; gap: 16px;",
                            div { style: "flex-shrink: 0; width: 40px; height: 40px; background-color: #00090C; color: #ffffff; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700; border: 2px solid #00090C;",
                                "3"
                            }
                            div {
                                h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                    "Partner Validates & Broadcasts"
                                }
                                p { style: "color: #666; line-height: 1.6;",
                                    "Partners verify the transaction complies with covenant rules, then broadcast it to the Liquid Network."
                                }
                            }
                        }
                    }
                }
            }

            // Footer
            footer { style: "border-top: 2px solid #00090C; margin-top: 80px;",
                div { style: "max-width: 1200px; margin: 0 auto; padding: 32px 24px;",
                    div { style: "display: flex; flex-direction: column; gap: 16px; align-items: center; justify-content: space-between;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            div { style: "width: 24px; height: 24px; background-color: #00090C; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                                "🔒"
                            }
                            span { style: "font-weight: 600;", "Partnerfy" }
                        }
                        p { style: "font-size: 0.875rem; color: #666; text-align: center;",
                            "Built with SimplicityHL on Liquid Network"
                        }
                    }
                }
            }
        }
    }
}

