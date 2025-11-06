//! P2MS (Pay-to-Multisig) workflow page
//! 
//! Creates a Simplicity contract address for multisig, funds it, and manages spending

use crate::app_core::{ElementsRPC, HalWrapper};
use dioxus::prelude::*;
use std::sync::Arc;
use serde_json;

#[component]
pub fn P2MS() -> Element {
    let mut required_sigs = use_signal(|| String::new());
    let mut pubkeys = use_signal(|| String::new());
    let mut contract_program_input = use_signal(|| String::new());
    let mut contract_address = use_signal(|| String::new());
    let mut contract_cmr = use_signal(|| String::new());
    let mut contract_program = use_signal(|| String::new());
    let mut funding_amount = use_signal(|| String::new());
    let mut funding_txid = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    
    let rpc_context = consume_context::<Arc<ElementsRPC>>();
    let hal_context = consume_context::<Arc<HalWrapper>>();

    let create_contract_address = {
        let hal_context = hal_context.clone();
        move |_| {
            let hal_context = hal_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Creating P2MS contract address...".to_string());
                
                let m: u32 = required_sigs.read().parse().unwrap_or(0);
                let pubkeys_str = pubkeys.read().clone();
                
                if m == 0 {
                    status_message.set("Please enter the number of required signatures (m)".to_string());
                    is_loading.set(false);
                    return;
                }
                
                if pubkeys_str.is_empty() {
                    status_message.set("Please enter public keys (one per line or comma-separated)".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Parse public keys
                let keys: Vec<String> = pubkeys_str
                    .lines()
                    .chain(pubkeys_str.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                if keys.is_empty() {
                    status_message.set("No valid public keys found".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let n = keys.len() as u32;
                
                if m > n {
                    status_message.set(format!("Required signatures (m={}) cannot exceed total keys (n={})", m, n));
                    is_loading.set(false);
                    return;
                }
                
                // For now, we'll create a placeholder program
                // In a real implementation, you would use hal-simplicity to create a P2MS Simplicity program
                // This would involve creating a program that checks m-of-n signatures
                
                // Placeholder: Create a simple program representation
                // In practice, you'd call hal-simplicity to compile a P2MS Simplicity program
                let program_placeholder = format!("p2ms_{}of{}", m, n);
                
                // Check if user provided a compiled program
                let program = contract_program_input.read().clone();
                
                if !program.is_empty() {
                    // User provided a compiled program - use hal-simplicity to get the address
                    match hal_context.get_covenant_info(&program) {
                        Ok(info_str) => {
                            // Try to parse JSON response
                            if let Ok(info_json) = serde_json::from_str::<serde_json::Value>(&info_str) {
                                if let (Some(cmr), Some(addr)) = (
                                    info_json.get("cmr").and_then(|v| v.as_str()),
                                    info_json.get("liquid_testnet_address_unconf").and_then(|v| v.as_str())
                                ) {
                                    contract_cmr.set(cmr.to_string());
                                    contract_address.set(addr.to_string());
                                    contract_program.set(program.clone());
                                    status_message.set(format!(
                                        "P2MS Contract created successfully!\n\nType: {}-of-{} multisig\nCMR: {}\nAddress: {}\n\nPublic Keys ({}):\n{}",
                                        m, n, cmr, addr, n,
                                        keys.iter().enumerate().map(|(i, k)| format!("  {}. {}", i+1, k)).collect::<Vec<_>>().join("\n")
                                    ));
                                } else {
                                    // Fallback if JSON parsing works but structure is different
                                    contract_program.set(program.clone());
                                    status_message.set(format!(
                                        "Program processed. Response:\n{}\n\nPlease check the output for CMR and address information.\n\nPublic Keys ({}):\n{}",
                                        info_str, n,
                                        keys.iter().enumerate().map(|(i, k)| format!("  {}. {}", i+1, k)).collect::<Vec<_>>().join("\n")
                                    ));
                                }
                            } else {
                                // Not JSON, treat as plain text
                                contract_program.set(program.clone());
                                status_message.set(format!(
                                    "Program processed. hal-simplicity output:\n{}\n\nPublic Keys ({}):\n{}",
                                    info_str, n,
                                    keys.iter().enumerate().map(|(i, k)| format!("  {}. {}", i+1, k)).collect::<Vec<_>>().join("\n")
                                ));
                            }
                        }
                        Err(e) => {
                            status_message.set(format!(
                                "Error getting covenant info from hal-simplicity: {}\n\nPlease ensure:\n1. hal-simplicity is installed and in PATH\n2. The program is valid base64\n3. Try running: hal-simplicity simplicity info <your_program.base64>",
                                e
                            ));
                        }
                    }
                } else {
                    // No program provided - generate placeholder and guide user
                    let placeholder_addr = format!("p2ms_{}of{}_placeholder", m, n);
                    contract_address.set(placeholder_addr.clone());
                    contract_cmr.set(format!("cmr_{}of{}", m, n));
                    contract_program.set(program_placeholder.clone());
                    let cmr_val = format!("cmr_{}of{}", m, n);
                    status_message.set(format!(
                        "P2MS Contract parameters configured!\n\nType: {}-of-{} multisig\nAddress: {}\nCMR: {}\n\n⚠️  Note: This is a placeholder address.\n\nTo get a real Simplicity P2MS address:\n1. Create a .simf file with your P2MS logic\n2. Compile it: simc p2ms.simf -o p2ms.base64\n3. Paste the base64 program above and click 'Create Contract Address' again\n\nPublic Keys ({}):\n{}",
                        m, n, placeholder_addr, cmr_val, n,
                        keys.iter().enumerate().map(|(i, k)| format!("  {}. {}", i+1, k)).collect::<Vec<_>>().join("\n")
                    ));
                }
                
                is_loading.set(false);
            });
        }
    };

    let fund_address = {
        let rpc_context = rpc_context.clone();
        move |_| {
            let rpc_context = rpc_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Funding contract address...".to_string());
                
                let addr = contract_address.read().clone();
                if addr.is_empty() {
                    status_message.set("Please create the contract address first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let amount: f64 = funding_amount.read().parse().unwrap_or(0.0);
                if amount <= 0.0 {
                    status_message.set("Please enter a valid funding amount".to_string());
                    is_loading.set(false);
                    return;
                }
                
                match rpc_context.send_to_address(&addr, amount).await {
                    Ok(txid) => {
                        funding_txid.set(txid.clone());
                        status_message.set(format!(
                            "Funding transaction sent successfully!\n\nTransaction ID: {}\nAmount: {} L-BTC\nAddress: {}\n\nView on explorer: https://blockstream.info/liquidtestnet/tx/{}",
                            txid, amount, addr, txid
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Error funding address: {}", e));
                    }
                }
                
                is_loading.set(false);
            });
        }
    };

    rsx! {
        div { id: "p2ms-panel",
            h1 { style: "font-size: 2rem; margin-bottom: 24px;", "P2MS Workflow" }
            
            div { class: "panel-section",
                h2 { "1. Create P2MS Contract Address" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Required Signatures (m)" }
                    input {
                        r#type: "number",
                        min: "1",
                        value: "{required_sigs}",
                        oninput: move |evt| required_sigs.set(evt.value().to_string()),
                        placeholder: "e.g., 2 for 2-of-3 multisig"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Number of signatures required to spend (m)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Keys (n total keys)" }
                    textarea {
                        rows: "5",
                        value: "{pubkeys}",
                        oninput: move |evt| pubkeys.set(evt.value().to_string()),
                        placeholder: "Enter public keys, one per line or comma-separated"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Enter all public keys that can sign. Total number of keys = n"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Compiled Simplicity Program (base64) - Optional" }
                    textarea {
                        rows: "4",
                        value: "{contract_program_input}",
                        oninput: move |evt| contract_program_input.set(evt.value().to_string()),
                        placeholder: "Paste compiled P2MS program base64 here, or leave empty for placeholder"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "If you have a compiled Simplicity P2MS program, paste it here to get the real covenant address"
                    }
                }
                
                button {
                    class: "button",
                    onclick: create_contract_address,
                    disabled: is_loading(),
                    "Create Contract Address"
                }
                
                if !contract_address().is_empty() {
                    div { class: "info-box info", style: "margin-top: 16px;",
                        p { style: "font-weight: 600; margin-bottom: 8px;", "Contract Address:" }
                        p { style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem; word-break: break-all;",
                            "{contract_address}"
                        }
                        if !contract_cmr().is_empty() {
                            p { style: "font-weight: 600; margin-top: 8px; margin-bottom: 4px;", "CMR:" }
                            p { style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem;",
                                "{contract_cmr}"
                            }
                        }
                    }
                }
            }
            
            div { class: "panel-section",
                h2 { "2. Fund Contract Address" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Contract Address" }
                    input {
                        value: "{contract_address}",
                        oninput: move |evt| contract_address.set(evt.value().to_string()),
                        placeholder: "Will be auto-filled after creating contract",
                        readonly: !contract_address().is_empty()
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Funding Amount (L-BTC)" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        min: "0",
                        value: "{funding_amount}",
                        oninput: move |evt| funding_amount.set(evt.value().to_string()),
                        placeholder: "0.01"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Amount to send to the contract address"
                    }
                }
                
                button {
                    class: "button",
                    onclick: fund_address,
                    disabled: is_loading() || contract_address().is_empty(),
                    "Fund Address"
                }
                
                if !funding_txid().is_empty() {
                    div { class: "info-box info", style: "margin-top: 16px;",
                        p { style: "font-weight: 600; margin-bottom: 8px;", "Funding Transaction ID:" }
                        p { style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem; word-break: break-all;",
                            "{funding_txid}"
                        }
                        p { style: "margin-top: 8px;",
                            a {
                                href: format!("https://blockstream.info/liquidtestnet/tx/{}", funding_txid()),
                                target: "_blank",
                                style: "color: #0066cc; text-decoration: underline;",
                                "View on Blockstream Explorer →"
                            }
                        }
                    }
                }
            }
            
            div { id: "spend-p2ms", class: "panel-section",
                h2 { "3. Spend P2MS (Placeholder)" }
                p { style: "color: #666;",
                    "This section will be implemented later for spending from the P2MS contract."
                }
            }
            
            if !status_message().is_empty() {
                div { class: "status-message",
                    pre { style: "white-space: pre-wrap; font-family: inherit;",
                        "{status_message}"
                    }
                }
            }
            
            if is_loading() {
                div { style: "text-align: center; padding: 16px;", "Loading..." }
            }
        }
    }
}

