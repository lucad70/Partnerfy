//! Data models for Partnerfy

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Voucher UTXO information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherUTXO {
    pub txid: String,
    pub vout: u32,
    pub amount: f64,
    pub owner_pubkey: String,
    pub covenant_script: String,
    pub covenant_address: String,
}

/// Compiled Simplicity covenant contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub voucher_base64: String,
    pub script_pubkey: String,
    pub address: String,
    pub info: Option<String>, // Output from hal-simplicity info
}

/// Partner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partner {
    pub address: String,
    pub pubkey: Option<String>,
    pub name: String,
}

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub pubkey: String,
    pub wallet_path: Option<String>,
    pub voucher_utxos: Vec<VoucherUTXO>,
}

/// Witness data for transaction signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub participant_sig: Option<String>,
    pub partner_sig: Option<String>,
    pub oracle_data: Option<String>,
}

/// Application settings and RPC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub rpc_host: String,
    pub rpc_port: u16,
    pub rpc_user: String,
    pub rpc_password: String,
    pub chain: String, // "liquidtestnet" or "liquid"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rpc_host: "localhost".to_string(),
            rpc_port: 18891, // Default Liquid Testnet RPC port
            rpc_user: "user".to_string(),
            rpc_password: "password".to_string(),
            chain: "liquidtestnet".to_string(),
        }
    }
}

/// Application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub settings: Settings,
    pub contract: Option<Contract>,
    pub partners: Vec<Partner>,
    pub participants: Vec<Participant>,
    pub vouchers: Vec<VoucherUTXO>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            contract: None,
            partners: Vec::new(),
            participants: Vec::new(),
            vouchers: Vec::new(),
        }
    }
}

/// Transaction output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub address: String,
    pub amount: f64,
}

/// Raw transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    pub hex: String,
    pub inputs: Vec<(String, u32)>, // (txid, vout)
    pub outputs: Vec<TxOutput>,
}

