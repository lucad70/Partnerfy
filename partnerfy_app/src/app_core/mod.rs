//! Core modules for Partnerfy
//! 
//! This module contains the core functionality for interacting with Elements,
//! building transactions, managing witnesses, and interfacing with hal-simplicity.

pub mod elements_rpc;
pub mod tx_builder;
pub mod witness;
pub mod hal_wrapper;
pub mod models;

pub use elements_rpc::ElementsRPC;
pub use tx_builder::TxBuilder;
pub use witness::WitnessBuilder;
pub use hal_wrapper::HalWrapper;
pub use models::*;

