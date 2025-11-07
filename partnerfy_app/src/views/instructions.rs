//! Instructions page for Partnerfy

use dioxus::prelude::*;

#[component]
pub fn Instructions() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #ffffff; padding: 24px;",
            // Header
            header { style: "border-bottom: 2px solid #00090C; margin-bottom: 40px;",
                div { style: "max-width: 1200px; margin: 0 auto; padding: 16px 24px; display: flex; align-items: center; justify-content: space-between;",
                    div { style: "display: flex; align-items: center; gap: 8px;",
                        Link {
                            to: crate::Route::LandingPage {},
                            style: "text-decoration: none; color: #00090C;",
                            div { style: "display: flex; align-items: center; gap: 8px;",
                                div { style: "width: 32px; height: 32px; background-color: #00090C; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                                    "🔒"
                                }
                                span { style: "font-size: 1.25rem; font-weight: 700;", "Partnerfy" }
                            }
                        }
                    }
                    nav {
                        Link {
                            to: crate::Route::LandingPage {},
                            style: "text-decoration: none; color: #00090C; font-size: 0.9rem; margin-right: 24px;",
                            "← Back"
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

            // Main Content
            div { style: "max-width: 900px; margin: 0 auto;",
                h1 { style: "font-size: 2.5rem; font-weight: 700; margin-bottom: 16px; color: #00090C;",
                    "Instructions"
                }
                p { style: "font-size: 1.125rem; color: #666; margin-bottom: 48px; line-height: 1.6;",
                    "Follow these steps to use Partnerfy for managing covenant-based vouchers on Liquid Testnet."
                }

                // Prerequisites
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "Prerequisites"
                    }
                    ul { class: "rules-list",
                        li { "Install and run Elements Core (elementsd) on Liquid Testnet" }
                        li { "Install SimplicityHL compiler (simc) for covenant compilation" }
                        li { "Install hal-simplicity for covenant info and witness generation" }
                        li { "Get testnet LBTC from the Liquid Testnet faucet" }
                    }
                }

                // Promoter Instructions
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "For Promoters"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 24px;",
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "1. Compile Covenant"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Write your Simplicity covenant in voucher.simf, then compile it:"
                            }
                            pre { style: "background-color: #f5f5f5; padding: 16px; border-radius: 5px; font-family: 'Roboto Mono', monospace; font-size: 0.9rem; overflow-x: auto;",
                                "simc voucher.simf -o voucher.base64"
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "2. Load Covenant Info"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "In the Promoter panel, click 'Load Covenant Info' to extract the covenant address."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "3. Fund Covenant Pool"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the funding amount and click 'Fund Covenant' to send LBTC to the covenant address."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "4. Issue Vouchers"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter comma-separated voucher amounts (e.g., '0.01, 0.01, 0.01') and click 'Create Vouchers' to split the funding into individual vouchers."
                            }
                        }
                    }
                }

                // Participant Instructions
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "For Participants"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 24px;",
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "1. Import Voucher"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Click 'Import Voucher' and provide the voucher UTXO information (txid:vout) and covenant details from the promoter."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "2. Select Voucher & Partner"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Select a voucher from your list, then enter the partner's P2PKH address where you want to redeem."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "3. Build & Sign Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the redemption amount and click 'Redeem Voucher'. The app will build a transaction with outputs to the partner and change back to the covenant. Sign the transaction with your witness."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "4. Send to Partner"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Send the signed transaction hex to the partner for validation and co-signature."
                            }
                        }
                    }
                }

                // Partner Instructions
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "For Partners"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 24px;",
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "1. Set Your Address"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter your P2PKH address in the Partner panel."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "2. Receive Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Receive the transaction hex from the participant and paste it into the 'Transaction Hex' field."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "3. Validate Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Click 'Validate Transaction' to verify it complies with covenant rules:"
                            }
                            ul { class: "rules-list", style: "margin-top: 8px;",
                                li { "Input references a valid voucher covenant UTXO" }
                                li { "At least one output pays to your partner address" }
                                li { "Change output is locked by the same covenant" }
                                li { "Transaction includes valid signatures" }
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "4. Broadcast Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "If validation passes, click 'Broadcast Transaction' to send it to the Liquid Network."
                            }
                        }
                    }
                }

                // Important Notes
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "Important Notes"
                    }
                    div { class: "info-box warning", style: "margin-bottom: 16px;",
                        p { style: "font-weight: 600; margin-bottom: 4px;", "⚠️ Always test on Liquid Testnet first" }
                        p { style: "font-size: 0.9rem;", "Never use mainnet until you've thoroughly tested all functionality." }
                    }
                    ul { class: "rules-list",
                        li { "Store private keys securely and encrypted locally" }
                        li { "Validate witness correctness before broadcasting transactions" }
                        li { "Keep an off-chain log of vouchers and redemption events" }
                        li { "Verify covenant recursion: each child UTXO's script should match the parent's covenant hash" }
                    }
                }

                // Resources
                div { class: "panel-section",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "Resources"
                    }
                    ul { class: "rules-list",
                        li {
                            a { href: "https://liquidtestnet.com/faucet", target: "_blank", style: "color: #00090C; text-decoration: underline;",
                                "Liquid Testnet Faucet"
                            }
                        }
                        li {
                            a { href: "https://blockstream.info/liquidtestnet", target: "_blank", style: "color: #00090C; text-decoration: underline;",
                                "Liquid Testnet Explorer"
                            }
                        }
                        li {
                            a { href: "https://elementsproject.org/en/doc/0.21.0.2/rpc/", target: "_blank", style: "color: #00090C; text-decoration: underline;",
                                "Elements RPC Documentation"
                            }
                        }
                        li {
                            a { href: "https://docs.liquid.net", target: "_blank", style: "color: #00090C; text-decoration: underline;",
                                "Simplicity Documentation"
                            }
                        }
                        li {
                            a { href: "https://github.com/Blockstream/hal-simplicity", target: "_blank", style: "color: #00090C; text-decoration: underline;",
                                "hal-simplicity GitHub"
                            }
                        }
                        li {
                            a { href: "https://github.com/ElementsProject/simplicity", target: "_blank", style: "color: #00090C; text-decoration: underline;",
                                "SimplicityHL Compiler"
                            }
                        }
                    }
                }

                // CTA
                div { style: "text-align: center; margin-top: 48px; padding: 32px;",
                    Link {
                        to: crate::Route::PromoterPage {},
                        class: "button",
                        style: "font-size: 1.125rem; padding: 16px 32px;",
                        "Get Started →"
                    }
                }
            }
        }
    }
}

