//! Wrapper for hal-simplicity CLI tool
//! 
//! Executes hal-simplicity commands for covenant compilation and witness generation

use anyhow::{Result, Context};
use std::path::PathBuf;
use std::process::Command;
use serde_json;

/// Wrapper for hal-simplicity CLI
pub struct HalWrapper {
    hal_path: Option<PathBuf>,
}

impl HalWrapper {
    /// Create a new hal-simplicity wrapper
    pub fn new(hal_path: Option<PathBuf>) -> Self {
        Self { hal_path }
    }

    /// Get the hal-simplicity command path
    fn hal_cmd(&self) -> String {
        self.hal_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "hal-simplicity".to_string())
    }

    /// Get the simc compiler command path
    fn simc_cmd(&self) -> String {
        // simc is a separate tool, not part of hal-simplicity
        "simc".to_string()
    }

    /// Compile a SimplicityHL source file (.simf) to base64
    /// 
    /// Runs: simc <input.simf>
    /// Returns: The compiled base64 program string
    pub fn compile_simf(&self, input_path: &str) -> Result<String> {
        let output = Command::new(&self.simc_cmd())
            .arg(input_path)
            .output()
            .context("Failed to execute simc compiler")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "simc compilation failed: {}",
                stderr
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in simc output")?;

        // Parse output: simc outputs "Program:\n<base64>\n"
        // Extract the base64 program from the output
        let program = stdout
            .lines()
            .find(|line| !line.trim().is_empty() && !line.trim().starts_with("Program:"))
            .ok_or_else(|| anyhow::anyhow!("Could not find program in simc output"))?
            .trim()
            .to_string();

        if program.is_empty() {
            return Err(anyhow::anyhow!("Empty program in simc output"));
        }

        Ok(program)
    }

    /// Get covenant info from compiled program
    /// 
    /// Runs: hal-simplicity simplicity info <program.base64>
    /// Returns: JSON string with CMR, address, etc.
    pub fn get_covenant_info(&self, program_base64: &str) -> Result<String> {
        let output = Command::new(&self.hal_cmd())
            .arg("simplicity")
            .arg("simplicity")
            .arg("info")
            .arg(program_base64)
            .output()
            .context("Failed to execute hal-simplicity")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity info failed: {}",
                stderr
            ));
        }

        String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")
    }

    /// Create transaction with witness
    /// 
    /// Runs: hal-simplicity tx create --program <program> --inputs <inputs> --outputs <outputs> --witness-file <witness>
    pub fn create_tx_with_witness(
        &self,
        program_path: &str,
        inputs: &[(String, u32)],
        outputs: &[(String, f64)],
        witness_file: &str,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("tx")
            .arg("create")
            .arg("--program")
            .arg(program_path);

        // Format inputs
        let inputs_str: Vec<String> = inputs
            .iter()
            .map(|(txid, vout)| format!("{}:{}", txid, vout))
            .collect();
        cmd.arg("--inputs")
            .arg(inputs_str.join(","));

        // Format outputs
        let outputs_str: Vec<String> = outputs
            .iter()
            .map(|(addr, amount)| format!("{}:{}", addr, amount))
            .collect();
        cmd.arg("--outputs")
            .arg(outputs_str.join(","));

        cmd.arg("--witness-file")
            .arg(witness_file);

        let output = cmd.output()
            .context("Failed to execute hal-simplicity tx create")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity tx create failed: {}",
                stderr
            ));
        }

        String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")
    }

    /// Validate witness file
    pub fn validate_witness(&self, program_path: &str, witness_file: &str) -> Result<bool> {
        // This would call a validation command if available
        // For now, just check if files exist
        if !std::path::Path::new(program_path).exists() {
            return Err(anyhow::anyhow!("Program file not found: {}", program_path));
        }
        
        if !std::path::Path::new(witness_file).exists() {
            return Err(anyhow::anyhow!("Witness file not found: {}", witness_file));
        }
        
        Ok(true)
    }

    /// Create a PSET (Partially Signed Elements Transaction) for spending from a Simplicity contract
    /// 
    /// Runs: hal-simplicity simplicity pset create --program <program> --inputs <inputs> --outputs <outputs>
    /// Returns: PSET base64 string
    pub fn create_pset(
        &self,
        program_base64: &str,
        inputs: &[(String, u32)],
        outputs: &[(String, f64)],
    ) -> Result<String> {
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("simplicity")
            .arg("pset")
            .arg("create")
            .arg("--program")
            .arg(program_base64);

        // Format inputs as txid:vout
        let inputs_str: Vec<String> = inputs
            .iter()
            .map(|(txid, vout)| format!("{}:{}", txid, vout))
            .collect();
        cmd.arg("--inputs")
            .arg(inputs_str.join(","));

        // Format outputs as address:amount
        let outputs_str: Vec<String> = outputs
            .iter()
            .map(|(addr, amount)| format!("{}:{}", addr, amount))
            .collect();
        cmd.arg("--outputs")
            .arg(outputs_str.join(","));

        let output = cmd.output()
            .context("Failed to execute hal-simplicity pset create")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity pset create failed: {}",
                stderr
            ));
        }

        String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")
            .map(|s| s.trim().to_string())
    }

    /// Add witness to a PSET
    /// 
    /// Runs: hal-simplicity simplicity pset witness --pset <pset> --witness-file <witness_file>
    /// Returns: Updated PSET base64 string
    pub fn add_witness_to_pset(
        &self,
        pset_base64: &str,
        witness_file: &str,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("simplicity")
            .arg("pset")
            .arg("witness")
            .arg("--pset")
            .arg(pset_base64)
            .arg("--witness-file")
            .arg(witness_file);

        let output = cmd.output()
            .context("Failed to execute hal-simplicity pset witness")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity pset witness failed: {}",
                stderr
            ));
        }

        String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")
            .map(|s| s.trim().to_string())
    }

    /// Update PSET input with Simplicity data
    /// 
    /// Runs: hal-simplicity simplicity pset update-input <pset> <input_index> -i <scriptPubKey:asset:value> -c <cmr> -p <internal_key>
    /// Returns: Updated PSET base64 string
    pub fn update_pset_input(
        &self,
        pset_base64: &str,
        input_index: u32,
        script_pubkey: &str,
        asset: &str,
        value: &str,
        cmr: &str,
        internal_key: &str,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("simplicity")
            .arg("pset")
            .arg("update-input")
            .arg(pset_base64)
            .arg(input_index.to_string())
            .arg("-i")
            .arg(format!("{}:{}:{}", script_pubkey, asset, value))
            .arg("-c")
            .arg(cmr)
            .arg("-p")
            .arg(internal_key);

        let output = cmd.output()
            .context("Failed to execute hal-simplicity pset update-input")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity pset update-input failed: {}",
                stderr
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")?;

        // Parse JSON response to extract pset field
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .context("Failed to parse hal-simplicity JSON response")?;
        
        json.get("pset")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No pset field in response"))
    }

    /// Calculate sighash and sign
    /// 
    /// Runs: hal-simplicity simplicity sighash <pset> <input_index> <cmr> -x <privkey>
    /// Returns: Signature hex string
    pub fn sighash_and_sign(
        &self,
        pset_base64: &str,
        input_index: u32,
        cmr: &str,
        privkey: &str,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("simplicity")
            .arg("sighash")
            .arg(pset_base64)
            .arg(input_index.to_string())
            .arg(cmr)
            .arg("-x")
            .arg(privkey);

        let output = cmd.output()
            .context("Failed to execute hal-simplicity sighash")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity sighash failed: {}",
                stderr
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")?;

        // Parse JSON response to extract signature field
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .context("Failed to parse hal-simplicity JSON response")?;
        
        json.get("signature")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No signature field in response"))
    }

    /// Finalize PSET with Simplicity program and witness
    /// 
    /// Runs: hal-simplicity simplicity pset finalize <pset> <input_index> <program> <witness>
    /// Returns: Finalized PSET base64 string
    pub fn finalize_pset_with_witness(
        &self,
        pset_base64: &str,
        input_index: u32,
        program: &str,
        witness: &str,
    ) -> Result<String> {
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("simplicity")
            .arg("pset")
            .arg("finalize")
            .arg(pset_base64)
            .arg(input_index.to_string())
            .arg(program)
            .arg(witness);

        let output = cmd.output()
            .context("Failed to execute hal-simplicity pset finalize")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "hal-simplicity pset finalize failed: {}",
                stderr
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in hal-simplicity output")?;

        // Parse JSON response to extract pset field
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .context("Failed to parse hal-simplicity JSON response")?;
        
        json.get("pset")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No pset field in response"))
    }
}

