//! Wrapper for hal-simplicity CLI tool
//! 
//! Executes hal-simplicity commands for covenant compilation and witness generation

use anyhow::{Result, Context};
use std::path::PathBuf;
use std::process::Command;

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

    /// Get covenant info from compiled program
    /// 
    /// Runs: hal-simplicity simplicity info <program.base64>
    pub fn get_covenant_info(&self, program_path: &str) -> Result<String> {
        let output = Command::new(&self.hal_cmd())
            .arg("simplicity")
            .arg("info")
            .arg(program_path)
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
}

