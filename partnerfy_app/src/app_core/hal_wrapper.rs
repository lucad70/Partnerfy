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
    /// Returns: The compiled base64 program string (from the last line of output)
    pub fn compile_simf(&self, input_path: &str) -> Result<String> {
        let cmd = self.simc_cmd();
        let output = match Command::new(&cmd)
            .arg(input_path)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "simc compiler not found at: {}\n\nCommand: simc {}\n\nTroubleshooting:\n1. Check if simc is installed: which simc\n2. Verify PATH: echo $PATH\n3. Common locations:\n   - /usr/local/bin/simc\n   - /usr/bin/simc\n   - ~/.cargo/bin/simc\n   - ~/bin/simc\n4. Install SimplicityHL from: https://github.com/ElementsProject/simplicity\n\nOriginal error: {}",
                        cmd, input_path, e
                    )
                } else if error_kind == std::io::ErrorKind::PermissionDenied {
                    format!(
                        "Permission denied when executing simc\n\nCommand: simc {}\n\nTroubleshooting:\n1. Check if simc has execute permissions: ls -l $(which simc)\n2. Try running: chmod +x /path/to/simc\n\nOriginal error: {}",
                        input_path, e
                    )
                } else {
                    format!(
                        "Failed to execute simc compiler\n\nCommand: simc {}\n\nOriginal error: {}",
                        input_path, e
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
                "simc compilation failed with exit code {}\n\nCommand: simc {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code, input_path, stderr, stdout
            );
            
            // Add troubleshooting based on error content
            if stderr.contains("No such file") || stderr.contains("not found") {
                error_details.push_str("\n\nFile not found:\n");
                error_details.push_str(&format!("1. Verify the file exists: ls -l {}\n", input_path));
                error_details.push_str("2. Check the file path is correct\n");
                error_details.push_str("3. Ensure you have read permissions\n");
            } else if stderr.contains("syntax error") || stderr.contains("parse error") {
                error_details.push_str("\n\nSyntax error detected:\n");
                error_details.push_str("1. Check the SimplicityHL source file syntax\n");
                error_details.push_str("2. Verify the file is a valid .simf file\n");
                error_details.push_str("3. Review the error message above for specific issues\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context(format!("Invalid UTF-8 in simc output\n\nCommand: simc {}\n\nOutput may contain binary data", input_path))?;

        // Parse output: simc outputs multiple lines, the last line is the compiled program
        // Extract the last non-empty line
        let program = stdout
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find program in simc output\n\nCommand: simc {}\n\nOutput:\n{}",
                    input_path,
                    stdout.chars().take(500).collect::<String>()
                )
            })?
            .trim()
            .to_string();

        if program.is_empty() {
            return Err(anyhow::anyhow!(
                "Empty program in simc output\n\nCommand: simc {}\n\nOutput:\n{}",
                input_path,
                stdout.chars().take(500).collect::<String>()
            ));
        }

        Ok(program)
    }

    /// Compile a SimplicityHL source file with witness file
    /// 
    /// Runs: simc <input.simf> <witness.wit>
    /// Returns: Tuple of (program, witness) as base64 strings
    /// The output format is:
    ///   Program:
    ///   <program_base64>
    ///   Witness:
    ///   <witness_base64>
    pub fn compile_simf_with_witness(&self, input_path: &str, witness_path: &str) -> Result<(String, String)> {
        let cmd = self.simc_cmd();
        let output = match Command::new(&cmd)
            .arg(input_path)
            .arg(witness_path)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "simc compiler not found at: {}\n\nCommand: simc {} {}\n\nTroubleshooting:\n1. Check if simc is installed: which simc\n2. Verify PATH: echo $PATH\n3. Common locations:\n   - /usr/local/bin/simc\n   - /usr/bin/simc\n   - ~/.cargo/bin/simc\n4. Install SimplicityHL from: https://github.com/ElementsProject/simplicity\n\nOriginal error: {}",
                        cmd, input_path, witness_path, e
                    )
                } else {
                    format!(
                        "Failed to execute simc compiler with witness\n\nCommand: simc {} {}\n\nOriginal error: {}",
                        input_path, witness_path, e
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
                "simc compilation with witness failed with exit code {}\n\nCommand: simc {} {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code, input_path, witness_path, stderr, stdout
            );
            
            // Add troubleshooting
            if stderr.contains("No such file") {
                error_details.push_str("\n\nFile not found:\n");
                error_details.push_str(&format!("1. Verify source file exists: ls -l {}\n", input_path));
                error_details.push_str(&format!("2. Verify witness file exists: ls -l {}\n", witness_path));
            } else if stderr.contains("syntax error") || stderr.contains("parse error") {
                error_details.push_str("\n\nSyntax error detected:\n");
                error_details.push_str("1. Check the SimplicityHL source file syntax\n");
                error_details.push_str("2. Verify the witness file is valid JSON\n");
                error_details.push_str("3. Check that signatures in witness file are correctly formatted\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context(format!("Invalid UTF-8 in simc output\n\nCommand: simc {} {}", input_path, witness_path))?;

        // Parse output: simc outputs:
        //   Program:
        //   <program_base64>
        //   Witness:
        //   <witness_base64>
        let lines: Vec<&str> = stdout.lines().collect();
        
        // Find program line (line after "Program:")
        let program_idx = lines.iter()
            .position(|line| line.trim().starts_with("Program:"))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find 'Program:' in simc output\n\nCommand: simc {} {}\n\nOutput:\n{}",
                    input_path,
                    witness_path,
                    stdout.chars().take(500).collect::<String>()
                )
            })?;
        
        let program = if program_idx + 1 < lines.len() {
            lines[program_idx + 1].trim().to_string()
        } else {
            return Err(anyhow::anyhow!(
                "Program line missing after 'Program:'\n\nCommand: simc {} {}\n\nOutput:\n{}",
                input_path,
                witness_path,
                stdout.chars().take(500).collect::<String>()
            ));
        };

        // Find witness line (line after "Witness:")
        let witness_idx = lines.iter()
            .position(|line| line.trim().starts_with("Witness:"))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find 'Witness:' in simc output\n\nCommand: simc {} {}\n\nOutput:\n{}",
                    input_path,
                    witness_path,
                    stdout.chars().take(500).collect::<String>()
                )
            })?;
        
        let witness = if witness_idx + 1 < lines.len() {
            lines[witness_idx + 1].trim().to_string()
        } else {
            return Err(anyhow::anyhow!(
                "Witness line missing after 'Witness:'\n\nCommand: simc {} {}\n\nOutput:\n{}",
                input_path,
                witness_path,
                stdout.chars().take(500).collect::<String>()
            ));
        };

        if program.is_empty() || witness.is_empty() {
            return Err(anyhow::anyhow!(
                "Empty program or witness in simc output\n\nCommand: simc {} {}\n\nOutput:\n{}",
                input_path,
                witness_path,
                stdout.chars().take(500).collect::<String>()
            ));
        }

        Ok((program, witness))
    }

    /// Get covenant info from compiled program
    /// 
    /// Runs: hal-simplicity simplicity info <program.base64>
    /// Returns: JSON string with CMR, address, etc.
    pub fn get_covenant_info(&self, program_base64: &str) -> Result<String> {
        let cmd = self.hal_cmd();
        let program_preview = if program_base64.len() > 100 {
            format!("{}...", &program_base64[..100])
        } else {
            program_base64.to_string()
        };
        
        let output = match Command::new(&cmd)
            .arg("simplicity")
            .arg("info")
            .arg(program_base64)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "hal-simplicity not found at: {}\n\nCommand: hal-simplicity simplicity info <program>\n\nTroubleshooting:\n1. Check if hal-simplicity is installed: which hal-simplicity\n2. Verify PATH: echo $PATH\n3. Common locations:\n   - /usr/local/bin/hal-simplicity\n   - /usr/bin/hal-simplicity\n   - ~/.cargo/bin/hal-simplicity\n4. Install hal-simplicity from: https://github.com/Blockstream/hal-simplicity\n\nOriginal error: {}",
                        cmd, e
                    )
                } else {
                    format!(
                        "Failed to execute hal-simplicity\n\nCommand: hal-simplicity simplicity info <program>\n\nOriginal error: {}",
                        e
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
                "hal-simplicity info failed with exit code {}\n\nCommand: hal-simplicity simplicity info <program>\nProgram (first 100 chars): {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code, program_preview, stderr, stdout
            );
            
            // Add troubleshooting
            if stderr.contains("invalid") || stderr.contains("Invalid") {
                error_details.push_str("\n\nInvalid program detected:\n");
                error_details.push_str("1. Verify the program is valid base64\n");
                error_details.push_str("2. Check that the program was compiled correctly\n");
                error_details.push_str("3. Ensure the program is a valid Simplicity program\n");
            } else if stderr.contains("parse") || stderr.contains("JSON") {
                error_details.push_str("\n\nParse error detected:\n");
                error_details.push_str("1. Check that hal-simplicity output is valid JSON\n");
                error_details.push_str("2. Verify hal-simplicity version is compatible\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        String::from_utf8(output.stdout)
            .context(format!("Invalid UTF-8 in hal-simplicity output\n\nCommand: hal-simplicity simplicity info <program>"))
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

        let program_preview = if program_base64.len() > 100 {
            format!("{}...", &program_base64[..100])
        } else {
            program_base64.to_string()
        };
        
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "hal-simplicity not found at: {}\n\nCommand: hal-simplicity simplicity pset create --program <program> --inputs <inputs> --outputs <outputs>\n\nTroubleshooting:\n1. Check if hal-simplicity is installed: which hal-simplicity\n2. Verify PATH: echo $PATH\n3. Install hal-simplicity from: https://github.com/Blockstream/hal-simplicity\n\nOriginal error: {}",
                        self.hal_cmd(), e
                    )
                } else {
                    format!(
                        "Failed to execute hal-simplicity pset create\n\nCommand: hal-simplicity simplicity pset create --program <program> --inputs <inputs> --outputs <outputs>\n\nOriginal error: {}",
                        e
                    )
                };
                return Err(anyhow::anyhow!(error_msg));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let exit_code = output.status.code().unwrap_or(-1);
            
            let inputs_str: Vec<String> = inputs.iter().map(|(txid, vout)| format!("{}:{}", txid, vout)).collect();
            let outputs_str: Vec<String> = outputs.iter().map(|(addr, amount)| format!("{}:{}", addr, amount)).collect();
            
            let mut error_details = format!(
                "hal-simplicity pset create failed with exit code {}\n\nCommand: hal-simplicity simplicity pset create --program <program> --inputs {} --outputs {}\nProgram (first 100 chars): {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code,
                inputs_str.join(","),
                outputs_str.join(","),
                program_preview,
                stderr,
                stdout
            );
            
            // Add troubleshooting
            if stderr.contains("invalid") || stderr.contains("Invalid") {
                error_details.push_str("\n\nInvalid input detected:\n");
                error_details.push_str("1. Verify the program is valid base64\n");
                error_details.push_str("2. Check that inputs are in format txid:vout\n");
                error_details.push_str("3. Ensure outputs are in format address:amount\n");
            } else if stderr.contains("parse") || stderr.contains("JSON") {
                error_details.push_str("\n\nParse error detected:\n");
                error_details.push_str("1. Check that hal-simplicity output is valid\n");
                error_details.push_str("2. Verify hal-simplicity version is compatible\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context(format!("Invalid UTF-8 in hal-simplicity output\n\nCommand: hal-simplicity simplicity pset create"))?;
        
        let result = stdout.trim();
        if result.is_empty() {
            return Err(anyhow::anyhow!(
                "hal-simplicity pset create returned empty output\n\nCommand: hal-simplicity simplicity pset create --program <program> --inputs <inputs> --outputs <outputs>"
            ));
        }
        
        Ok(result.to_string())
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
        // Trim whitespace and newlines from PSET (elements-cli might add them)
        let pset_trimmed = pset_base64.trim();
        
        // Validate PSET is not empty
        if pset_trimmed.is_empty() {
            return Err(anyhow::anyhow!(
                "PSET is empty\n\nCannot update empty PSET with Simplicity data"
            ));
        }
        
        // Basic validation: PSET should be base64-like (alphanumeric, +, /, =)
        // Check for obviously invalid characters
        let invalid_chars: Vec<char> = pset_trimmed
            .chars()
            .filter(|c| !c.is_alphanumeric() && *c != '+' && *c != '/' && *c != '=' && !c.is_whitespace())
            .collect();
        if !invalid_chars.is_empty() {
            return Err(anyhow::anyhow!(
                "PSET contains invalid characters for base64 encoding\n\nInvalid characters found: {:?}\nPSET length: {}\nPSET preview (first 200 chars): {}\n\nThis suggests the PSET format is incorrect",
                invalid_chars,
                pset_trimmed.len(),
                pset_trimmed.chars().take(200).collect::<String>()
            ));
        }
        
        let mut cmd = Command::new(&self.hal_cmd());
        cmd.arg("simplicity")
            .arg("pset")
            .arg("update-input")
            .arg(pset_trimmed)
            .arg(input_index.to_string())
            .arg("-i")
            .arg(format!("{}:{}:{}", script_pubkey, asset, value))
            .arg("-c")
            .arg(cmr)
            .arg("-p")
            .arg(internal_key);

        let pset_preview = if pset_base64.len() > 100 {
            format!("{}...", &pset_base64[..100])
        } else {
            pset_base64.to_string()
        };
        
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "hal-simplicity not found at: {}\n\nCommand: hal-simplicity simplicity pset update-input <pset> {} -i {}:{}:{} -c {} -p <internal_key>\n\nTroubleshooting:\n1. Check if hal-simplicity is installed: which hal-simplicity\n2. Verify PATH: echo $PATH\n3. Install hal-simplicity from: https://github.com/Blockstream/hal-simplicity\n\nOriginal error: {}",
                        self.hal_cmd(), input_index, script_pubkey, asset, value, cmr, e
                    )
                } else {
                    format!(
                        "Failed to execute hal-simplicity pset update-input\n\nCommand: hal-simplicity simplicity pset update-input <pset> {} -i {}:{}:{} -c {} -p <internal_key>\n\nOriginal error: {}",
                        input_index, script_pubkey, asset, value, cmr, e
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
                "hal-simplicity pset update-input failed with exit code {}\n\nCommand: hal-simplicity simplicity pset update-input <pset> {} -i {}:{}:{} -c {} -p <internal_key>\nPSET (first 100 chars): {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code, input_index, script_pubkey, asset, value, cmr, pset_preview, stderr, stdout
            );
            
            // Add troubleshooting
            if stderr.contains("Deserialize error") || stderr.contains("decoding PSET") || stdout.contains("Deserialize error") {
                error_details.push_str("\n\nPSET Deserialize Error detected:\n");
                error_details.push_str("This error suggests hal-simplicity cannot decode the PSET format from elements-cli.\n\n");
                error_details.push_str("Possible causes:\n");
                error_details.push_str("1. Version incompatibility between elements-cli and hal-simplicity\n");
                error_details.push_str("2. PSET format mismatch (elements-cli createpsbt might return PSBT, not PSET)\n");
                error_details.push_str("3. The PSET might need to be converted or validated\n\n");
                error_details.push_str("Troubleshooting steps:\n");
                error_details.push_str("1. Check elements-cli version: elements-cli --version\n");
                error_details.push_str("2. Check hal-simplicity version: hal-simplicity --version\n");
                error_details.push_str("3. Try manually testing the PSET:\n");
                error_details.push_str(&format!("   elements-cli createpsbt '[{{\"txid\":\"<txid>\",\"vout\":0}}]' '[{{\"<address>\":<amount>}}]' > test.pset\n"));
                error_details.push_str("   hal-simplicity simplicity pset update-input $(cat test.pset) 0 -i <hex>:<asset>:<value> -c <cmr> -p <internal_key>\n");
                error_details.push_str("4. Verify the PSET is valid base64: base64 -d test.pset > /dev/null 2>&1 && echo 'Valid base64'\n");
                error_details.push_str(&format!("\nPSET details:\n- Length: {} chars\n- Preview (first 200 chars): {}\n- Last 50 chars: {}", 
                    pset_trimmed.len(), 
                    pset_trimmed.chars().take(200).collect::<String>(),
                    pset_trimmed.chars().rev().take(50).collect::<String>().chars().rev().collect::<String>()));
            } else if stderr.contains("invalid") || stderr.contains("Invalid") {
                error_details.push_str("\n\nInvalid input detected:\n");
                error_details.push_str("1. Verify the PSET is valid base64\n");
                error_details.push_str("2. Check that scriptPubKey, asset, and value are correct\n");
                error_details.push_str("3. Ensure CMR matches the contract\n");
                error_details.push_str("4. Verify internal key is correct\n");
            } else if stderr.contains("parse") || stderr.contains("JSON") {
                error_details.push_str("\n\nParse error detected:\n");
                error_details.push_str("1. Check that hal-simplicity output is valid JSON\n");
                error_details.push_str("2. Verify hal-simplicity version is compatible\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context(format!("Invalid UTF-8 in hal-simplicity output\n\nCommand: hal-simplicity simplicity pset update-input"))?;

        // Parse JSON response to extract pset field
        let json: serde_json::Value = match serde_json::from_str(&stdout) {
            Ok(j) => j,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse hal-simplicity JSON response: {}\n\nCommand: hal-simplicity simplicity pset update-input\n\nRaw stdout:\n{}\n\nExpected JSON with 'pset' field",
                    e,
                    stdout.chars().take(500).collect::<String>()
                ));
            }
        };
        
        match json.get("pset").and_then(|v| v.as_str()) {
            Some(pset) => Ok(pset.to_string()),
            None => {
                Err(anyhow::anyhow!(
                    "No 'pset' field in response\n\nCommand: hal-simplicity simplicity pset update-input\n\nFull JSON response:\n{}\n\nAvailable fields: {:?}",
                    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Failed to serialize".to_string()),
                    json.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
                ))
            }
        }
    }

    /// Calculate sighash and sign
    /// 
    /// Runs: hal-simplicity simplicity sighash <pset> <input_index> <cmr> -x <privkey>
    /// This matches the script: hal-simplicity simplicity sighash "$PSET" 0 "$CMR" -x "$PRIVKEY_1"
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

        // Note: The command structure matches the script exactly:
        // hal-simplicity simplicity sighash "$PSET" 0 "$CMR" -x "$PRIVKEY_1"
        // This uses SIGHASH_ALL (sig_all_hash) as defined in the Simplicity contract

        let output = cmd.output()
            .context(format!("Failed to execute hal-simplicity sighash\n\nCommand: {} simplicity sighash <pset> {} {} -x <privkey>", 
                self.hal_cmd(), input_index, cmr))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "hal-simplicity sighash failed with exit code {}\n\nCommand: hal-simplicity simplicity sighash <pset> {} {} -x <privkey>\n\nStdout:\n{}\n\nStderr:\n{}",
                output.status.code().unwrap_or(-1),
                input_index,
                cmr,
                stdout,
                stderr
            ));
        }

        // Parse JSON response to extract signature field
        let json: serde_json::Value = match serde_json::from_str(&stdout) {
            Ok(j) => j,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse hal-simplicity JSON response: {}\n\nRaw stdout:\n{}\n\nStderr:\n{}",
                    e,
                    stdout,
                    stderr
                ));
            }
        };
        
        match json.get("signature") {
            Some(v) => {
                match v.as_str() {
                    Some(s) => Ok(s.to_string()),
                    None => Err(anyhow::anyhow!(
                        "Signature field is not a string in response\n\nFull JSON response:\n{}\n\nStdout:\n{}\n\nStderr:\n{}",
                        serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Failed to serialize".to_string()),
                        stdout,
                        stderr
                    ))
                }
            }
            None => Err(anyhow::anyhow!(
                "No 'signature' field found in response\n\nFull JSON response:\n{}\n\nStdout:\n{}\n\nStderr:\n{}\n\nAvailable fields: {:?}",
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Failed to serialize".to_string()),
                stdout,
                stderr,
                json.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
            ))
        }
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

        let pset_preview = if pset_base64.len() > 100 {
            format!("{}...", &pset_base64[..100])
        } else {
            pset_base64.to_string()
        };
        let program_preview = if program.len() > 100 {
            format!("{}...", &program[..100])
        } else {
            program.to_string()
        };
        
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                let error_kind = e.kind();
                let error_msg = if error_kind == std::io::ErrorKind::NotFound {
                    format!(
                        "hal-simplicity not found at: {}\n\nCommand: hal-simplicity simplicity pset finalize <pset> {} <program> <witness>\n\nTroubleshooting:\n1. Check if hal-simplicity is installed: which hal-simplicity\n2. Verify PATH: echo $PATH\n3. Install hal-simplicity from: https://github.com/Blockstream/hal-simplicity\n\nOriginal error: {}",
                        self.hal_cmd(), input_index, e
                    )
                } else {
                    format!(
                        "Failed to execute hal-simplicity pset finalize\n\nCommand: hal-simplicity simplicity pset finalize <pset> {} <program> <witness>\n\nOriginal error: {}",
                        input_index, e
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
                "hal-simplicity pset finalize failed with exit code {}\n\nCommand: hal-simplicity simplicity pset finalize <pset> {} <program> <witness>\nPSET (first 100 chars): {}\nProgram (first 100 chars): {}\n\nStderr:\n{}\n\nStdout:\n{}",
                exit_code, input_index, pset_preview, program_preview, stderr, stdout
            );
            
            // Add troubleshooting
            if stderr.contains("invalid") || stderr.contains("Invalid") {
                error_details.push_str("\n\nInvalid input detected:\n");
                error_details.push_str("1. Verify the PSET is valid and properly updated\n");
                error_details.push_str("2. Check that the program and witness are valid base64\n");
                error_details.push_str("3. Ensure all signatures are present in the witness\n");
                error_details.push_str("4. Verify the program matches the contract CMR\n");
            } else if stderr.contains("witness") || stderr.contains("signature") {
                error_details.push_str("\n\nWitness/Signature error detected:\n");
                error_details.push_str("1. Check that all required signatures are present\n");
                error_details.push_str("2. Verify signatures are correctly formatted\n");
                error_details.push_str("3. Ensure witness file matches the contract requirements\n");
            } else if stderr.contains("parse") || stderr.contains("JSON") {
                error_details.push_str("\n\nParse error detected:\n");
                error_details.push_str("1. Check that hal-simplicity output is valid JSON\n");
                error_details.push_str("2. Verify hal-simplicity version is compatible\n");
            }
            
            return Err(anyhow::anyhow!(error_details));
        }

        let stdout = String::from_utf8(output.stdout)
            .context(format!("Invalid UTF-8 in hal-simplicity output\n\nCommand: hal-simplicity simplicity pset finalize"))?;

        // Parse JSON response to extract pset field
        let json: serde_json::Value = match serde_json::from_str(&stdout) {
            Ok(j) => j,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse hal-simplicity JSON response: {}\n\nCommand: hal-simplicity simplicity pset finalize\n\nRaw stdout:\n{}\n\nExpected JSON with 'pset' field",
                    e,
                    stdout.chars().take(500).collect::<String>()
                ));
            }
        };
        
        match json.get("pset").and_then(|v| v.as_str()) {
            Some(pset) => Ok(pset.to_string()),
            None => {
                Err(anyhow::anyhow!(
                    "No 'pset' field in response\n\nCommand: hal-simplicity simplicity pset finalize\n\nFull JSON response:\n{}\n\nAvailable fields: {:?}",
                    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Failed to serialize".to_string()),
                    json.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
                ))
            }
        }
    }
}

