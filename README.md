# Partnerfy
**Partnerfy** is a **desktop app** that issues, manages, and redeems **covenant-based vouchers** on **Liquid Testnet**.  

---

## 1. Overview

**Partnerfy** is a **desktop app** that issues, manages, and redeems **covenant-based vouchers** on **Liquid Testnet**.  
Each voucher is a UTXO locked under a **Simplicity covenant**, ensuring that **any change output** created when the participant spends the voucher **inherits the same spending conditions** as the parent (i.e., remains locked by the same covenant).

This system is implemented as a **Dioxus** desktop application (Rust) communicating with **Elements (Liquid)** via RPC and integrating **SimplicityHL** and **hal-simplicity** for covenant creation and validation.

---

## 2. Core Concept

### Covenant Goal
The voucher UTXO enforces that:
1. When the participant spends it:
   - Outputs must pay only:
     - To approved **partner P2PKH addresses**,  
     - To the **promoter** (refund path), or  
     - To a **2-of-m multisig** among partners,  
     - **And** any *change output* returning to the participant is **locked again by the same covenant**.
2. The covenant repeats recursively for all descendant outputs.
3. The Simplicity program statically enforces these output constraints.

### Actors
| Role | Description |
|------|--------------|
| **Promoter** | Deploys the Simplicity covenant, funds voucher pool, distributes voucher UTXOs. |
| **Participant** | Receives vouchers, redeems them at partner locations, inherits covenant on change. |
| **Partner** | Accepts redemption transactions, validates them, and receives funds. |

---

## 3. Technical Stack

| Layer | Function | Tools |
|--------|-----------|-------|
| **Frontend/UI** | Multi-role desktop app (Promoter / Participant / Partner) | [Dioxus](https://dioxuslabs.com) |
| **Logic/Core** | RPC management, transaction assembly, Simplicity witness handling | Rust modules, `bitcoincore-rpc`, `elements` crate |
| **Node** | On-chain communication | `elementsd` (Liquid Testnet) |
| **Covenant Tooling** | Build, compile, and inspect Simplicity covenant | [SimplicityHL](https://github.com/ElementsProject/simplicity), [hal-simplicity](https://github.com/Blockstream/hal-simplicity) |

---

## 4. Covenant Specification (Simplicity Logic)

### Program Intent
The Simplicity covenant program (`voucher.simf`) enforces:
1. Each spending transaction **must** have outputs that satisfy:
   - `scriptPubKey == partner_P2PKH OR promoter_P2PKH OR 2-of-m_multisig OR covenant_scriptPubKey`
2. If any output returns funds to the participant (change), that output **must use the same covenant script** as the parent.
3. Any spend **must** contain valid participant (and possibly partner) signatures in the witness.
4. Future improvement: add hash/time locks for refund or time-based conditions.
