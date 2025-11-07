//! Transaction builder for creating and managing raw transactions
//! 
//! Handles transaction assembly, output construction, and change handling

use crate::app_core::models::{TxOutput, RawTransaction, VoucherUTXO};
use anyhow::{Result, Context};

/// Transaction builder helper
pub struct TxBuilder;

impl TxBuilder {
    /// Build a transaction from voucher UTXO to partner and change
    /// 
    /// This ensures that change output uses the same covenant address
    pub fn build_redemption_tx(
        voucher: &VoucherUTXO,
        partner_address: &str,
        partner_amount: f64,
        covenant_address: &str,
    ) -> Result<RawTransaction> {
        let change_amount = voucher.amount - partner_amount;
        
        if change_amount < 0.0 {
            return Err(anyhow::anyhow!(
                "Insufficient voucher amount: {} < {}",
                voucher.amount,
                partner_amount
            ));
        }

        let inputs = vec![(voucher.txid.clone(), voucher.vout)];
        let outputs = vec![
            TxOutput {
                address: partner_address.to_string(),
                amount: partner_amount,
            },
            TxOutput {
                address: covenant_address.to_string(),
                amount: change_amount,
            },
        ];

        Ok(RawTransaction {
            hex: String::new(), // Will be filled by RPC
            inputs,
            outputs,
        })
    }

    /// Build a split transaction to create multiple vouchers
    pub fn build_split_tx(
        input_txid: &str,
        input_vout: u32,
        covenant_address: &str,
        voucher_amounts: &[f64],
    ) -> Result<RawTransaction> {
        let inputs = vec![(input_txid.to_string(), input_vout)];
        let outputs: Vec<TxOutput> = voucher_amounts
            .iter()
            .map(|&amount| TxOutput {
                address: covenant_address.to_string(),
                amount,
            })
            .collect();

        Ok(RawTransaction {
            hex: String::new(),
            inputs,
            outputs,
        })
    }

    /// Validate transaction outputs comply with covenant rules
    /// 
    /// Checks that outputs are either:
    /// - Partner addresses
    /// - Promoter address (refund)
    /// - Covenant address (change)
    /// - 2-of-m multisig (future)
    pub fn validate_covenant_outputs(
        outputs: &[TxOutput],
        allowed_partners: &[&str],
        promoter_address: &str,
        covenant_address: &str,
    ) -> Result<()> {
        for output in outputs {
            let addr = &output.address;
            
            let is_valid = allowed_partners.contains(&addr.as_str())
                || addr == promoter_address
                || addr == covenant_address;
            
            if !is_valid {
                return Err(anyhow::anyhow!(
                    "Output address {} is not allowed by covenant rules",
                    addr
                ));
            }
        }
        
        Ok(())
    }

    /// Calculate change amount
    pub fn calculate_change(input_amount: f64, outputs: &[TxOutput], fee: f64) -> f64 {
        let total_output: f64 = outputs.iter().map(|o| o.amount).sum();
        (input_amount - total_output - fee).max(0.0)
    }
}

