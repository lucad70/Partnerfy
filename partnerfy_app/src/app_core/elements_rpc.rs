//! Elements RPC client wrapper
//! 
//! Provides a high-level interface to elementsd JSON-RPC API using direct JSON-RPC calls
//! and Elements-specific types from the elements crate

use crate::app_core::models::Settings;
use anyhow::{Result, Context};
use serde_json::{json, Value};
use tokio::process::Command;

/// Elements RPC client wrapper using direct JSON-RPC
pub struct ElementsRPC {
    client: reqwest::Client,
    url: String,
    settings: Settings,
}

impl ElementsRPC {
    /// Get the elements-cli command path
    /// Tries to find elements-cli in common locations if not in PATH
    /// Also tries "elements" as an alternative name
    fn elements_cli_cmd(&self) -> String {
        // Try both "elements-cli" and "elements" as command names
        let command_names = ["elements-cli", "elements"];
        
        for cmd_name in &command_names {
            // First, try to find it using 'which' command
            if let Ok(output) = std::process::Command::new("which")
                .arg(cmd_name)
                .output()
            {
                if output.status.success() {
                    if let Ok(path) = String::from_utf8(output.stdout) {
                        let path = path.trim();
                        if !path.is_empty() && std::path::Path::new(path).exists() {
                            return path.to_string();
                        }
                    }
                }
            }
            
            // Try common locations for both names
            let home = std::env::var("HOME").unwrap_or_default();
            let common_paths = [
                format!("/usr/local/bin/{}", cmd_name),
                format!("/usr/bin/{}", cmd_name),
                format!("/opt/elements/bin/{}", cmd_name),
                format!("{}/.cargo/bin/{}", home, cmd_name),
                format!("{}/bin/{}", home, cmd_name),
                format!("/opt/homebrew/bin/{}", cmd_name), // macOS Homebrew
            ];
            
            for path in &common_paths {
                if std::path::Path::new(path).exists() {
                    return path.to_string();
                }
            }
        }
        
        // Fallback to "elements-cli" and let the error handler deal with it
        "elements-cli".to_string()
    }

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
    /// Creates a base PSET without signatures using elements-cli directly
    /// 
    /// Inputs: Array of objects with "txid" and "vout"
    /// Outputs: Array of objects with "address": amount pairs
    pub async fn create_pset(
        &self,
        inputs: &[(String, u32)],
        outputs: &[(String, f64)],
    ) -> Result<String> {
        // Format inputs as JSON array string
        let inputs_json: Vec<Value> = inputs
            .iter()
            .map(|(txid, vout)| {
                json!({
                    "txid": txid,
                    "vout": vout
                })
            })
            .collect();
        let inputs_str = serde_json::to_string(&inputs_json)
            .context("Failed to serialize inputs")?;

        // Format outputs as JSON array string
        // Format: [{"address_string": amount}, ...]
        let outputs_json: Vec<Value> = outputs
            .iter()
            .map(|(addr, amount)| {
                let mut output_obj = serde_json::Map::new();
                output_obj.insert(addr.clone(), json!(amount));
                json!(output_obj)
            })
            .collect();
        let outputs_str = serde_json::to_string(&outputs_json)
            .context("Failed to serialize outputs")?;

        // Call elements-cli createpsbt directly (like simc)
        let cmd = self.elements_cli_cmd();
        let output = match Command::new(&cmd)
            .arg("createpsbt")
            .arg(&inputs_str)
            .arg(&outputs_str)
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "elements-cli command not found at: {}\n\nTroubleshooting:\n1. Check if elements-cli is installed: which elements-cli\n2. Try finding it: find /usr /opt ~/.cargo -name 'elements-cli' 2>/dev/null\n3. If not found, install Elements from: https://github.com/ElementsProject/elements\n4. Make sure elements-cli is in your PATH or specify full path\n5. Common locations:\n   - /usr/local/bin/elements-cli\n   - /usr/bin/elements-cli\n   - ~/.cargo/bin/elements-cli\n   - /opt/elements/bin/elements-cli\n   - ~/bin/elements-cli\n\nNote: The app tried to find elements-cli but couldn't locate it.\n\nOriginal error: {}",
                        cmd, e
                    )
                } else if error_kind == std::io::ErrorKind::PermissionDenied {
                    format!(
                        "Permission denied when executing elements-cli\n\nTroubleshooting:\n1. Check if elements-cli has execute permissions: ls -l $(which elements-cli)\n2. Try running: chmod +x /path/to/elements-cli\n\nOriginal error: {}",
                        e
                    )
                } else {
                    format!(
                        "Failed to execute elements-cli createpsbt: {}\n\nTroubleshooting:\n1. Make sure elements-cli is installed and accessible\n2. Check if elementsd is running: elements-cli getblockchaininfo\n3. Verify your elements.conf configuration\n\nOriginal error: {}",
                        e, e
                    )
                };
                return Err(anyhow::anyhow!(error_msg));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let exit_code = output.status.code().unwrap_or(-1);
            
            // Parse common error patterns
            let mut error_details = format!(
                "elements-cli createpsbt failed with exit code {}\n\nCommand: elements-cli createpsbt\nInputs: {}\nOutputs: {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code, inputs_str, outputs_str, stderr, stdout
            );
            
            // Add specific troubleshooting based on error content
            if stderr.contains("error code: -1") || stderr.contains("error message:") {
                error_details.push_str("\n\nThis looks like an RPC error. Possible causes:\n");
                error_details.push_str("1. elementsd is not running - start it with: elementsd\n");
                error_details.push_str("2. Wrong RPC credentials in ~/.elements/elements.conf\n");
                error_details.push_str("3. Wrong network (testnet vs mainnet) - check your elements.conf\n");
            } else if stderr.contains("Could not connect") || stderr.contains("Connection refused") {
                error_details.push_str("\n\nConnection error detected:\n");
                error_details.push_str("1. Make sure elementsd is running: elements-cli getblockchaininfo\n");
                error_details.push_str("2. Check RPC port in ~/.elements/elements.conf (default: 18884 for testnet)\n");
            } else if stderr.contains("Invalid") || stderr.contains("invalid") {
                error_details.push_str("\n\nInvalid input detected:\n");
                error_details.push_str("1. Check that the transaction ID (txid) is valid and exists\n");
                error_details.push_str("2. Verify the vout index is correct (usually 0)\n");
                error_details.push_str("3. Ensure the destination address is valid for the network\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in elements-cli output")?;
        
        let result = stdout.trim();
        if result.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "elements-cli createpsbt returned empty output\n\nCommand: elements-cli createpsbt\nInputs: {}\nOutputs: {}\n\nStderr:\n{}",
                inputs_str,
                outputs_str,
                stderr
            ));
        }
        
        Ok(result.to_string())
    }

    /// Finalize a PSET to get the raw transaction hex
    /// Uses elements-cli finalizepsbt directly (like simc)
    pub async fn finalize_pset(&self, pset: &str) -> Result<String> {
        // Call elements-cli finalizepsbt directly
        let cmd = self.elements_cli_cmd();
        let output = match Command::new(&cmd)
            .arg("finalizepsbt")
            .arg(pset)
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "elements-cli command not found at: {}\n\nTroubleshooting:\n1. Check if elements-cli is installed: which elements-cli\n2. Try finding it: find /usr /opt ~/.cargo -name 'elements-cli' 2>/dev/null\n3. If not found, install Elements from: https://github.com/ElementsProject/elements\n4. Make sure elements-cli is in your PATH\n\nOriginal error: {}",
                        cmd, e
                    )
                } else {
                    format!(
                        "Failed to execute elements-cli finalizepsbt: {}\n\nMake sure elements-cli is installed and accessible.\n\nOriginal error: {}",
                        e, e
                    )
                };
                return Err(anyhow::anyhow!(error_msg));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let exit_code = output.status.code().unwrap_or(-1);
            let pset_preview = pset.chars().take(200).collect::<String>();
            
            let mut error_details = format!(
                "{} finalizepsbt failed with exit code {}\n\nCommand: {} finalizepsbt\nPSET (first 200 chars): {}...\n\nStderr:\n{}\n\nStdout:\n{}",
                cmd, exit_code, cmd, pset_preview, stderr, stdout
            );
            
            // Add specific troubleshooting
            if stderr.contains("not final") || stderr.contains("incomplete") {
                error_details.push_str("\n\nPSET is not fully signed:\n");
                error_details.push_str("1. Make sure all required signatures are present\n");
                error_details.push_str("2. Verify that all participants have signed the PSET\n");
                error_details.push_str("3. Check that the witness data is complete\n");
            } else if stderr.contains("Invalid") || stderr.contains("invalid") {
                error_details.push_str("\n\nInvalid PSET detected:\n");
                error_details.push_str("1. Verify the PSET format is correct (base64)\n");
                error_details.push_str("2. Make sure the PSET hasn't been corrupted\n");
                error_details.push_str("3. Try recreating the PSET from scratch\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in elements-cli output")?;
        
        // Parse JSON response to extract hex
        let json: Value = serde_json::from_str(&stdout)
            .context("Failed to parse finalizepsbt JSON response")?;
        
        if let Some(hex) = json.get("hex").and_then(|v| v.as_str()) {
            Ok(hex.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!(
                "PSET finalization failed or incomplete - no 'hex' field in response\n\nResponse JSON:\n{}\n\nStderr:\n{}",
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| stdout),
                stderr
            ))
        }
    }

    /// Get transaction output details
    /// Uses elements-cli gettxout directly (like simc)
    pub async fn get_txout(&self, txid: &str, vout: u32) -> Result<Value> {
        // Call elements-cli gettxout directly
        let cmd = self.elements_cli_cmd();
        let output = match Command::new(&cmd)
            .arg("gettxout")
            .arg(txid)
            .arg(vout.to_string())
            .output()
            .await
        {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "elements-cli command not found at: {}\n\nTroubleshooting:\n1. Check if elements-cli is installed: which elements-cli\n2. Try finding it: find /usr /opt ~/.cargo -name 'elements-cli' 2>/dev/null\n3. If not found, install Elements from: https://github.com/ElementsProject/elements\n4. Make sure elements-cli is in your PATH\n\nOriginal error: {}",
                        cmd, e
                    )
                } else {
                    format!(
                        "Failed to execute elements-cli gettxout: {}\n\nMake sure elements-cli is installed and accessible.\n\nOriginal error: {}",
                        e, e
                    )
                };
                return Err(anyhow::anyhow!(error_msg));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let exit_code = output.status.code().unwrap_or(-1);
            
            let mut error_details = format!(
                "{} gettxout failed with exit code {}\n\nCommand: {} gettxout {} {}\n\nStderr:\n{}\n\nStdout:\n{}",
                cmd, exit_code, cmd, txid, vout, stderr, stdout
            );
            
            // Add specific troubleshooting
            if stderr.contains("not found") || stderr.contains("null") || stdout.contains("null") {
                error_details.push_str("\n\nUTXO not found. Possible causes:\n");
                error_details.push_str("1. The transaction hasn't been confirmed yet - wait for confirmation\n");
                error_details.push_str("2. The UTXO has already been spent\n");
                error_details.push_str("3. Wrong transaction ID or vout index\n");
                error_details.push_str("4. The transaction is on a different network (testnet vs mainnet)\n");
                error_details.push_str("5. Your elementsd node hasn't synced this transaction yet\n");
            } else if stderr.contains("error code: -1") || stderr.contains("error message:") {
                error_details.push_str("\n\nRPC error detected:\n");
                error_details.push_str("1. Make sure elementsd is running: elements-cli getblockchaininfo\n");
                error_details.push_str("2. Check RPC credentials in ~/.elements/elements.conf\n");
            } else if stderr.contains("Could not connect") || stderr.contains("Connection refused") {
                error_details.push_str("\n\nConnection error:\n");
                error_details.push_str("1. Start elementsd: elementsd\n");
                error_details.push_str("2. Wait for it to sync: elements-cli getblockchaininfo\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in elements-cli output")?;
        
        serde_json::from_str(&stdout)
            .context("Failed to parse gettxout JSON response")
    }

    /// Get settings reference
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}
