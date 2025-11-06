//! Elements RPC client wrapper
//! 
//! Provides a high-level interface to elementsd JSON-RPC API using direct JSON-RPC calls
//! and Elements-specific types from the elements crate

use crate::app_core::models::Settings;
use anyhow::{Result, Context};
use serde_json::{json, Value};

/// Elements RPC client wrapper using direct JSON-RPC
pub struct ElementsRPC {
    client: reqwest::Client,
    url: String,
    settings: Settings,
}

impl ElementsRPC {
    /// Create a new RPC client with the given settings
    pub fn new(settings: Settings) -> Result<Self> {
        let url = format!(
            "http://{}:{}@{}:{}",
            settings.rpc_user,
            settings.rpc_password,
            settings.rpc_host,
            settings.rpc_port
        );
        
        Ok(Self {
            client: reqwest::Client::new(),
            url,
            settings,
        })
    }

    /// Make a JSON-RPC call
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let payload = json!({
            "jsonrpc": "1.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self.client
            .post(&self.url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send RPC request")?;

        let result: Value = response
            .json()
            .await
            .context("Failed to parse RPC response")?;

        if let Some(error) = result.get("error") {
            return Err(anyhow::anyhow!("RPC error: {}", error));
        }

        result.get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))
    }

    /// Test connection to the node
    pub async fn test_connection(&self) -> Result<()> {
        self.get_blockchain_info().await?;
        Ok(())
    }

    /// Get blockchain info
    pub async fn get_blockchain_info(&self) -> Result<Value> {
        self.call("getblockchaininfo", json!([])).await
    }

    /// Get wallet balance
    pub async fn get_balance(&self) -> Result<f64> {
        let result = self.call("getbalance", json!([])).await?;
        result.as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid balance format"))
    }

    /// Get new address
    pub async fn get_new_address(&self, label: Option<&str>) -> Result<String> {
        let params = if let Some(l) = label {
            json!([l])
        } else {
            json!([])
        };
        let result = self.call("getnewaddress", params).await?;
        result.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid address format"))
    }

    /// Send to address (L-BTC)
    pub async fn send_to_address(&self, address: &str, amount: f64) -> Result<String> {
        let result = self.call("sendtoaddress", json!([address, amount])).await?;
        result.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid txid format"))
    }

    /// Create raw transaction
    pub async fn create_raw_transaction(
        &self,
        inputs: &[(String, u32)],
        outputs: &[(String, f64)],
    ) -> Result<String> {
        let inputs_json: Vec<Value> = inputs
            .iter()
            .map(|(txid, vout)| {
                json!({
                    "txid": txid,
                    "vout": vout
                })
            })
            .collect();

        let mut outputs_map = serde_json::Map::new();
        for (addr, amount) in outputs {
            outputs_map.insert(addr.clone(), json!(amount));
        }

        let params = json!([inputs_json, outputs_map]);
        let result = self.call("createrawtransaction", params).await?;
        result.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction hex format"))
    }

    /// Sign raw transaction with wallet
    pub async fn sign_raw_transaction_with_wallet(&self, hex: &str) -> Result<Value> {
        let result = self.call("signrawtransactionwithwallet", json!([hex])).await?;
        Ok(result)
    }

    /// Send raw transaction
    pub async fn send_raw_transaction(&self, hex: &str) -> Result<String> {
        let result = self.call("sendrawtransaction", json!([hex])).await?;
        result.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid txid format"))
    }

    /// Get transaction details
    pub async fn get_transaction(&self, txid: &str) -> Result<Value> {
        self.call("gettransaction", json!([txid])).await
    }

    /// List unspent outputs
    pub async fn list_unspent(
        &self,
        minconf: Option<u32>,
        maxconf: Option<u32>,
    ) -> Result<Vec<Value>> {
        let params = json!([minconf.unwrap_or(0), maxconf.unwrap_or(9999999)]);
        let result = self.call("listunspent", params).await?;
        result.as_array()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Invalid unspent format"))
    }

    /// Create a PSET (Partially Signed Elements Transaction)
    /// 
    /// Creates a base PSET without signatures
    pub async fn create_pset(
        &self,
        inputs: &[(String, u32)],
        outputs: &[(String, f64)],
    ) -> Result<String> {
        let inputs_json: Vec<Value> = inputs
            .iter()
            .map(|(txid, vout)| {
                json!({
                    "txid": txid,
                    "vout": vout
                })
            })
            .collect();

        let mut outputs_map = serde_json::Map::new();
        for (addr, amount) in outputs {
            outputs_map.insert(addr.clone(), json!(amount));
        }

        let params = json!([inputs_json, outputs_map]);
        let result = self.call("createpsbt", params).await?;
        result.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid PSET format"))
    }

    /// Finalize a PSET to get the raw transaction hex
    pub async fn finalize_pset(&self, pset: &str) -> Result<String> {
        let result = self.call("finalizepsbt", json!([pset])).await?;
        if let Some(hex) = result.get("hex").and_then(|v| v.as_str()) {
            Ok(hex.to_string())
        } else {
            Err(anyhow::anyhow!("PSET finalization failed or incomplete"))
        }
    }

    /// Get transaction output details
    pub async fn get_txout(&self, txid: &str, vout: u32) -> Result<Value> {
        self.call("gettxout", json!([txid, vout])).await
    }

    /// Get settings reference
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}
