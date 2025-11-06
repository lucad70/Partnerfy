//! P2MS (Pay-to-Multisig) workflow page
//! 
//! Creates a Simplicity contract address for multisig, funds it via faucet, and manages spending

use crate::app_core::{ElementsRPC, HalWrapper};
use dioxus::prelude::*;
use std::sync::Arc;
use serde_json::{self, json};
use regex::Regex;
use std::path::Path;

#[component]
pub fn P2MS() -> Element {
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
                                        "P2MS Contract created successfully!\n\nCMR: {}\nAddress: {}",
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

    let create_spend_pset = {
        let rpc_context = rpc_context.clone();
        let hal_context = hal_context.clone();
        move |_| {
            let rpc_context = rpc_context.clone();
            let hal_context = hal_context.clone();
            spawn(async move {
                is_loading.set(true);
                status_message.set("Creating spending PSET...".to_string());
                
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
                // We need the UTXO value to calculate correct outputs and fees
                // Script: while ! $ELEMENTS_CLI gettxout $FAUCET_TRANSACTION 0 | grep . >/dev/null; do sleep 5; done
                status_message.set("Waiting for UTXO to be available...".to_string());
                let mut utxo_data: Option<serde_json::Value> = None;
                let mut attempts = 0;
                const MAX_ATTEMPTS: u32 = 20; // Wait up to 100 seconds (20 * 5)
                
                while attempts < MAX_ATTEMPTS {
                    match rpc_context.get_txout(&txid, vout).await {
                        Ok(data) => {
                            // Check if data is valid (not null/empty)
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
                        // Try Blockstream API as fallback
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
                
                // Extract UTXO data (matching script: HEX=$(jq -r .scriptPubKey.hex), ASSET=$(jq -r .asset), VALUE=$(jq -r .value))
                let script_pubkey = utxo_data["scriptPubKey"]["hex"].as_str()
                    .or_else(|| utxo_data["scriptpubkey"].as_str())
                    .unwrap_or("");
                let asset = utxo_data["asset"].as_str().unwrap_or("");
                // Script uses: VALUE=$(jq -r .value) - this is in sats (as a number)
                // elements-cli gettxout returns value as a number (sats), not BTC
                let value_sats = utxo_data["value"].as_u64()
                    .or_else(|| utxo_data["value"].as_f64().map(|v| (v * 100_000_000.0) as u64))
                    .unwrap_or(0);
                
                if script_pubkey.is_empty() || asset.is_empty() || value_sats == 0 {
                    status_message.set(format!("Failed to extract UTXO data. Response: {}", serde_json::to_string_pretty(&utxo_data).unwrap_or_default()));
                    is_loading.set(false);
                    return;
                }
                
                // Convert UTXO value from sats to BTC for display
                let utxo_value_btc = value_sats as f64 / 100_000_000.0;
                
                // Convert user's spend amount from BTC to sats (using integer arithmetic to avoid precision errors)
                // Round to nearest sat to avoid floating point issues
                let amount_sats = (amount * 100_000_000.0).round() as u64;
                
                // Validate that the user's spend amount doesn't exceed the UTXO value
                if amount_sats > value_sats {
                    status_message.set(format!(
                        "Spend amount {} L-BTC ({} sats) exceeds available UTXO value {} L-BTC ({} sats).\n\nPlease enter an amount less than or equal to the funded amount.",
                        amount, amount_sats, utxo_value_btc, value_sats
                    ));
                    is_loading.set(false);
                    return;
                }
                
                // Step 2: Create base PSET using elements-cli createpsbt (matching the bash script)
                // The script uses: elements-cli createpsbt '[ { "txid": "...", "vout": 0 } ]' '[ { "address": amount }, { "fee": fee_amount } ]'
                // Calculate fee in sats (using integer arithmetic to avoid precision errors)
                let fee_sats = value_sats - amount_sats;
                
                // Validate fee is reasonable (not negative, not too small)
                if fee_sats == 0 {
                    status_message.set(format!(
                        "Fee is zero. You cannot spend the entire UTXO value without leaving room for fees.\n\nUTXO value: {} L-BTC ({} sats)\nSpend amount: {} L-BTC ({} sats)\n\nPlease reduce the spend amount to allow for a fee.",
                        utxo_value_btc, value_sats, amount, amount_sats
                    ));
                    is_loading.set(false);
                    return;
                }
                
                // Minimum fee: 100 sats (0.00000100 L-BTC)
                const MIN_FEE_SATS: u64 = 100;
                if fee_sats < MIN_FEE_SATS {
                    status_message.set(format!(
                        "Fee {} sats ({} L-BTC) is too small (minimum recommended: {} sats / 0.00000100 L-BTC).\n\nUTXO value: {} L-BTC ({} sats)\nSpend amount: {} L-BTC ({} sats)\nCalculated fee: {} sats ({} L-BTC)\n\nPlease reduce the spend amount to allow for a reasonable fee.",
                        fee_sats, fee_sats as f64 / 100_000_000.0, MIN_FEE_SATS,
                        utxo_value_btc, value_sats, amount, amount_sats, fee_sats, fee_sats as f64 / 100_000_000.0
                    ));
                    is_loading.set(false);
                    return;
                }
                
                // Convert back to BTC for API calls, using proper rounding to 8 decimal places
                // Round to avoid floating point precision issues (Bitcoin uses 8 decimal places max)
                let amount_btc = (amount_sats as f64 / 100_000_000.0 * 100_000_000.0).round() / 100_000_000.0;
                let fee_btc = (fee_sats as f64 / 100_000_000.0 * 100_000_000.0).round() / 100_000_000.0;
                
                status_message.set(format!(
                    "Creating base PSET with:\nUTXO value: {} L-BTC ({} sats)\nSpend amount: {} L-BTC ({} sats)\nFee: {} L-BTC ({} sats)",
                    utxo_value_btc, value_sats, amount_btc, amount_sats, fee_btc, fee_sats
                ));
                
                let inputs = vec![(txid.clone(), vout)];
                let outputs = vec![(destination.clone(), amount_btc)];
                
                // Create base PSET using elements-cli (matching script workflow)
                // Use the properly calculated fee_btc to avoid floating point precision errors
                let base_pset = match rpc_context.create_pset(&inputs, &outputs, Some(fee_btc)).await {
                    Ok(pset) => pset,
                    Err(e) => {
                        status_message.set(format!("Failed to create base PSET with elements-cli: {}\n\nThis creates the initial PSET that will be updated with Simplicity data.", e));
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Step 3: Update PSET with Simplicity data using hal-simplicity
                // Script: hal-simplicity simplicity pset update-input $PSET 0 -i $HEX:$ASSET:$VALUE -c $CMR -p "$INTERNAL_KEY"
                status_message.set("Updating PSET with Simplicity data...".to_string());
                
                let internal_key_val = internal_key.read().clone();
                if internal_key_val.is_empty() {
                    status_message.set("Internal key is required. Please provide it.".to_string());
                    is_loading.set(false);
                    return;
                }
                
                // Format: HEX:ASSET:VALUE (value in sats, matching script)
                let value_str = value_sats.to_string();
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
                status_message.set(format!(
                    "PSET updated successfully!\n\nPSET (first 200 chars): {}...\n\nReady for signing.",
                    updated_pset.chars().take(200).collect::<String>()
                ));
                
                is_loading.set(false);
            });
        }
    };

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
                
                // Step 1: Sign with private keys and capture signatures
                let mut current_pset = pset.clone();
                let privkey1 = privkey_1.read().clone();
                let privkey2 = privkey_2.read().clone();
                let privkey3 = privkey_3.read().clone();
                
                let mut sig1: Option<String> = None;
                let mut sig2: Option<String> = None;
                let mut sig3: Option<String> = None;
                
                // Sign with available private keys and capture signatures
                // Collect all errors to show at the end
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
                
                // Check if we have at least 2 signatures (required for 2-of-3 multisig)
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
                
                // Step 2: Update witness file with signatures (like the script does with sed)
                // Script: sed -i "s/\[Some([^)]*)/[Some(0x$SIGNATURE_1)/" and sed -i "s/Some([^)]*)]/Some(0x$SIGNATURE_3)]/"
                status_message.set("Updating witness file with signatures...".to_string());
                
                // Read the original witness file (JSON format)
                // Note: Signatures are PSET-specific, so we'll reset them to None and use fresh signatures
                let witness_template = r#"{
    "MAYBE_SIGS": {
        "value": "[None, None, None]",
        "type": "[Option<Signature>; 3]"
    }
}"#;
                
                let witness_content = match tokio::fs::read_to_string(&witness_path).await {
                    Ok(content) if !content.trim().is_empty() => {
                        // Parse existing file to validate structure, but we'll reset signatures
                        // Signatures must match the current PSET, so we always start fresh
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(_) => witness_template.to_string(), // Valid JSON, but reset signatures
                            Err(_) => witness_template.to_string(), // Invalid JSON, use template
                        }
                    }
                    _ => witness_template.to_string(), // File doesn't exist or is empty
                };
                
                // Parse JSON and extract the array string from MAYBE_SIGS.value
                let mut witness_json: serde_json::Value = match serde_json::from_str(&witness_content) {
                    Ok(json) => json,
                    Err(e) => {
                        status_message.set(format!("Failed to parse witness file as JSON: {}\n\nFile content:\n{}", e, witness_content));
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Get the array string from MAYBE_SIGS.value
                let array_string = match witness_json["MAYBE_SIGS"]["value"].as_str() {
                    Some(s) => s,
                    None => {
                        status_message.set(format!("Invalid witness file format: MAYBE_SIGS.value is not a string\n\nFile content:\n{}", witness_content));
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Update signatures in the array string
                // The program expects signatures in positions matching the public keys:
                // - Position 0: signature for pk1 (0x79be667e... = 1*G, private key ending in ...0001)
                // - Position 1: signature for pk2 (0xc6047f94... = 2*G, private key ending in ...0002)
                // - Position 2: signature for pk3 (0xf9308a01... = 3*G, private key ending in ...0003)
                // We need exactly 2 signatures for 2-of-3 multisig
                
                // Build the array properly: [Some(0x...), None, Some(0x...)] etc.
                // Start with all None, then replace with signatures in the correct positions
                let mut array_elements = vec!["None".to_string(), "None".to_string(), "None".to_string()];
                
                // Place signatures in the correct positions
                // sig1 corresponds to privkey_1 -> position 0 (pk1)
                // sig2 corresponds to privkey_2 -> position 1 (pk2)
                // sig3 corresponds to privkey_3 -> position 2 (pk3)
                if let Some(ref sig) = sig1 {
                    array_elements[0] = format!("Some(0x{})", sig);
                }
                if let Some(ref sig) = sig2 {
                    array_elements[1] = format!("Some(0x{})", sig);
                }
                if let Some(ref sig) = sig3 {
                    array_elements[2] = format!("Some(0x{})", sig);
                }
                
                // Construct the final array string
                let updated_array_string = format!("[{}]", array_elements.join(", "));
                
                // Debug: Show which signatures were placed
                let sig_count = array_elements.iter().filter(|s| !s.starts_with("None")).count();
                status_message.set(format!(
                    "Witness file updated with {} signature(s):\nPosition 0 (pk1): {}\nPosition 1 (pk2): {}\nPosition 2 (pk3): {}",
                    sig_count,
                    if array_elements[0].starts_with("None") { "None".to_string() } else { format!("Some(0x{}...)", &array_elements[0].chars().skip(9).take(16).collect::<String>()) },
                    if array_elements[1].starts_with("None") { "None".to_string() } else { format!("Some(0x{}...)", &array_elements[1].chars().skip(9).take(16).collect::<String>()) },
                    if array_elements[2].starts_with("None") { "None".to_string() } else { format!("Some(0x{}...)", &array_elements[2].chars().skip(9).take(16).collect::<String>()) },
                ));
                
                // Update the JSON with the new array string
                if let Some(maybe_sigs) = witness_json.get_mut("MAYBE_SIGS") {
                    if let Some(value_field) = maybe_sigs.get_mut("value") {
                        *value_field = serde_json::Value::String(updated_array_string);
                    }
                }
                
                // Convert back to JSON string
                let updated_witness = match serde_json::to_string_pretty(&witness_json) {
                    Ok(json_str) => json_str,
                    Err(e) => {
                        status_message.set(format!("Failed to serialize updated witness JSON: {}", e));
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Write updated witness file to a temporary location
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
                
                // Step 3: Compile program with updated witness file
                status_message.set("Compiling program with updated witness file...".to_string());
                let (program_with_witness, witness_data) = match hal_context.compile_simf_with_witness(&simf_path, &temp_witness_path) {
                    Ok((prog, wit)) => (prog, wit),
                    Err(e) => {
                        status_message.set(format!("Failed to compile with witness: {}", e));
                        // Clean up temp file
                        let _ = tokio::fs::remove_file(&temp_witness_path).await;
                        is_loading.set(false);
                        return;
                    }
                };
                
                // Optionally save updated witness file back to original location for inspection
                // (The script uses a temp file, but we can save it back so user can see the result)
                if let Err(e) = tokio::fs::write(&witness_path, &updated_witness).await {
                    status_message.set(format!("Warning: Could not save updated witness file: {}. Continuing with temp file...", e));
                } else {
                    status_message.set("Witness file updated and saved".to_string());
                }
                
                // Clean up temp file after successful compilation
                let _ = tokio::fs::remove_file(&temp_witness_path).await;
                
                // Step 4: Finalize PSET with hal-simplicity
                status_message.set("Finalizing PSET with program and witness...".to_string());
                let finalized_pset = match hal_context.finalize_pset_with_witness(
                    &current_pset,
                    0,
                    &program_with_witness,
                    &witness_data,
                ) {
                    Ok(pset) => pset,
                    Err(e) => {
                        // Provide detailed error message for "Jet failed during execution"
                        let error_msg = e.to_string();
                        let detailed_error = if error_msg.contains("Jet failed") || error_msg.contains("failed during execution") {
                            format!(
                                "Failed to finalize PSET: {}\n\n\
                                This error ('Jet failed during execution') typically means:\n\
                                1. Signatures don't match the public keys in the program\n\
                                2. Private keys don't correspond to the public keys in p2ms.simf\n\
                                3. The program expects:\n\
                                   - Position 0: signature for pk1 (0x79be667e... = 1*G, use private key ending in ...0001)\n\
                                   - Position 1: signature for pk2 (0xc6047f94... = 2*G, use private key ending in ...0002)\n\
                                   - Position 2: signature for pk3 (0xf9308a01... = 3*G, use private key ending in ...0003)\n\
                                4. You need exactly 2 valid signatures for 2-of-3 multisig\n\
                                5. Signatures are PSET-specific - if you changed the PSET, you need new signatures\n\n\
                                Check:\n\
                                - Private keys match the public keys in your p2ms.simf file\n\
                                - You provided at least 2 private keys\n\
                                - The witness file was updated with the correct signatures\n\
                                - The PSET hasn't changed since signing",
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
                
                // Step 4: Finalize PSBT with elements-cli
                status_message.set("Finalizing PSBT...".to_string());
                match rpc_context.finalize_pset(&finalized_pset).await {
                    Ok(tx_hex) => {
                        final_tx_hex.set(tx_hex.clone());
                        status_message.set(format!(
                            "Transaction finalized successfully!\n\nTransaction Hex (first 200 chars): {}...\n\nReady to broadcast.",
                            tx_hex.chars().take(200).collect::<String>()
                        ));
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to finalize PSBT: {}\n\nMake sure all signatures are correct.", e));
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
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Key 1 (Participant 1)" }
                    input {
                        r#type: "text",
                        value: "{pubkey_1}",
                        oninput: move |evt| pubkey_1.set(evt.value().to_string()),
                        placeholder: "Enter public key hash for participant 1"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Key 2 (Participant 2)" }
                    input {
                        r#type: "text",
                        value: "{pubkey_2}",
                        oninput: move |evt| pubkey_2.set(evt.value().to_string()),
                        placeholder: "Enter public key hash for participant 2"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Public Key 3 (Participant 3)" }
                    input {
                        r#type: "text",
                        value: "{pubkey_3}",
                        oninput: move |evt| pubkey_3.set(evt.value().to_string()),
                        placeholder: "Enter public key hash for participant 3"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Required Signatures (m)" }
                    input {
                        r#type: "number",
                        min: "1",
                        max: "3",
                        value: "{required_sigs}",
                        oninput: move |evt| required_sigs.set(evt.value().to_string()),
                        placeholder: "e.g., 2 for 2-of-3 multisig"
                    }
                    p { style: "font-size: 0.875rem; color: #666; margin-top: 4px;",
                        "Number of signatures required to spend (m)"
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
            
            div { id: "spend-p2ms", class: "panel-section",
                h2 { "3. Create Spending PSET" }
                
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
                        placeholder: "/path/to/p2ms.wit"
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
                div { style: "text-align: center; padding: 16px;", "Loading..." }
            }
        }
    }
}

