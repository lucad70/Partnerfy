//! P2MS (Pay-to-Multisig) workflow page
//! 
//! Creates a Simplicity contract address for multisig, funds it via faucet, and manages spending

use crate::app_core::{ElementsRPC, HalWrapper};
use dioxus::prelude::*;
use std::sync::Arc;
use serde_json::{self, json};
use regex::Regex;
use std::fs;
use std::path::Path;

#[component]
pub fn P2MS() -> Element {
    let mut simf_file_path = use_signal(|| String::new());
    let mut required_sigs = use_signal(|| String::new());
    let mut pubkeys = use_signal(|| String::new());
    let mut contract_program_input = use_signal(|| String::new());
    let mut contract_address = use_signal(|| String::new());
    let mut contract_cmr = use_signal(|| String::new());
    let mut contract_program = use_signal(|| String::new());
    let mut funding_txid = use_signal(|| String::new());
    let mut funding_vout = use_signal(|| String::new());
    let mut funding_amount = use_signal(|| String::new());
    let mut spend_destination = use_signal(|| String::new());
    let mut spend_amount = use_signal(|| String::new());
    let mut signatures = use_signal(|| String::new());
    let mut final_tx_hex = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    
    let rpc_context = consume_context::<Arc<ElementsRPC>>();
    let hal_context = consume_context::<Arc<HalWrapper>>();

    // Compile .simf file
    let compile_simf = {
        let hal_context = hal_context.clone();
        move |_| {
            let hal_context = hal_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Compiling Simplicity source file...".to_string());
                
                let input_path = simf_file_path.read().clone();
                if input_path.is_empty() {
                    status_message.set("Please enter a path to the .simf file".to_string());
                    is_loading.set(false);
                    return;
                }
                
                if !Path::new(&input_path).exists() {
                    status_message.set(format!("File not found: {}", input_path));
                    is_loading.set(false);
                    return;
                }
                
                // Compile using simc (outputs to stdout)
                match hal_context.compile_simf(&input_path) {
                    Ok(program_base64) => {
                        contract_program_input.set(program_base64.clone());
                        status_message.set(format!(
                            "Compilation successful!\n\nInput: {}\n\nCompiled program (first 100 chars): {}...\n\nYou can now create the contract address.",
                            input_path, 
                            program_base64.chars().take(100).collect::<String>()
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Compilation failed: {}", e));
                    }
                }
                
                is_loading.set(false);
            });
        }
    };

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
                    status_message.set("Please enter a compiled Simplicity program (base64) or compile a .simf file first".to_string());
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
                
                let amount_str = funding_amount.read().clone();
                if amount_str.is_empty() {
                    status_message.set("Please enter a funding amount".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let amount: f64 = match amount_str.parse() {
                    Ok(a) if a > 0.0 => a,
                    _ => {
                        status_message.set("Please enter a valid positive amount".to_string());
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Use RPC to send funds to the address
                match rpc_context.send_to_address(&addr, amount).await {
                    Ok(txid) => {
                        funding_txid.set(txid.clone());
                        funding_vout.set("0".to_string());
                        status_message.set(format!(
                            "Funding successful!\n\nContract Address: {}\nAmount: {} L-BTC\nTransaction ID: {}\nVOUT: 0\n\nView on explorer: https://blockstream.info/liquidtestnet/tx/{}",
                            addr, amount, txid, txid
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Error funding address: {}\n\nMake sure your elementsd wallet has sufficient balance.", e));
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
                                let txid_pattern = Regex::new(r"transaction\s+([a-f0-9]{64})").unwrap();
                                
                                if let Some(captures) = txid_pattern.captures(&html_response) {
                                    if let Some(txid) = captures.get(1) {
                                        let txid_str = txid.as_str().to_string();
                                        funding_txid.set(txid_str.clone());
                                        funding_vout.set("0".to_string());
                                        funding_amount.set("0.001".to_string()); // Faucet sends 0.001 L-BTC
                                        
                                        status_message.set(format!(
                                            "Funding successful via faucet!\n\nContract Address: {}\nAmount: 0.001 L-BTC (100,000 sats)\nTransaction ID: {}\nVOUT: 0\n\nView on explorer: https://blockstream.info/liquidtestnet/tx/{}",
                                            addr, txid_str, txid_str
                                        ));
                                    } else {
                                        status_message.set(format!(
                                            "Faucet response received but could not extract transaction ID.\n\nResponse:\n{}",
                                            html_response.chars().take(500).collect::<String>()
                                        ));
                                    }
                                } else {
                                    let alt_pattern = Regex::new(r"txid[:\s]+([a-f0-9]{64})").unwrap();
                                    if let Some(captures) = alt_pattern.captures(&html_response) {
                                        if let Some(txid) = captures.get(1) {
                                            let txid_str = txid.as_str().to_string();
                                            funding_txid.set(txid_str.clone());
                                            funding_vout.set("0".to_string());
                                            funding_amount.set("0.001".to_string());
                                            status_message.set(format!(
                                                "Funding successful via faucet!\n\nContract Address: {}\nAmount: 0.001 L-BTC\nTransaction ID: {}\nVOUT: 0",
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

    let spend_p2ms = {
        let rpc_context = rpc_context.clone();
        let hal_context = hal_context.clone();
        move |_| {
            let rpc_context = rpc_context.clone();
            let hal_context = hal_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Creating spend transaction...".to_string());
                
                // Validate inputs
                let program = contract_program_input.read().clone();
                if program.is_empty() {
                    status_message.set("Please create the contract address first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let txid = funding_txid.read().clone();
                let vout_str = funding_vout.read().clone();
                if txid.is_empty() || vout_str.is_empty() {
                    status_message.set("Please fund the contract address first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let vout: u32 = vout_str.parse().unwrap_or(0);
                
                let destination = spend_destination.read().clone();
                if destination.is_empty() {
                    status_message.set("Please enter a destination address".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let amount: f64 = spend_amount.read().parse().unwrap_or(0.0);
                if amount <= 0.0 {
                    status_message.set("Please enter a valid amount".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Parse signatures (one per line or comma-separated)
                let sigs_str = signatures.read().clone();
                let sigs: Vec<String> = sigs_str
                    .lines()
                    .chain(sigs_str.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                if sigs.is_empty() {
                    status_message.set("Please enter at least one signature".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Create witness file
                let witness_json = json!({
                    "MAYBE_SIGS": sigs.iter().map(|s| json!({"Some": s})).collect::<Vec<_>>()
                });
                
                let witness_file = std::env::temp_dir().join("p2ms_witness.json");
                let witness_path = witness_file.to_string_lossy().to_string();
                
                if let Err(e) = fs::write(&witness_path, serde_json::to_string_pretty(&witness_json).unwrap()) {
                    status_message.set(format!("Failed to create witness file: {}", e));
                    is_loading.set(false);
                    return;
                }
                
                // Step 1: Create PSET
                status_message.set("Creating PSET...".to_string());
                let inputs = vec![(txid.clone(), vout)];
                let outputs = vec![(destination.clone(), amount)];
                
                match hal_context.create_pset(&program, &inputs, &outputs) {
                    Ok(pset_base64) => {
                        status_message.set("PSET created. Adding witness...".to_string());
                        
                        // Step 2: Add witness to PSET
                        match hal_context.add_witness_to_pset(&pset_base64, &witness_path) {
                            Ok(pset_with_witness) => {
                                status_message.set("Witness added. Finalizing transaction...".to_string());
                                
                                // Step 3: Finalize PSET
                                match hal_context.finalize_pset(&pset_with_witness) {
                                    Ok(tx_hex) => {
                                        final_tx_hex.set(tx_hex.clone());
                                        status_message.set(format!(
                                            "Transaction created successfully!\n\nTransaction Hex:\n{}\n\nYou can now broadcast this transaction.",
                                            tx_hex
                                        ));
                                    }
                                    Err(e) => {
                                        status_message.set(format!("Failed to finalize PSET: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                status_message.set(format!("Failed to add witness to PSET: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to create PSET: {}", e));
                    }
                }
                
                // Clean up witness file
                let _ = fs::remove_file(&witness_path);
                
                is_loading.set(false);
            });
        }
    };

    let broadcast_tx = {
        let rpc_context = rpc_context.clone();
        move |_| {
            let rpc_context = rpc_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Broadcasting transaction...".to_string());
                
                let tx_hex = final_tx_hex.read().clone();
                if tx_hex.is_empty() {
                    status_message.set("Please create a transaction first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                match rpc_context.send_raw_transaction(&tx_hex).await {
                    Ok(txid) => {
                        status_message.set(format!(
                            "Transaction broadcast successfully!\n\nTransaction ID: {}\n\nView on explorer: https://blockstream.info/liquidtestnet/tx/{}",
                            txid, txid
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to broadcast transaction: {}", e));
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
                h2 { "0. Compile Simplicity Source (Optional)" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Simplicity Source File (.simf)" }
                    input {
                        r#type: "text",
                        value: "{simf_file_path}",
                        oninput: move |evt| simf_file_path.set(evt.value().to_string()),
                        placeholder: "/path/to/p2ms.simf"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Enter the full path to your .simf source file"
                    }
                }
                
                button {
                    class: "button",
                    onclick: compile_simf,
                    disabled: is_loading(),
                    "Compile .simf File"
                }
                
            }
            
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
                        placeholder: "Paste compiled P2MS program base64 here or compile from .simf above"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Paste the base64-encoded compiled Simplicity program"
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
                        "Enter the amount you want to send to the contract address"
                    }
                }
                
                div { style: "display: flex; gap: 12px; margin-bottom: 16px;",
                    button {
                        class: "button",
                        onclick: fund_address,
                        disabled: is_loading() || contract_address().is_empty() || funding_amount().is_empty(),
                        "Fund via RPC"
                    }
                    button {
                        class: "button",
                        onclick: fund_via_faucet,
                        disabled: is_loading() || contract_address().is_empty(),
                        "Fund via Faucet (0.001 L-BTC)"
                    }
                }
                p { style: "font-size: 0.875rem; color: #666;",
                    "Fund via RPC: Send funds from your elementsd wallet (requires wallet balance).\nFund via Faucet: Request 0.001 L-BTC (100,000 sats) from the Liquid Testnet faucet."
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
                h2 { "3. Spend P2MS Funds" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Destination Address" }
                    input {
                        r#type: "text",
                        value: "{spend_destination}",
                        oninput: move |evt| spend_destination.set(evt.value().to_string()),
                        placeholder: "Enter destination address"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Address to send the funds to"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Amount (L-BTC)" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        min: "0",
                        value: "{spend_amount}",
                        oninput: move |evt| spend_amount.set(evt.value().to_string()),
                        placeholder: "0.0005"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Amount to send (must be less than or equal to the funded amount)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Signatures (one per line or comma-separated)" }
                    textarea {
                        rows: "4",
                        value: "{signatures}",
                        oninput: move |evt| signatures.set(evt.value().to_string()),
                        placeholder: "Enter signatures required for the m-of-n multisig"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Enter the signatures from the required signers (m signatures)"
                    }
                }
                
                button {
                    class: "button",
                    onclick: spend_p2ms,
                    disabled: is_loading() || funding_txid().is_empty() || contract_program_input().is_empty(),
                    "Create Spend Transaction"
                }
                
                if !final_tx_hex().is_empty() {
                    div { class: "info-box info", style: "margin-top: 16px;",
                        p { style: "font-weight: 600; margin-bottom: 8px;", "Transaction Hex:" }
                        textarea {
                            rows: "4",
                            readonly: true,
                            value: "{final_tx_hex}",
                            style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem; width: 100%;"
                        }
                        button {
                            class: "button",
                            onclick: broadcast_tx,
                            disabled: is_loading(),
                            style: "margin-top: 8px;",
                            "Broadcast Transaction"
                        }
                    }
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
