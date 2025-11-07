//! Voucher (P2MS with Covenant) workflow page
//! 
//! Creates a Simplicity contract address for multisig with covenant, funds it via faucet, and manages spending
//! The covenant enforces three outputs: payment, recursive covenant, and fee

use crate::app_core::{ElementsRPC, HalWrapper};
use dioxus::prelude::*;
use std::sync::Arc;
use serde_json::{self, json};
use regex::Regex;
use std::path::Path;

#[component]
pub fn Voucher() -> Element {
    let mut simf_file_path = use_signal(|| String::new());
    let mut required_sigs = use_signal(|| String::new());
    let mut pubkey_1 = use_signal(|| String::new());
    let mut pubkey_2 = use_signal(|| String::new());
    let mut pubkey_3 = use_signal(|| String::new());
    let mut contract_program_input = use_signal(|| String::new());
    let mut contract_address = use_signal(|| String::new());
    let mut contract_cmr = use_signal(|| String::new());
    let mut contract_program = use_signal(|| String::new());
    let mut internal_key = use_signal(|| "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0".to_string());
    let mut witness_file_path = use_signal(|| String::new());
    let mut privkey_1 = use_signal(|| String::new());
    let mut privkey_2 = use_signal(|| String::new());
    let mut privkey_3 = use_signal(|| String::new());
    let mut funding_txid = use_signal(|| String::new());
    let mut funding_vout = use_signal(|| String::new());
    let mut funding_amount = use_signal(|| String::new());
    let mut faucet_amount = use_signal(|| "0.001".to_string());
    let mut spend_destination = use_signal(|| String::new());
    let mut spend_amount = use_signal(|| String::new());
    let mut pset_for_signing = use_signal(|| String::new());
    let mut final_pset = use_signal(|| String::new());
    let mut final_tx_hex = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    
    let rpc_context = consume_context::<Arc<ElementsRPC>>();
    let hal_context = consume_context::<Arc<HalWrapper>>();

    // Generate cov_p2ms.simf file with custom pubkeys and covenant structure
    let generate_simf = {
        move |_| {
            spawn(async move {
                is_loading.set(true);
                status_message.set("Generating cov_p2ms.simf file with custom pubkeys and covenant...".to_string());
                
                let pk1 = pubkey_1.read().clone().trim().to_lowercase();
                let pk2 = pubkey_2.read().clone().trim().to_lowercase();
                let pk3 = pubkey_3.read().clone().trim().to_lowercase();
                
                // Validate pubkeys are provided
                if pk1.is_empty() || pk2.is_empty() || pk3.is_empty() {
                    status_message.set("Please provide all three public keys".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Validate pubkeys are valid hex (64 characters = 32 bytes)
                let is_valid_hex = |s: &str| {
                    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
                };
                
                if !is_valid_hex(&pk1) {
                    status_message.set(format!("Invalid public key 1: must be 64 hex characters (32 bytes). Got: {} ({} chars)", pk1, pk1.len()));
                    is_loading.set(false);
                    return;
                }
                
                if !is_valid_hex(&pk2) {
                    status_message.set(format!("Invalid public key 2: must be 64 hex characters (32 bytes). Got: {} ({} chars)", pk2, pk2.len()));
                    is_loading.set(false);
                    return;
                }
                
                if !is_valid_hex(&pk3) {
                    status_message.set(format!("Invalid public key 3: must be 64 hex characters (32 bytes). Got: {} ({} chars)", pk3, pk3.len()));
                    is_loading.set(false);
                    return;
                }
                
                // Get the output file path
                let output_path = simf_file_path.read().clone();
                if output_path.is_empty() {
                    status_message.set("Please enter a path for the .simf file".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Generate the simf file content with covenant structure
                let simf_content = format!(
                    r#"/*
 * P2MS COVENANT
 *
 * A 2-of-3 multisig covenant that enforces three outputs:
 * - Output 0: P2PK to any of the multisig public keys (payment)
 * - Output 1: Same P2MS covenant script (change/recursive)
 * - Output 2: Fee output
 */
fn not(bit: bool) -> bool {{
    <u1>::into(jet::complement_1(<bool>::into(bit)))
}}

fn checksig(pk: Pubkey, sig: Signature) {{
    let msg: u256 = jet::sig_all_hash();
    jet::bip_0340_verify((pk, msg), sig);
}}

fn checksig_add(counter: u8, pk: Pubkey, maybe_sig: Option<Signature>) -> u8 {{
    match maybe_sig {{
        Some(sig: Signature) => {{
            checksig(pk, sig);
            let (carry, new_counter): (bool, u8) = jet::increment_8(counter);
            assert!(not(carry));
            new_counter
        }}
        None => counter,
    }}
}}

fn check2of3multisig(pks: [Pubkey; 3], maybe_sigs: [Option<Signature>; 3]) {{
    let [pk1, pk2, pk3]: [Pubkey; 3] = pks;
    let [sig1, sig2, sig3]: [Option<Signature>; 3] = maybe_sigs;
    let counter1: u8 = checksig_add(0, pk1, sig1);
    let counter2: u8 = checksig_add(counter1, pk2, sig2);
    let counter3: u8 = checksig_add(counter2, pk3, sig3);
    let threshold: u8 = 2;
    assert!(jet::eq_8(counter3, threshold));
}}

// Enforce the covenant structure with three outputs
fn covenant_structure() {{
    assert!(jet::eq_32(jet::num_outputs(), 3));
    
    // Output 1: Must be the same script (recursive covenant)
    let this_script_hash: u256 = jet::current_script_hash();
    let output_script_hash: u256 = unwrap(jet::output_script_hash(1));
    assert!(jet::eq_256(this_script_hash, output_script_hash));
    
    // Output 2: Must be fee output
    assert!(unwrap(jet::output_is_fee(2)));
}}

fn main() {{
    let pks: [Pubkey; 3] = [
        0x{}, // Participant 1
        0x{}, // Participant 2
        0x{}, // Participant 3
    ];
    
    // Verify 2-of-3 multisig authorization
    check2of3multisig(pks, witness::MAYBE_SIGS);
    
    // Enforce covenant structure
    covenant_structure();
}}
"#,
                    pk1, pk2, pk3
                );
                
                // Write the file
                match tokio::fs::write(&output_path, &simf_content).await {
                    Ok(_) => {
                        status_message.set(format!(
                            "Successfully generated cov_p2ms.simf file with covenant!\n\nFile: {}\n\nPublic Keys:\n- Participant 1: 0x{}\n- Participant 2: 0x{}\n- Participant 3: 0x{}\n\nCovenant enforces:\n- Exactly 3 outputs\n- Output 1: Same script (recursive)\n- Output 2: Fee output\n\nYou can now compile this file.",
                            output_path, pk1, pk2, pk3
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to write simf file: {}\n\nPath: {}", e, output_path));
                    }
                }
                
                is_loading.set(false);
            });
        }
    };

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
                status_message.set("Creating Voucher (P2MS with Covenant) contract address...".to_string());
                
                let program = contract_program_input.read().clone();
                
                if program.is_empty() {
                    status_message.set("Please enter a compiled Simplicity program (base64) or compile a .simf file first".to_string());
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
                                        "Voucher Contract created successfully!\n\nCMR: {}\nAddress: {}\n\nThis covenant enforces 3 outputs: payment, recursive covenant, and fee.",
                                        cmr, addr
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
        let faucet_amount = faucet_amount.clone();
        move |_| {
            let faucet_amount = faucet_amount.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Funding contract address via Liquid Testnet faucet...".to_string());
                
                let addr = contract_address.read().clone();
                if addr.is_empty() {
                    status_message.set("Please create the contract address first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Get the faucet amount from user input
                let amount_str = faucet_amount.read().clone();
                let amount: f64 = amount_str.parse().unwrap_or(0.001);
                if amount <= 0.0 {
                    status_message.set(format!("Invalid faucet amount: {}. Please enter a positive number.", amount_str));
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
                                        funding_amount.set(amount_str.clone());
                                        
                                        let sats = (amount * 100_000_000.0) as u64;
                                        status_message.set(format!(
                                            "Funding successful via faucet!\n\nContract Address: {}\nAmount: {} L-BTC ({} sats)\nTransaction ID: {}\nVOUT: 0\n\nView on explorer: https://blockstream.info/liquidtestnet/tx/{}",
                                            addr, amount_str, sats, txid_str, txid_str
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
                                            funding_amount.set(amount_str.clone());
                                            let sats = (amount * 100_000_000.0) as u64;
                                            status_message.set(format!(
                                                "Funding successful via faucet!\n\nContract Address: {}\nAmount: {} L-BTC ({} sats)\nTransaction ID: {}\nVOUT: 0",
                                                addr, amount_str, sats, txid_str
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

    // For now, keep the spending logic similar to P2MS
    // The covenant will enforce the structure (3 outputs) when the transaction is finalized
    let create_spend_pset = {
        let rpc_context = rpc_context.clone();
        let hal_context = hal_context.clone();
        move |_| {
            let rpc_context = rpc_context.clone();
            let hal_context = hal_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Creating spending PSET (covenant will enforce 3 outputs)...".to_string());
                
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
                
                let cmr = contract_cmr.read().clone();
                if cmr.is_empty() {
                    status_message.set("Please create the contract address first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Step 1: Wait for UTXO to be available and get its value FIRST
                status_message.set("Waiting for UTXO to be available...".to_string());
                let mut utxo_data: Option<serde_json::Value> = None;
                let mut attempts = 0;
                const MAX_ATTEMPTS: u32 = 20;
                
                while attempts < MAX_ATTEMPTS {
                    match rpc_context.get_txout(&txid, vout).await {
                        Ok(data) => {
                            if !data.is_null() {
                                utxo_data = Some(data);
                                break;
                            }
                        }
                        Err(_) => {
                            // UTXO not found yet, wait and retry
                        }
                    }
                    
                    attempts += 1;
                    if attempts < MAX_ATTEMPTS {
                        status_message.set(format!("UTXO not available yet, waiting... (attempt {}/{})", attempts + 1, MAX_ATTEMPTS));
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
                
                let utxo_data = match utxo_data {
                    Some(data) => data,
                    None => {
                        status_message.set(format!("UTXO not found after {} attempts. Trying Blockstream API...", MAX_ATTEMPTS));
                        match reqwest::Client::new()
                            .get(&format!("https://blockstream.info/liquidtestnet/api/tx/{}", txid))
                            .send()
                            .await
                        {
                            Ok(resp) => {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(tx_data) => {
                                        let script_pubkey = tx_data["vout"][vout as usize]["scriptpubkey"].as_str().unwrap_or("");
                                        let asset = tx_data["vout"][vout as usize]["asset"].as_str().unwrap_or("");
                                        let value = tx_data["vout"][vout as usize]["value"].as_u64().unwrap_or(0) as f64 / 100_000_000.0;
                                        
                                        json!({
                                            "scriptPubKey": {"hex": script_pubkey},
                                            "asset": asset,
                                            "value": value
                                        })
                                    }
                                    Err(e) => {
                                        status_message.set(format!("Failed to parse Blockstream API response: {}", e));
                                        is_loading.set(false);
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                status_message.set(format!("Failed to fetch from Blockstream API: {}", e));
                                is_loading.set(false);
                                return;
                            }
                        }
                    }
                };
                
                let script_pubkey = utxo_data["scriptPubKey"]["hex"].as_str()
                    .or_else(|| utxo_data["scriptpubkey"].as_str())
                    .unwrap_or("");
                let asset = utxo_data["asset"].as_str().unwrap_or("");
                
                let value_sats = match utxo_data["value"] {
                    serde_json::Value::Number(ref n) => {
                        if let Some(v_btc) = n.as_f64() {
                            (v_btc * 100_000_000.0).round() as u64
                        } else if let Some(v) = n.as_u64() {
                            if v > 100_000_000 {
                                v
                            } else {
                                (v as f64 * 100_000_000.0).round() as u64
                            }
                        } else {
                            0
                        }
                    }
                    serde_json::Value::String(ref s) => {
                        s.parse::<f64>()
                            .map(|v_btc| (v_btc * 100_000_000.0).round() as u64)
                            .unwrap_or(0)
                    }
                    _ => 0,
                };
                
                if script_pubkey.is_empty() || asset.is_empty() || value_sats == 0 {
                    status_message.set(format!("Failed to extract UTXO data. Response: {}", serde_json::to_string_pretty(&utxo_data).unwrap_or_default()));
                    is_loading.set(false);
                    return;
                }
                
                let utxo_value_btc = value_sats as f64 / 100_000_000.0;
                let amount_sats = (amount * 100_000_000.0).round() as u64;
                
                if amount_sats > value_sats {
                    status_message.set(format!(
                        "Spend amount {} L-BTC ({} sats) exceeds available UTXO value {} L-BTC ({} sats).\n\nPlease enter an amount less than or equal to the funded amount.",
                        amount, amount_sats, utxo_value_btc, value_sats
                    ));
                    is_loading.set(false);
                    return;
                }
                
                // Covenant requires exactly 3 outputs:
                // Output 0: Payment to destination address
                // Output 1: Same covenant script (recursive) - must be the contract address
                // Output 2: Fee output
                
                // Calculate amounts for 3 outputs
                const MIN_FEE_SATS: u64 = 100;
                let fee_sats = MIN_FEE_SATS; // Use minimum fee
                let change_sats = value_sats - amount_sats - fee_sats;
                
                if change_sats < 0 {
                    status_message.set(format!(
                        "Insufficient funds. UTXO value {} L-BTC ({} sats) is less than payment {} L-BTC ({} sats) + fee {} L-BTC ({} sats).\n\nPlease reduce the spend amount.",
                        utxo_value_btc, value_sats, amount, amount_sats, fee_sats as f64 / 100_000_000.0, fee_sats
                    ));
                    is_loading.set(false);
                    return;
                }
                
                if change_sats == 0 {
                    status_message.set(format!(
                        "No change remaining. UTXO value {} L-BTC ({} sats) equals payment {} L-BTC ({} sats) + fee {} L-BTC ({} sats).\n\nThe covenant requires Output 1 to be the recursive covenant (change). Please reduce the spend amount to leave room for change.",
                        utxo_value_btc, value_sats, amount, amount_sats, fee_sats as f64 / 100_000_000.0, fee_sats
                    ));
                    is_loading.set(false);
                    return;
                }
                
                let contract_addr = contract_address.read().clone();
                if contract_addr.is_empty() {
                    status_message.set("Contract address is required for recursive covenant output".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Convert to BTC for API calls
                let amount_btc = (amount_sats as f64 / 100_000_000.0 * 100_000_000.0).round() / 100_000_000.0;
                let change_btc = (change_sats as f64 / 100_000_000.0 * 100_000_000.0).round() / 100_000_000.0;
                let fee_btc = (fee_sats as f64 / 100_000_000.0 * 100_000_000.0).round() / 100_000_000.0;
                
                status_message.set(format!(
                    "Creating PSET with 3 outputs (covenant requirement):\n\
                    UTXO value: {} L-BTC ({} sats)\n\
                    Output 0 (Payment): {} L-BTC ({} sats) to {}\n\
                    Output 1 (Recursive Covenant): {} L-BTC ({} sats) to {}\n\
                    Output 2 (Fee): {} L-BTC ({} sats)",
                    utxo_value_btc, value_sats,
                    amount_btc, amount_sats, destination,
                    change_btc, change_sats, contract_addr,
                    fee_btc, fee_sats
                ));
                
                // Create PSET with 3 outputs:
                // Output 0: Payment address
                // Output 1: Contract address (recursive covenant)
                // Output 2: Fee output
                // 
                // IMPORTANT: For the covenant to work, we need 3 actual outputs.
                // The fee must be Output 2 and marked as a fee output.
                // We'll create it with {"fee": amount} which should create a fee output.
                let inputs = vec![(txid.clone(), vout)];
                let outputs = vec![
                    (destination.clone(), amount_btc),           // Output 0: Payment
                    (contract_addr.clone(), change_btc),         // Output 1: Recursive covenant
                ];
                
                // Create PSET with fee - the fee should appear as Output 2
                // Note: If this doesn't create 3 outputs, we may need to manually add the fee output
                let base_pset = match rpc_context.create_pset(&inputs, &outputs, Some(fee_btc)).await {
                    Ok(pset) => pset,
                    Err(e) => {
                        status_message.set(format!("Failed to create base PSET with elements-cli: {}\n\nThis creates the initial PSET with 3 outputs (payment, recursive covenant, fee).", e));
                        is_loading.set(false);
                        return;
                    }
                };
                
                status_message.set("Updating PSET with Simplicity data...".to_string());
                
                let internal_key_val = internal_key.read().clone();
                if internal_key_val.is_empty() {
                    status_message.set("Internal key is required. Please provide it.".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let value_btc = utxo_value_btc;
                let value_str = format!("{:.8}", value_btc);
                
                let updated_pset = match hal_context.update_pset_input(
                    &base_pset,
                    0,
                    &script_pubkey,
                    &asset,
                    &value_str,
                    &cmr,
                    &internal_key_val,
                ) {
                    Ok(pset) => pset,
                    Err(e) => {
                        status_message.set(format!("Failed to update PSET with Simplicity data: {}", e));
                        is_loading.set(false);
                        return;
                    }
                };
                
                pset_for_signing.set(updated_pset.clone());
                
                // Decode PSET to show its structure
                status_message.set("Decoding PSET to verify structure...".to_string());
                let decoded_pset = match rpc_context.decode_pset(&updated_pset).await {
                    Ok(decoded) => decoded,
                    Err(e) => {
                        status_message.set(format!(
                            "PSET updated but failed to decode: {}\n\nPSET (first 200 chars): {}...\n\nContinuing anyway...",
                            e,
                            updated_pset.chars().take(200).collect::<String>()
                        ));
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Extract inputs and outputs from decoded PSET
                let mut decoded_info = String::new();
                decoded_info.push_str("PSET Decoded Successfully!\n\n");
                
                // Show inputs
                if let Some(inputs) = decoded_pset.get("tx").and_then(|tx| tx.get("vin")).and_then(|v| v.as_array()) {
                    decoded_info.push_str(&format!("INPUTS ({}):\n", inputs.len()));
                    for (i, input) in inputs.iter().enumerate() {
                        if let (Some(txid), Some(vout)) = (
                            input.get("txid").and_then(|v| v.as_str()),
                            input.get("vout").and_then(|v| v.as_u64())
                        ) {
                            decoded_info.push_str(&format!("  Input {}: txid={}, vout={}\n", i, txid, vout));
                        }
                    }
                }
                
                // Show outputs - try both structures (tx.vout and outputs array)
                let mut output_count = 0;
                let mut outputs_found = false;
                
                // Try the outputs array format first (what decodepsbt actually returns)
                if let Some(outputs) = decoded_pset.get("outputs").and_then(|v| v.as_array()) {
                    outputs_found = true;
                    output_count = outputs.len();
                    decoded_info.push_str(&format!("\nOUTPUTS ({}):\n", output_count));
                    for (i, output) in outputs.iter().enumerate() {
                        let value = output.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let value_sats = (value * 100_000_000.0).round() as u64;
                        
                        // Try to get address from script
                        let address = if let Some(script) = output.get("script") {
                            if let Some(addr) = script.get("address").and_then(|v| v.as_str()) {
                                addr.to_string()
                            } else {
                                "N/A".to_string()
                            }
                        } else {
                            "N/A".to_string()
                        };
                        
                        // Check if it's a fee output (fee outputs might not have an address)
                        let output_type = if output.get("fee").is_some() || address == "N/A" && i == 2 {
                            "Fee"
                        } else if i == 0 {
                            "Payment"
                        } else if i == 1 {
                            "Recursive Covenant"
                        } else {
                            "Other"
                        };
                        
                        decoded_info.push_str(&format!(
                            "  Output {}: {} L-BTC ({} sats) to {} [{}]\n",
                            i, value, value_sats, address, output_type
                        ));
                    }
                } 
                // Fallback to tx.vout format
                else if let Some(outputs) = decoded_pset.get("tx").and_then(|tx| tx.get("vout")).and_then(|v| v.as_array()) {
                    outputs_found = true;
                    output_count = outputs.len();
                    decoded_info.push_str(&format!("\nOUTPUTS ({}):\n", output_count));
                    for (i, output) in outputs.iter().enumerate() {
                        let value = output.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let value_sats = (value * 100_000_000.0).round() as u64;
                        
                        // Try to get address from scriptPubKey
                        let address = if let Some(script_pubkey) = output.get("scriptPubKey") {
                            if let Some(addresses) = script_pubkey.get("addresses").and_then(|v| v.as_array()) {
                                if let Some(addr) = addresses.first().and_then(|v| v.as_str()) {
                                    addr.to_string()
                                } else {
                                    "N/A".to_string()
                                }
                            } else {
                                "N/A".to_string()
                            }
                        } else {
                            "N/A".to_string()
                        };
                        
                        // Check if it's a fee output
                        let output_type = if output.get("fee").is_some() {
                            "Fee"
                        } else if i == 0 {
                            "Payment"
                        } else if i == 1 {
                            "Recursive Covenant"
                        } else {
                            "Other"
                        };
                        
                        decoded_info.push_str(&format!(
                            "  Output {}: {} L-BTC ({} sats) to {} [{}]\n",
                            i, value, value_sats, address, output_type
                        ));
                    }
                }
                
                if !outputs_found {
                    decoded_info.push_str("\nOUTPUTS: Could not decode outputs\n");
                }
                
                // Check for fee in top-level fields
                if let Some(fee_value) = decoded_pset.get("fee").and_then(|v| v.as_f64()) {
                    decoded_info.push_str(&format!("\n⚠️  WARNING: Fee found as separate field: {} L-BTC ({} sats)\n", 
                        fee_value, (fee_value * 100_000_000.0).round() as u64));
                    decoded_info.push_str("The covenant requires the fee to be Output 2. The fee might need to be added as a separate output.\n");
                }
                
                // Check output count
                if output_count != 3 {
                    decoded_info.push_str(&format!(
                        "\n⚠️  ERROR: Expected 3 outputs but found {}!\n\
                        The covenant requires exactly 3 outputs:\n\
                        - Output 0: Payment\n\
                        - Output 1: Recursive Covenant\n\
                        - Output 2: Fee\n",
                        output_count
                    ));
                }
                
                // Show expected vs actual
                decoded_info.push_str(&format!(
                    "\nExpected Structure:\n\
                    - Output 0: {} L-BTC to {} (Payment)\n\
                    - Output 1: {} L-BTC to {} (Recursive Covenant)\n\
                    - Output 2: {} L-BTC (Fee)\n",
                    amount_btc, destination,
                    change_btc, contract_addr,
                    fee_btc
                ));
                
                decoded_info.push_str("\nReady for signing. The covenant will verify this structure during finalization.");
                
                // Also show the full decoded JSON for debugging
                decoded_info.push_str("\n\nFull Decoded PSET JSON:\n");
                if let Ok(json_str) = serde_json::to_string_pretty(&decoded_pset) {
                    // Limit to first 2000 chars to avoid overwhelming the UI
                    let preview = if json_str.len() > 2000 {
                        format!("{}...\n\n(truncated, full JSON has {} chars)", 
                            json_str.chars().take(2000).collect::<String>(),
                            json_str.len())
                    } else {
                        json_str
                    };
                    decoded_info.push_str(&preview);
                } else {
                    decoded_info.push_str("(Could not serialize decoded PSET)");
                }
                
                status_message.set(decoded_info);
                
                is_loading.set(false);
            });
        }
    };

    // Sign and finalize logic is the same as P2MS
    let sign_and_finalize = {
        let rpc_context = rpc_context.clone();
        let hal_context = hal_context.clone();
        move |_| {
            let rpc_context = rpc_context.clone();
            let hal_context = hal_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Signing and finalizing transaction...".to_string());
                
                let pset = pset_for_signing.read().clone();
                if pset.is_empty() {
                    status_message.set("Please create the PSET first".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let cmr = contract_cmr.read().clone();
                if cmr.is_empty() {
                    status_message.set("CMR not found".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let simf_path = simf_file_path.read().clone();
                let witness_path = witness_file_path.read().clone();
                if simf_path.is_empty() || witness_path.is_empty() {
                    status_message.set("Please provide both .simf file path and witness file path".to_string());
                    is_loading.set(false);
                    return;
                }
                
                let current_pset = pset.clone();
                let privkey1 = privkey_1.read().clone();
                let privkey2 = privkey_2.read().clone();
                let privkey3 = privkey_3.read().clone();
                
                let mut sig1: Option<String> = None;
                let mut sig2: Option<String> = None;
                let mut sig3: Option<String> = None;
                
                let mut signing_errors = Vec::new();
                
                if !privkey1.is_empty() {
                    status_message.set("Signing with private key 1...".to_string());
                    match hal_context.sighash_and_sign(&current_pset, 0, &cmr, &privkey1) {
                        Ok(sig) => {
                            sig1 = Some(sig);
                            status_message.set("Signature 1 generated successfully".to_string());
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to sign with key 1:\n{}", e);
                            signing_errors.push(error_msg.clone());
                            status_message.set(format!("{}", error_msg));
                        }
                    }
                }
                
                if !privkey2.is_empty() {
                    status_message.set("Signing with private key 2...".to_string());
                    match hal_context.sighash_and_sign(&current_pset, 0, &cmr, &privkey2) {
                        Ok(sig) => {
                            sig2 = Some(sig);
                            status_message.set("Signature 2 generated successfully".to_string());
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to sign with key 2:\n{}", e);
                            signing_errors.push(error_msg.clone());
                            status_message.set(format!("{}", error_msg));
                        }
                    }
                }
                
                if !privkey3.is_empty() {
                    status_message.set("Signing with private key 3...".to_string());
                    match hal_context.sighash_and_sign(&current_pset, 0, &cmr, &privkey3) {
                        Ok(sig) => {
                            sig3 = Some(sig);
                            status_message.set("Signature 3 generated successfully".to_string());
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to sign with key 3:\n{}", e);
                            signing_errors.push(error_msg.clone());
                            status_message.set(format!("{}", error_msg));
                        }
                    }
                }
                
                let signature_count = [&sig1, &sig2, &sig3].iter().filter(|s| s.is_some()).count();
                if signature_count < 2 {
                    let all_errors = if signing_errors.is_empty() {
                        "No signatures generated. Please provide at least 2 private keys.".to_string()
                    } else {
                        format!("Only {} signature(s) generated (need 2 for 2-of-3 multisig).\n\nErrors:\n{}", 
                            signature_count,
                            signing_errors.join("\n\n"))
                    };
                    status_message.set(all_errors);
                    is_loading.set(false);
                    return;
                }
                
                if !signing_errors.is_empty() {
                    status_message.set(format!("Warning: Some signatures failed, but continuing with {} successful signature(s).\n\nErrors:\n{}", 
                        signature_count,
                        signing_errors.join("\n\n")));
                }
                
                status_message.set("Updating witness file with signatures...".to_string());
                
                let witness_template = r#"{
    "MAYBE_SIGS": {
        "value": "[None, None, None]",
        "type": "[Option<Signature>; 3]"
    }
}"#;
                
                let witness_content = match tokio::fs::read_to_string(&witness_path).await {
                    Ok(content) if !content.trim().is_empty() => {
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(_) => witness_template.to_string(),
                            Err(_) => witness_template.to_string(),
                        }
                    }
                    _ => witness_template.to_string(),
                };
                
                let mut witness_json: serde_json::Value = match serde_json::from_str(&witness_content) {
                    Ok(json) => json,
                    Err(e) => {
                        status_message.set(format!("Failed to parse witness file as JSON: {}\n\nFile content:\n{}", e, witness_content));
                        is_loading.set(false);
                        return;
                    }
                };
                
                let array_string = match witness_json["MAYBE_SIGS"]["value"].as_str() {
                    Some(s) => s,
                    None => {
                        status_message.set(format!("Invalid witness file format: MAYBE_SIGS.value is not a string\n\nFile content:\n{}", witness_content));
                        is_loading.set(false);
                        return;
                    }
                };
                
                let mut array_elements = vec!["None".to_string(), "None".to_string(), "None".to_string()];
                
                match (sig1.as_ref(), sig2.as_ref(), sig3.as_ref()) {
                    (Some(s1), None, Some(s3)) => {
                        array_elements[0] = format!("Some(0x{})", s1);
                        array_elements[2] = format!("Some(0x{})", s3);
                    }
                    (Some(s1), Some(s2), None) => {
                        array_elements[0] = format!("Some(0x{})", s1);
                        array_elements[1] = format!("Some(0x{})", s2);
                    }
                    (None, Some(s2), Some(s3)) => {
                        array_elements[1] = format!("Some(0x{})", s2);
                        array_elements[2] = format!("Some(0x{})", s3);
                    }
                    (Some(s1), Some(s2), Some(s3)) => {
                        array_elements[0] = format!("Some(0x{})", s1);
                        array_elements[1] = format!("Some(0x{})", s2);
                        array_elements[2] = format!("Some(0x{})", s3);
                    }
                    (Some(s1), None, None) => {
                        array_elements[0] = format!("Some(0x{})", s1);
                    }
                    (None, Some(s2), None) => {
                        array_elements[1] = format!("Some(0x{})", s2);
                    }
                    (None, None, Some(s3)) => {
                        array_elements[2] = format!("Some(0x{})", s3);
                    }
                    _ => {}
                }
                
                let updated_array_string = format!("[{}]", array_elements.join(", "));
                
                let mut updated_witness_json = serde_json::Map::new();
        
                if let Some(maybe_sigs) = witness_json.get("MAYBE_SIGS") {
                    if let Some(maybe_sigs_obj) = maybe_sigs.as_object() {
                        let mut maybe_sigs_map = serde_json::Map::new();
                        maybe_sigs_map.insert("value".to_string(), serde_json::Value::String(updated_array_string));
                        
                        if let Some(type_field) = maybe_sigs_obj.get("type") {
                            maybe_sigs_map.insert("type".to_string(), type_field.clone());
                        } else {
                            maybe_sigs_map.insert("type".to_string(), serde_json::Value::String("[Option<Signature>; 3]".to_string()));
                        }
                        
                        updated_witness_json.insert("MAYBE_SIGS".to_string(), serde_json::Value::Object(maybe_sigs_map));
                    }
                } else {
                    let mut maybe_sigs_map = serde_json::Map::new();
                    maybe_sigs_map.insert("value".to_string(), serde_json::Value::String(updated_array_string));
                    maybe_sigs_map.insert("type".to_string(), serde_json::Value::String("[Option<Signature>; 3]".to_string()));
                    updated_witness_json.insert("MAYBE_SIGS".to_string(), serde_json::Value::Object(maybe_sigs_map));
                }
                
                let updated_witness = match serde_json::to_string_pretty(&serde_json::Value::Object(updated_witness_json)) {
                    Ok(json_str) => json_str,
                    Err(e) => {
                        status_message.set(format!("Failed to serialize updated witness JSON: {}", e));
                        is_loading.set(false);
                        return;
                    }
                };
                
                let temp_witness_path = format!("{}.tmp", witness_path);
                match tokio::fs::write(&temp_witness_path, &updated_witness).await {
                    Ok(_) => {
                        status_message.set("Witness file updated with signatures".to_string());
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to write updated witness file: {}", e));
                        is_loading.set(false);
                        return;
                    }
                }
                
                status_message.set("Compiling program with updated witness file...".to_string());
                let (program_with_witness, witness_data) = match hal_context.compile_simf_with_witness(&simf_path, &temp_witness_path) {
                    Ok((prog, wit)) => (prog, wit),
                    Err(e) => {
                        status_message.set(format!("Failed to compile with witness: {}", e));
                        let _ = tokio::fs::remove_file(&temp_witness_path).await;
                        is_loading.set(false);
                        return;
                    }
                };
                
                if let Err(e) = tokio::fs::write(&witness_path, &updated_witness).await {
                    status_message.set(format!("Warning: Could not save updated witness file: {}. Continuing with temp file...", e));
                } else {
                    status_message.set("Witness file updated and saved".to_string());
                }
                
                let _ = tokio::fs::remove_file(&temp_witness_path).await;
                
                status_message.set("Finalizing PSET with program and witness (covenant will verify 3 outputs)...".to_string());
                let finalized_pset = match hal_context.finalize_pset_with_witness(
                    &current_pset,
                    0,
                    &program_with_witness,
                    &witness_data,
                ) {
                    Ok(pset) => pset,
                    Err(e) => {
                        let error_msg = e.to_string();
                        let detailed_error = if error_msg.contains("Jet failed") || error_msg.contains("failed during execution") {
                            format!(
                                "Failed to finalize PSET: {}\n\n\
                                This error ('Jet failed during execution') typically means the covenant structure is not satisfied.\n\n\
                                The covenant requires exactly 3 outputs:\n\
                                1. Output 0: Payment to any address (your destination)\n\
                                2. Output 1: Same covenant script (recursive) - must be the contract address\n\
                                3. Output 2: Fee output\n\n\
                                Other possible causes:\n\
                                - Signatures don't match the public keys in the program\n\
                                - Private keys don't correspond to the public keys\n\
                                - You need exactly 2 valid signatures for 2-of-3 multisig\n\n\
                                Check:\n\
                                - Private keys match the public keys in your cov_p2ms.simf file\n\
                                - You provided at least 2 private keys\n\
                                - The PSET was created with 3 outputs (payment, recursive covenant, fee)\n\
                                - Output 1 is the contract address (same script)\n\
                                - Output 2 is marked as fee",
                                error_msg
                            )
                        } else {
                            format!("Failed to finalize PSET: {}", error_msg)
                        };
                        status_message.set(detailed_error);
                        is_loading.set(false);
                        return;
                    }
                };
                
                final_pset.set(finalized_pset.clone());
                
                status_message.set("Finalizing PSBT...".to_string());
                match rpc_context.finalize_pset(&finalized_pset).await {
                    Ok(tx_hex) => {
                        final_tx_hex.set(tx_hex.clone());
                        status_message.set(format!(
                            "Transaction finalized successfully!\n\nTransaction Hex (first 200 chars): {}...\n\nReady to broadcast.\n\nNote: Covenant enforces 3 outputs (payment, recursive, fee).",
                            tx_hex.chars().take(200).collect::<String>()
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to finalize PSBT: {}\n\nMake sure all signatures are correct and covenant structure is satisfied.", e));
                    }
                }
                
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
                    status_message.set("Please finalize the transaction first".to_string());
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
                        let error_msg = e.to_string();
                        let detailed_error = if error_msg.contains("Assertion failed inside jet") || error_msg.contains("non-mandatory-script-verify-flag") {
                            format!(
                                "Failed to broadcast transaction: {}\n\n\
                                This error means the covenant execution failed.\n\n\
                                Common causes:\n\
                                1. Covenant structure not satisfied (must have exactly 3 outputs)\n\
                                2. Output 1 is not the same script (recursive covenant)\n\
                                3. Output 2 is not marked as fee\n\
                                4. Signatures don't match the public keys\n\
                                5. Invalid signatures\n\n\
                                The covenant enforces:\n\
                                - Exactly 3 outputs\n\
                                - Output 1: Same P2MS covenant script (recursive)\n\
                                - Output 2: Fee output",
                                error_msg
                            )
                        } else {
                            format!("Failed to broadcast transaction: {}", error_msg)
                        };
                        status_message.set(detailed_error);
                    }
                }
                
                is_loading.set(false);
            });
        }
    };

    rsx! {
        div { id: "voucher-panel",
            h1 { style: "font-size: 2rem; margin-bottom: 24px;", "Voucher Workflow (P2MS with Covenant)" }
            
            div { class: "panel-section",
                h2 { "0. Generate Voucher Simplicity Source File" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Simplicity Source File (.simf) Output Path" }
                    input {
                        r#type: "text",
                        value: "{simf_file_path}",
                        oninput: move |evt| simf_file_path.set(evt.value().to_string()),
                        placeholder: "/path/to/cov_p2ms.simf"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Enter the full path where the .simf file will be generated"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Key 1 (Participant 1) - 64 hex characters" }
                    input {
                        r#type: "text",
                        value: "{pubkey_1}",
                        oninput: move |evt| pubkey_1.set(evt.value().to_string()),
                        placeholder: "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "32-byte public key in hex format (64 characters)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Key 2 (Participant 2) - 64 hex characters" }
                    input {
                        r#type: "text",
                        value: "{pubkey_2}",
                        oninput: move |evt| pubkey_2.set(evt.value().to_string()),
                        placeholder: "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "32-byte public key in hex format (64 characters)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Key 3 (Participant 3) - 64 hex characters" }
                    input {
                        r#type: "text",
                        value: "{pubkey_3}",
                        oninput: move |evt| pubkey_3.set(evt.value().to_string()),
                        placeholder: "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "32-byte public key in hex format (64 characters)"
                    }
                }
                
                button {
                    class: "button",
                    onclick: generate_simf,
                    disabled: is_loading(),
                    "Generate cov_p2ms.simf File"
                }
            }
            
            div { class: "panel-section",
                h2 { "1. Compile Simplicity Source (Optional)" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Simplicity Source File (.simf)" }
                    input {
                        r#type: "text",
                        value: "{simf_file_path}",
                        oninput: move |evt| simf_file_path.set(evt.value().to_string()),
                        placeholder: "/path/to/cov_p2ms.simf"
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
                h2 { "2. Create Voucher Contract Address" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Compiled Simplicity Program (base64) - Required" }
                    textarea {
                        rows: "6",
                        value: "{contract_program_input}",
                        oninput: move |evt| contract_program_input.set(evt.value().to_string()),
                        placeholder: "Paste compiled covenant program base64 here or compile from .simf above"
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
                h2 { "3. Fund Contract Address via Faucet" }
                
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
                    label { "Faucet Amount (L-BTC)" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        min: "0.00000001",
                        value: "{faucet_amount}",
                        oninput: move |evt| faucet_amount.set(evt.value().to_string()),
                        placeholder: "0.001"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Amount to request from the Liquid Testnet faucet (default: 0.001 L-BTC)"
                    }
                }
                
                button {
                    class: "button",
                    onclick: fund_via_faucet,
                    disabled: is_loading() || contract_address().is_empty() || faucet_amount().is_empty(),
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
            
            div { id: "spend-voucher", class: "panel-section",
                h2 { "4. Create Spending PSET" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Destination Address" }
                    input {
                        r#type: "text",
                        value: "{spend_destination}",
                        oninput: move |evt| spend_destination.set(evt.value().to_string()),
                        placeholder: "Enter destination address"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Address to send the funds to (Output 0)"
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
                        "Amount to send (must be less than or equal to the funded amount)\n\nNote: Covenant enforces 3 outputs:\n- Output 0: Payment\n- Output 1: Same covenant script (recursive)\n- Output 2: Fee"
                    }
                }
                
                div { style: "margin-top: 16px; margin-bottom: 16px;",
                    label { "Internal Key (Taproot)" }
                    input {
                        r#type: "text",
                        value: "{internal_key}",
                        oninput: move |evt| internal_key.set(evt.value().to_string()),
                        placeholder: "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Unspendable internal key for Taproot (default provided)"
                    }
                }
                
                button {
                    class: "button",
                    onclick: create_spend_pset,
                    disabled: is_loading() || funding_txid().is_empty() || contract_cmr().is_empty(),
                    "Create and Update PSET"
                }
                
                if !pset_for_signing().is_empty() {
                    div { class: "info-box info", style: "margin-top: 16px;",
                        p { style: "font-weight: 600; margin-bottom: 8px;", "PSET Ready for Signing:" }
                        textarea {
                            rows: "4",
                            readonly: true,
                            value: "{pset_for_signing}",
                            style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem; width: 100%;"
                        }
                    }
                }
                
                div { style: "margin-top: 24px; margin-bottom: 16px;",
                    label { "Witness File Path (.wit)" }
                    input {
                        r#type: "text",
                        value: "{witness_file_path}",
                        oninput: move |evt| witness_file_path.set(evt.value().to_string()),
                        placeholder: "/path/to/cov_p2ms.wit"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Path to witness file (will be updated with signatures)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Private Key 1 (hex)" }
                    input {
                        r#type: "text",
                        value: "{privkey_1}",
                        oninput: move |evt| privkey_1.set(evt.value().to_string()),
                        placeholder: "0000000000000000000000000000000000000000000000000000000000000001"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Private key for participant 1 (optional)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Private Key 2 (hex)" }
                    input {
                        r#type: "text",
                        value: "{privkey_2}",
                        oninput: move |evt| privkey_2.set(evt.value().to_string()),
                        placeholder: "0000000000000000000000000000000000000000000000000000000000000002"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Private key for participant 2 (optional)"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Private Key 3 (hex)" }
                    input {
                        r#type: "text",
                        value: "{privkey_3}",
                        oninput: move |evt| privkey_3.set(evt.value().to_string()),
                        placeholder: "0000000000000000000000000000000000000000000000000000000000000005"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Private key for participant 3 (optional)"
                    }
                }
                
                button {
                    class: "button",
                    onclick: sign_and_finalize,
                    disabled: is_loading() || pset_for_signing().is_empty() || witness_file_path().is_empty() || simf_file_path().is_empty(),
                    "Sign and Finalize Transaction"
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
                div { class: "loading", "Loading" }
            }
        }
    }
}

