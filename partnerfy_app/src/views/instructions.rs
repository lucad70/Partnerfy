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
                    }
                }
            }

            // Main Content
            div { style: "max-width: 900px; margin: 0 auto;",
                h1 { style: "font-size: 2.5rem; font-weight: 700; margin-bottom: 16px; color: #00090C;",
                    "Instructions"
                }
                p { style: "font-size: 1.125rem; color: #666; margin-bottom: 48px; line-height: 1.6;",
                    "Partnerfy provides two workflows for working with Simplicity contracts on Liquid Testnet: Multisig (P2MS) and Voucher (P2MS with Covenant)."
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

                // P2MS Instructions
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "Multisig (P2MS) Workflow"
                    }
                    p { style: "color: #666; margin-bottom: 24px; line-height: 1.6;",
                        "Create a 2-of-3 multisig contract where funds can be spent with signatures from any 2 of 3 participants."
                    }
                    div { style: "display: flex; flex-direction: column; gap: 24px;",
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "0. Generate P2MS Simplicity Source File"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the output path for your .simf file and provide three 32-byte public keys (64 hex characters each) for the three participants. Click 'Generate p2ms.simf File' to create the Simplicity source file."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "1. Compile Simplicity Source (Optional)"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the path to your .simf file and click 'Compile .simf File'. The compiled program (base64) will be displayed. You can also paste a pre-compiled program directly in the next step."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "2. Create P2MS Contract Address"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Paste the compiled Simplicity program (base64) and click 'Create Contract Address'. The app will generate a contract address and CMR (Contract Merkle Root) that you can use to receive funds."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "3. Fund Contract Address via Faucet"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the amount you want to request (default: 0.001 L-BTC) and click 'Fund via Faucet'. The app will automatically request funds from the Liquid Testnet faucet and display the funding transaction ID and VOUT."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "4. Create Spending PSET"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the destination address and amount you want to spend. Provide the internal key (Taproot key, default provided) and click 'Create and Update PSET'. The app will create a PSET ready for signing."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "5. Sign and Finalize Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Provide the witness file path (.wit) and at least 2 of the 3 private keys corresponding to the public keys in your contract. Click 'Sign and Finalize Transaction' to generate signatures, update the witness file, and finalize the PSET."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "6. Broadcast Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Once the transaction is finalized, click 'Broadcast Transaction' to send it to the Liquid Network. You'll receive a transaction ID and a link to view it on the Blockstream explorer."
                            }
                        }
                    }
                }

                // Voucher Instructions
                div { class: "panel-section", style: "margin-bottom: 32px;",
                    h2 { style: "font-size: 1.75rem; font-weight: 600; margin-bottom: 16px; color: #00090C;",
                        "Voucher (P2MS with Covenant) Workflow"
                    }
                    p { style: "color: #666; margin-bottom: 24px; line-height: 1.6;",
                        "Create a 2-of-3 multisig contract with a covenant that enforces exactly 3 outputs: payment, recursive covenant (change), and fee. This ensures that change automatically returns to the same covenant."
                    }
                    div { style: "display: flex; flex-direction: column; gap: 24px;",
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "0. Generate Voucher Simplicity Source File"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the output path for your .simf file and provide three 32-byte public keys (64 hex characters each) for the three participants. Click 'Generate cov_p2ms.simf File' to create the Simplicity source file with covenant structure."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "1. Compile Simplicity Source (Optional)"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the path to your .simf file and click 'Compile .simf File'. The compiled program (base64) will be displayed. You can also paste a pre-compiled program directly in the next step."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "2. Create Voucher Contract Address"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Paste the compiled Simplicity program (base64) and click 'Create Contract Address'. The app will generate a contract address and CMR. This covenant enforces 3 outputs: payment, recursive covenant, and fee."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "3. Fund Contract Address via Faucet"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the amount you want to request (default: 0.001 L-BTC) and click 'Fund via Faucet'. The app will automatically request funds from the Liquid Testnet faucet and display the funding transaction ID and VOUT."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "4. Create Spending PSET"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Enter the destination address and amount you want to spend. The covenant requires exactly 3 outputs: Output 0 (payment), Output 1 (recursive covenant/change), and Output 2 (fee). Provide the internal key and click 'Create and Update PSET'. The app will verify the structure matches the covenant requirements."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "5. Sign and Finalize Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Provide the witness file path (.wit) and at least 2 of the 3 private keys corresponding to the public keys in your contract. Click 'Sign and Finalize Transaction' to generate signatures, update the witness file, and finalize the PSET. The covenant will verify the 3-output structure during finalization."
                            }
                        }
                        div {
                            h3 { style: "font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; color: #00090C;",
                                "6. Broadcast Transaction"
                            }
                            p { style: "color: #666; margin-bottom: 8px; line-height: 1.6;",
                                "Once the transaction is finalized, click 'Broadcast Transaction' to send it to the Liquid Network. The covenant ensures that change (Output 1) automatically returns to the same covenant, maintaining the spending restrictions."
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
                        li { "Store private keys securely and encrypted locally - never share them" }
                        li { "Ensure private keys match the public keys in your contract (privkey_1 → pk1, privkey_2 → pk2, privkey_3 → pk3)" }
                        li { "For 2-of-3 multisig, you need at least 2 valid signatures from the 3 participants" }
                        li { "Signatures are PSET-specific - if you modify the PSET after signing, you must sign again" }
                        li { "For Voucher contracts, ensure the spending transaction has exactly 3 outputs: payment, recursive covenant, and fee" }
                        li { "Always test on Liquid Testnet first before using mainnet" }
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
                div { style: "text-align: center; margin-top: 48px; padding: 32px; display: flex; gap: 16px; justify-content: center;",
                    Link {
                        to: crate::Route::P2MSPage {},
                        class: "button",
                        style: "font-size: 1.125rem; padding: 16px 32px;",
                        "Multisig →"
                    }
                    Link {
                        to: crate::Route::VoucherPage {},
                        class: "button",
                        style: "font-size: 1.125rem; padding: 16px 32px; background-color: transparent; border: 2px solid #00090C;",
                        "Voucher"
                    }
                }
            }
        }
    }
}


