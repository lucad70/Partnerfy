//! Promoter panel for covenant creation, funding, and voucher issuance

use crate::app_core::{ElementsRPC, HalWrapper};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn Promoter() -> Element {
    let mut contract_base64 = use_signal(|| String::new());
    let mut covenant_address = use_signal(|| String::new());
    let mut funding_amount = use_signal(|| String::new());
    let mut voucher_amounts = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    
    let rpc_context = consume_context::<Arc<ElementsRPC>>();
    let hal_context = consume_context::<Arc<HalWrapper>>();

    let compile_covenant = move |_| {
        spawn(async move {
            is_loading.set(true);
            status_message.set("Compiling covenant...".to_string());
            
            // TODO: Call simc compiler or hal-simplicity to compile voucher.simf
            // For now, this is a placeholder
            status_message.set("Covenant compilation not yet implemented. Please use simc manually.".to_string());
            is_loading.set(false);
        });
    };

    let load_covenant_info = move |_| {
        let hal_context = hal_context.clone();
        spawn(async move {
            is_loading.set(true);
            status_message.set("Loading covenant info...".to_string());
            
            // TODO: Get program path from contract_base64
            match hal_context.get_covenant_info("voucher.base64") {
                Ok(info) => {
                    status_message.set(format!("Covenant info loaded:\n{}", info));
                    // TODO: Parse info to extract address
                }
                Err(e) => {
                    status_message.set(format!("Error: {}", e));
                }
            }
            is_loading.set(false);
        });
    };

    let fund_covenant = move |_| {
        let rpc_context = rpc_context.clone();
        spawn(async move {
            is_loading.set(true);
            status_message.set("Funding covenant...".to_string());
            
            let amount: f64 = funding_amount.read().parse().unwrap_or(0.0);
            let addr = covenant_address.read().clone();
            
            match rpc_context.send_to_address(&addr, amount).await {
                Ok(txid) => {
                    status_message.set(format!("Funding transaction sent: {}", txid));
                }
                Err(e) => {
                    status_message.set(format!("Error: {}", e));
                }
            }
            is_loading.set(false);
        });
    };

    let create_vouchers = move |_| {
        spawn(async move {
            is_loading.set(true);
            status_message.set("Creating vouchers...".to_string());
            
            // Parse voucher amounts
            let amounts: Vec<f64> = voucher_amounts
                .read()
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            
            if amounts.is_empty() {
                status_message.set("No valid voucher amounts provided".to_string());
                is_loading.set(false);
                return;
            }
            
            // TODO: Get input UTXO from funding transaction
            // TODO: Build split transaction
            // TODO: Sign and broadcast
            
            status_message.set(format!("Would create {} vouchers", amounts.len()));
            is_loading.set(false);
        });
    };

    rsx! {
        div { id: "promoter-panel",
            h1 { style: "font-size: 2rem; margin-bottom: 24px;", "Promoter Panel" }
            
            div { class: "panel-section",
                h2 { "Covenant Management" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Covenant Program (base64)" }
                    textarea {
                        value: "{contract_base64}",
                        oninput: move |evt| contract_base64.set(evt.value().to_string()),
                        placeholder: "Paste compiled covenant base64 or path to voucher.base64",
                        rows: "4"
                    }
                }
                
                div { style: "display: flex; gap: 12px; margin-bottom: 16px;",
                    button {
                        class: "button",
                        onclick: compile_covenant,
                        disabled: is_loading(),
                        "Compile Covenant"
                    }
                    button {
                        class: "button",
                        onclick: load_covenant_info,
                        disabled: is_loading(),
                        "Load Covenant Info"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Covenant Address" }
                    input {
                        value: "{covenant_address}",
                        oninput: move |evt| covenant_address.set(evt.value().to_string()),
                        placeholder: "Address from covenant info"
                    }
                }
            }
            
            div { class: "panel-section",
                h2 { "Fund Covenant Pool" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Funding Amount (L-BTC)" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        value: "{funding_amount}",
                        oninput: move |evt| funding_amount.set(evt.value().to_string()),
                        placeholder: "0.01"
                    }
                }
                
                button {
                    class: "button",
                    onclick: fund_covenant,
                    disabled: is_loading(),
                    "Fund Covenant"
                }
            }
            
            div { class: "panel-section",
                h2 { "Issue Vouchers" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Voucher Amounts (comma-separated, L-BTC)" }
                    input {
                        value: "{voucher_amounts}",
                        oninput: move |evt| voucher_amounts.set(evt.value().to_string()),
                        placeholder: "0.01, 0.01, 0.01"
                    }
                }
                
                button {
                    class: "button",
                    onclick: create_vouchers,
                    disabled: is_loading(),
                    "Create Vouchers"
                }
            }
            
            if !status_message().is_empty() {
                div { class: "status-message",
                    "{status_message}"
                }
            }
            
            if is_loading() {
                div { style: "text-align: center; padding: 16px;", "Loading..." }
            }
        }
    }
}

