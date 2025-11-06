//! P2MS (Pay-to-Multisig) workflow page
//! 
//! Creates a Simplicity contract address for multisig, funds it via faucet, and manages spending

use crate::app_core::{ElementsRPC, HalWrapper};
use dioxus::prelude::*;
use std::sync::Arc;
use serde_json;
use regex::Regex;

#[component]
pub fn P2MS() -> Element {
    let mut required_sigs = use_signal(|| String::new());
    let mut pubkeys = use_signal(|| String::new());
    let mut contract_program_input = use_signal(|| String::new());
    let mut contract_address = use_signal(|| String::new());
    let mut contract_cmr = use_signal(|| String::new());
    let mut contract_program = use_signal(|| String::new());
    let mut funding_txid = use_signal(|| String::new());
    let mut funding_vout = use_signal(|| String::new());
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
                let program = contract_program_input.read().clone();
                
                if program.is_empty() {
                    status_message.set("Please enter a compiled Simplicity program (base64)".to_string());
                    is_loading.set(false);
                    return;
                }
                
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
                
                // Call hal-simplicity to get covenant info
                match hal_context.get_covenant_info(&program) {
                    Ok(info_str) => {
                        // Parse JSON response
                        match serde_json::from_str::<serde_json::Value>(&info_str) {
                            Ok(info_json) => {
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
                                    status_message.set(format!(
                                        "Error: Could not extract CMR or address from hal-simplicity response.\n\nResponse:\n{}",
                                        serde_json::to_string_pretty(&info_json).unwrap_or_else(|_| info_str.clone())
                                    ));
                                }
                            }
                            Err(e) => {
                                status_message.set(format!(
                                    "Error parsing hal-simplicity JSON response: {}\n\nRaw output:\n{}",
                                    e, info_str
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        status_message.set(format!(
                            "Error calling hal-simplicity: {}\n\nPlease ensure:\n1. hal-simplicity is installed and in PATH\n2. The program is valid base64\n3. Try running: hal-simplicity simplicity simplicity info \"<your_program>\"",
                            e
                        ));
                    }
                }
                
                is_loading.set(false);
            });
        }
    };

    let fund_via_faucet = {
        move |_| {
            spawn(async move {
                is_loading.set(true);
                status_message.set("Funding contract address via Liquid Testnet faucet...".to_string());
                
                let addr = contract_address.read().clone();
                if addr.is_empty() {
                    status_message.set("Please create the contract address first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Call the Liquid Testnet faucet API
                let faucet_url = format!("https://liquidtestnet.com/faucet?address={}&action=lbtc", addr);
                
                match reqwest::Client::new().get(&faucet_url).send().await {
                    Ok(response) => {
                        match response.text().await {
                            Ok(html_response) => {
                                // Parse the HTML response to extract transaction ID
                                // Look for pattern like "transaction abc123..." or "txid: abc123"
                                let txid_pattern = Regex::new(r"transaction\s+([a-f0-9]{64})").unwrap();
                                
                                if let Some(captures) = txid_pattern.captures(&html_response) {
                                    if let Some(txid) = captures.get(1) {
                                        let txid_str = txid.as_str().to_string();
                                        funding_txid.set(txid_str.clone());
                                        funding_vout.set("0".to_string()); // Faucet typically sends to vout 0
                                        
                                        status_message.set(format!(
                                            "Funding successful via faucet!\n\nContract Address: {}\nTransaction ID: {}\nVOUT: 0\n\nView on explorer: https://blockstream.info/liquidtestnet/tx/{}",
                                            addr, txid_str, txid_str
                                        ));
                                    } else {
                                        status_message.set(format!(
                                            "Faucet response received but could not extract transaction ID.\n\nResponse:\n{}",
                                            html_response.chars().take(500).collect::<String>()
                                        ));
                                    }
                                } else {
                                    // Try alternative patterns
                                    let alt_pattern = Regex::new(r"txid[:\s]+([a-f0-9]{64})").unwrap();
                                    if let Some(captures) = alt_pattern.captures(&html_response) {
                                        if let Some(txid) = captures.get(1) {
                                            let txid_str = txid.as_str().to_string();
                                            funding_txid.set(txid_str.clone());
                                            funding_vout.set("0".to_string());
                                            status_message.set(format!(
                                                "Funding successful via faucet!\n\nContract Address: {}\nTransaction ID: {}\nVOUT: 0",
                                                addr, txid_str
                                            ));
                                        } else {
                                            status_message.set(format!(
                                                "Faucet response received but could not extract transaction ID.\n\nResponse:\n{}",
                                                html_response.chars().take(500).collect::<String>()
                                            ));
                                        }
                                    } else {
                                        status_message.set(format!(
                                            "Faucet response received but could not find transaction ID in response.\n\nResponse preview:\n{}",
                                            html_response.chars().take(500).collect::<String>()
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                status_message.set(format!("Error reading faucet response: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        status_message.set(format!("Error calling faucet API: {}\n\nURL: {}", e, faucet_url));
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
                    label { "Compiled Simplicity Program (base64) - Required" }
                    textarea {
                        rows: "6",
                        value: "{contract_program_input}",
                        oninput: move |evt| contract_program_input.set(evt.value().to_string()),
                        placeholder: "Paste compiled P2MS program base64 here"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Paste the base64-encoded compiled Simplicity program. Use: hal-simplicity simplicity simplicity info \"<program>\" to verify"
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
                h2 { "2. Fund Contract Address via Faucet" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Contract Address" }
                    input {
                        value: "{contract_address}",
                        oninput: move |evt| contract_address.set(evt.value().to_string()),
                        placeholder: "Will be auto-filled after creating contract",
                        readonly: !contract_address().is_empty()
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "This address will be funded with 100,000 sats (0.001 L-BTC) from the Liquid Testnet faucet"
                    }
                }
                
                button {
                    class: "button",
                    onclick: fund_via_faucet,
                    disabled: is_loading() || contract_address().is_empty(),
                    "Fund via Faucet"
                }
                
                if !funding_txid().is_empty() {
                    div { class: "info-box info", style: "margin-top: 16px;",
                        p { style: "font-weight: 600; margin-bottom: 8px;", "Funding Transaction ID:" }
                        p { style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem; word-break: break-all;",
                            "{funding_txid}"
                        }
                        p { style: "font-weight: 600; margin-top: 8px; margin-bottom: 4px;", "VOUT:" }
                        p { style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem;",
                            "{funding_vout}"
                        }
                        p { style: "margin-top: 8px;",
                            a {
                                href: format!("https://blockstream.info/liquidtestnet/tx/{}", funding_txid()),
                                target: "_blank",
                                style: "color: #0066cc; text-decoration: underline;",
                                "View on Blockstream Explorer →"
                            }
                        }
                        p { style: "margin-top: 8px; font-weight: 600;",
                            "UTXO Reference: {funding_txid}:{funding_vout}"
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
