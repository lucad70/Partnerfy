//! Partner panel for transaction verification and broadcast

use crate::app_core::ElementsRPC;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn Partner() -> Element {
    let mut transaction_hex = use_signal(|| String::new());
    let mut status_message = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut partner_address = use_signal(|| String::new());
    
    let rpc_context = consume_context::<Arc<ElementsRPC>>();

    let validate_transaction = move |_| {
        spawn(async move {
            is_loading.set(true);
            let tx_hex = transaction_hex.read().clone();
            
            if tx_hex.is_empty() {
                status_message.set("No transaction provided".to_string());
                is_loading.set(false);
                return;
            }
            
            // TODO: Decode transaction and validate:
            // 1. Input references valid voucher covenant
            // 2. Output includes partner's address
            // 3. Change is locked to covenant
            
            status_message.set("Transaction validation not yet fully implemented".to_string());
            is_loading.set(false);
        });
    };

    let broadcast_transaction = move |_| {
        let rpc_context = rpc_context.clone();
        spawn(async move {
            is_loading.set(true);
            let tx_hex = transaction_hex.read().clone();
            
            if tx_hex.is_empty() {
                status_message.set("No transaction provided".to_string());
                is_loading.set(false);
                return;
            }
            
            match rpc_context.send_raw_transaction(&tx_hex).await {
                Ok(txid) => {
                    status_message.set(format!("Transaction broadcasted successfully!\nTxID: {}", txid));
                    transaction_hex.set(String::new());
                }
                Err(e) => {
                    status_message.set(format!("Error broadcasting transaction: {}", e));
                }
            }
            
            is_loading.set(false);
        });
    };

    rsx! {
        div { id: "partner-panel",
            h1 { style: "font-size: 2rem; margin-bottom: 24px;", "Partner Panel" }
            
            div { class: "panel-section",
                h2 { "My Partner Address" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Partner P2PKH Address" }
                    input {
                        value: "{partner_address}",
                        oninput: move |evt| partner_address.set(evt.value().to_string()),
                        placeholder: "Your partner address"
                    }
                }
            }
            
            div { class: "panel-section",
                h2 { "Verify & Broadcast Transaction" }
                
                div { style: "margin-bottom: 16px;",
                    label { "Transaction Hex" }
                    textarea {
                        style: "font-family: 'Roboto Mono', monospace; font-size: 0.9rem;",
                        rows: "10",
                        value: "{transaction_hex}",
                        oninput: move |evt| transaction_hex.set(evt.value().to_string()),
                        placeholder: "Paste raw transaction hex from participant"
                    }
                }
                
                div { style: "display: flex; gap: 12px;",
                    button {
                        class: "button",
                        onclick: validate_transaction,
                        disabled: is_loading(),
                        "Validate Transaction"
                    }
                    button {
                        class: "button",
                        onclick: broadcast_transaction,
                        disabled: is_loading(),
                        "Broadcast Transaction"
                    }
                }
            }
            
            div { class: "panel-section",
                h2 { "Transaction Validation Rules" }
                ul { class: "rules-list",
                    li { "Input must reference a valid voucher covenant UTXO" }
                    li { "At least one output must pay to partner address" }
                    li { "Any change output must be locked by the same covenant" }
                    li { "Transaction must include valid participant signature" }
                    li { "Transaction must include partner signature if required" }
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

