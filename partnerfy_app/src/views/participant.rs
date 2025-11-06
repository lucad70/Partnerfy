//! Participant panel for voucher redemption and change management

use crate::app_core::{ElementsRPC, HalWrapper, TxBuilder, VoucherUTXO};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn Participant() -> Element {
    let mut selected_voucher = use_signal(|| Option::<VoucherUTXO>::None);
    let mut partner_address = use_signal(|| String::new());
    let mut redemption_amount = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut vouchers = use_signal(|| Vec::<VoucherUTXO>::new());
    
    let rpc_context = consume_context::<Arc<ElementsRPC>>();
    let hal_context = consume_context::<Arc<HalWrapper>>();

    // Load vouchers on mount
    use_effect(move || {
        spawn(async move {
            // TODO: Load vouchers from storage/wallet
            vouchers.set(vec![]);
        });
    });

    let import_voucher = move |_| {
        spawn(async move {
            is_loading.set(true);
            status_message.set("Import voucher functionality not yet implemented".to_string());
            is_loading.set(false);
        });
    };

    let redeem_voucher = move |_| {
        let rpc_context = rpc_context.clone();
        spawn(async move {
            is_loading.set(true);
            status_message.set("Building redemption transaction...".to_string());
            
            let voucher = match selected_voucher.read().as_ref() {
                Some(v) => v.clone(),
                None => {
                    status_message.set("No voucher selected".to_string());
                    is_loading.set(false);
                    return;
                }
            };
            
            let amount: f64 = redemption_amount.read().parse().unwrap_or(0.0);
            let partner_addr = partner_address.read().clone();
            let covenant_addr = voucher.covenant_address.clone();
            
            // Build transaction
            match TxBuilder::build_redemption_tx(&voucher, &partner_addr, amount, &covenant_addr) {
                Ok(tx) => {
                    // Create raw transaction
                    let inputs: Vec<(String, u32)> = tx.inputs.clone();
                    let outputs: Vec<(String, f64)> = tx.outputs
                        .iter()
                        .map(|o| (o.address.clone(), o.amount))
                        .collect();
                    
                    match rpc_context.create_raw_transaction(&inputs, &outputs).await {
                        Ok(hex) => {
                            status_message.set(format!("Transaction created:\n{}", hex));
                            // TODO: Sign with witness
                            // TODO: Send to partner for co-signature
                        }
                        Err(e) => {
                            status_message.set(format!("Error creating transaction: {}", e));
                        }
                    }
                }
                Err(e) => {
                    status_message.set(format!("Error: {}", e));
                }
            }
            
            is_loading.set(false);
        });
    };

    rsx! {
        div { id: "participant-panel",
            h1 { style: "font-size: 2rem; margin-bottom: 24px;", "Participant Panel" }
            
            div { class: "panel-section",
                h2 { "My Vouchers" }
                
                button {
                    class: "button",
                    onclick: import_voucher,
                    style: "margin-bottom: 16px;",
                    "Import Voucher"
                }
                
                if vouchers().is_empty() {
                    p { style: "color: #666;", "No vouchers imported yet" }
                } else {
                    div { class: "voucher-list",
                        for voucher in vouchers().iter() {
                            div {
                                class: "voucher-item",
                                onclick: {
                                    let voucher = voucher.clone();
                                    move |_| {
                                        selected_voucher.set(Some(voucher.clone()));
                                    }
                                },
                                div { class: "voucher-id", "Voucher: {voucher.txid}:{voucher.vout}" }
                                div { class: "voucher-amount", "Amount: {voucher.amount} L-BTC" }
                            }
                        }
                    }
                }
            }
            
            div { class: "panel-section",
                h2 { "Redeem Voucher" }
                
                if let Some(voucher) = selected_voucher.read().as_ref() {
                    div { class: "info-box info",
                        p { style: "font-weight: 600; margin-bottom: 4px;", "Selected: {voucher.txid}:{voucher.vout}" }
                        p { style: "font-size: 0.9rem;", "Available: {voucher.amount} L-BTC" }
                    }
                } else {
                    div { class: "info-box warning",
                        p { "Please select a voucher first" }
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Partner Address" }
                    input {
                        value: "{partner_address}",
                        oninput: move |evt| partner_address.set(evt.value().to_string()),
                        placeholder: "Enter partner P2PKH address"
                    }
                }
                
                div { style: "margin-bottom: 16px;",
                    label { "Redemption Amount (L-BTC)" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        value: "{redemption_amount}",
                        oninput: move |evt| redemption_amount.set(evt.value().to_string()),
                        placeholder: "0.01"
                    }
                }
                
                button {
                    class: "button",
                    onclick: redeem_voucher,
                    disabled: is_loading() || selected_voucher.read().is_none(),
                    "Redeem Voucher"
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

