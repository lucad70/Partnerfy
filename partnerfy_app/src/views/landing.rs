//! Landing page for Partnerfy

use dioxus::prelude::*;

#[component]
pub fn Landing() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: var(--background);",
            // Header
            header { style: "border-bottom: 1px solid var(--border);",
                div { style: "max-width: 1200px; margin: 0 auto; padding: 16px 24px; display: flex; align-items: center; justify-content: space-between;",
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        div { style: "width: 32px; height: 32px; background-color: var(--accent); border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                            "🔒"
                        }
                        span { style: "font-size: 1.25rem; font-weight: 700; color: var(--foreground);", "Partnerfy" }
                    }
                    nav { style: "display: flex; align-items: center; gap: 24px;",
                        Link {
                            to: crate::Route::InstructionsPage {},
                            class: "nav-link",
                            "Instructions"
                        }
                    }
                }
            }

            // Hero Section
            section { style: "max-width: 1200px; margin: 0 auto; padding: 80px 24px; text-align: center;",
                div { style: "max-width: 800px; margin: 0 auto;",
                    div { style: "display: inline-flex; align-items: center; gap: 8px; padding: 4px 12px; border-radius: 9999px; background-color: rgba(45, 212, 191, 0.1); color: var(--accent); font-size: 0.875rem; font-weight: 500; margin-bottom: 24px;",
                        "Powered by Liquid Network"
                    }
                    h1 { style: "font-size: 3.5rem; font-weight: 700; margin-bottom: 24px; line-height: 1.2; color: var(--foreground);",
                        "Simplify your Liquid Event"
                    }
                    p { style: "font-size: 1.25rem; color: var(--muted-foreground); margin-bottom: 40px; max-width: 600px; margin-left: auto; margin-right: auto; line-height: 1.6;",
                        "Send restricted LBTC to event participants that can only be spent with pre-approved partners. Secure, transparent, and powered by SimplicityHL."
                    }
                    div { style: "display: flex; flex-direction: row; gap: 16px; align-items: center; justify-content: center; flex-wrap: wrap;",
                        Link {
                            to: crate::Route::P2MSPage {},
                            class: "button",
                            style: "font-size: 1.125rem; padding: 16px 32px;",
                            "Multisig →"
                        }
                        Link {
                            to: crate::Route::VoucherPage {},
                            class: "button outline",
                            style: "font-size: 1.125rem; padding: 16px 32px;",
                            "Voucher"
                        }
                    }
                }
            }

            // Features Section
            section { style: "max-width: 1200px; margin: 0 auto; padding: 80px 24px; border-top: 1px solid var(--border);",
                h2 { style: "font-size: 2.5rem; font-weight: 700; text-align: center; margin-bottom: 48px; color: var(--foreground);",
                    "Why Partnerfy?"
                }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 24px;",
                    div { class: "panel-section",
                        div { style: "width: 48px; height: 48px; background-color: rgba(45, 212, 191, 0.1); border-radius: 8px; display: flex; align-items: center; justify-content: center; margin-bottom: 16px;",
                            "🔒"
                        }
                        h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: var(--foreground);", "Restricted Spending" }
                        p { style: "color: var(--muted-foreground); line-height: 1.6;",
                            "Vouchers can only be spent with pre-approved partners using a Multisig arrangement."
                        }
                    }
                    div { class: "panel-section",
                        div { style: "width: 48px; height: 48px; background-color: rgba(45, 212, 191, 0.1); border-radius: 8px; display: flex; align-items: center; justify-content: center; margin-bottom: 16px;",
                            "🔄"
                        }
                        h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: var(--foreground);", "Covenant Recursion" }
                        p { style: "color: var(--muted-foreground); line-height: 1.6;",
                            "Change outputs automatically inherit the same covenant, ensuring ease of use for participants."
                        }
                    }
                }
            }

            // How It Works Section
            section { style: "max-width: 1200px; margin: 0 auto; padding: 80px 24px; border-top: 1px solid var(--border);",
                div { style: "max-width: 800px; margin: 0 auto;",
                    h2 { style: "font-size: 2.5rem; font-weight: 700; text-align: center; margin-bottom: 48px; color: var(--foreground);",
                        "How It Works"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 32px;",
                        div { style: "display: flex; gap: 16px;",
                            div { style: "flex-shrink: 0; width: 40px; height: 40px; background-color: var(--accent); color: var(--accent-foreground); border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700;",
                                "1"
                            }
                            div {
                                h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: var(--foreground);",
                                    "Promoter Creates & Funds Covenant"
                                }
                                p { style: "color: var(--muted-foreground); line-height: 1.6;",
                                    "Promoters compile a Simplicity covenant, fund it with LBTC, and split it into individual vouchers for participants."
                                }
                            }
                        }
                        div { style: "display: flex; gap: 16px;",
                            div { style: "flex-shrink: 0; width: 40px; height: 40px; background-color: var(--accent); color: var(--accent-foreground); border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700;",
                                "2"
                            }
                            div {
                                h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: var(--foreground);",
                                    "Participant Redeems at Partner"
                                }
                                p { style: "color: var(--muted-foreground); line-height: 1.6;",
                                    "Participants select a partner, build a redemption transaction, and sign it. Change automatically returns to the same covenant."
                                }
                            }
                        }
                        div { style: "display: flex; gap: 16px;",
                            div { style: "flex-shrink: 0; width: 40px; height: 40px; background-color: var(--accent); color: var(--accent-foreground); border-radius: 50%; display: flex; align-items: center; justify-content: center; font-weight: 700; border: 1px solid var(--border);",
                                "3"
                            }
                            div {
                                h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: var(--foreground);",
                                    "Partner Validates & Broadcasts"
                                }
                                p { style: "color: var(--muted-foreground); line-height: 1.6;",
                                    "Partners verify the transaction complies with covenant rules, then broadcast it to the Liquid Network."
                                }
                            }
                        }
                    }
                }
            }

            // Footer
            footer { style: "border-top: 1px solid var(--border); margin-top: 80px;",
                div { style: "max-width: 1200px; margin: 0 auto; padding: 32px 24px;",
                    div { style: "display: flex; flex-direction: column; gap: 16px; align-items: center; justify-content: space-between;",
                        div { style: "display: flex; align-items: center; gap: 8px;",
                            div { style: "width: 24px; height: 24px; background-color: var(--accent); border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                                "🔒"
                            }
                            span { style: "font-weight: 600; color: var(--foreground);", "Partnerfy" }
                        }
                        p { style: "font-size: 0.875rem; color: var(--muted-foreground); text-align: center;",
                            "Built with SimplicityHL on Liquid Network"
                        }
                    }
                }
            }
        }
    }
}

