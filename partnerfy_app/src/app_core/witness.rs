//! Witness builder for Simplicity covenant transactions
//! 
//! Handles witness generation and serialization for covenant spending

use crate::app_core::models::Witness;
use anyhow::Result;
use serde_json::json;

/// Witness builder for covenant transactions
pub struct WitnessBuilder;

impl WitnessBuilder {
    /// Create a witness file for hal-simplicity
    /// 
    /// This creates a JSON structure that hal-simplicity can use
    /// to generate the witness for a Simplicity program
    pub fn create_witness_file(
        participant_sig: Option<&str>,
        partner_sig: Option<&str>,
        oracle_data: Option<&str>,
    ) -> Result<String> {
        let mut witness = json!({});
        
        if let Some(sig) = participant_sig {
            witness["participant_sig"] = json!(sig);
        }
        
        if let Some(sig) = partner_sig {
            witness["partner_sig"] = json!(sig);
        }
        
        if let Some(data) = oracle_data {
            witness["oracle_data"] = json!(data);
        }
        
        serde_json::to_string_pretty(&witness)
            .map_err(|e| anyhow::anyhow!("Failed to serialize witness: {}", e))
    }

    /// Build witness from Witness struct
    pub fn build_from_witness(witness: &Witness) -> Result<String> {
        Self::create_witness_file(
            witness.participant_sig.as_deref(),
            witness.partner_sig.as_deref(),
            witness.oracle_data.as_deref(),
        )
    }

    /// Create empty witness template
    pub fn create_empty_witness() -> String {
        json!({}).to_string()
    }
}

