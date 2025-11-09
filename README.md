# Partnerfy

**Partnerfy** is a desktop application for creating and managing **Simplicity covenant-based contracts** on **Liquid Testnet**. It provides workflows for working with multisig contracts (P2MS) and covenant-enforced vouchers.

It won 3rd place in SatsHack III.

## Desktop App

<img width="1470" height="892" alt="Screenshot 2025-11-07 at 07 07 35" src="https://github.com/user-attachments/assets/0159ca8f-d599-498e-8e15-76116663ffcb" />
<img width="1470" height="891" alt="Screenshot 2025-11-07 at 07 07 53" src="https://github.com/user-attachments/assets/7d5f74b1-a160-46c9-a093-c54450c54301" />


## Overview

Partnerfy is a **Dioxus** desktop application (Rust) that enables you to:

- Generate and compile Simplicity source files for 2-of-3 multisig contracts
- Create covenant-based contract addresses on Liquid Testnet
- Fund contracts via the Liquid Testnet faucet
- Build, sign, and broadcast spending transactions
- Work with covenants that enforce specific output structures (voucher workflow)

The application communicates with Elements (Liquid) via RPC and integrates **SimplicityHL** and **hal-simplicity** for covenant creation and validation.

## Core Concept

### Covenant-Based Vouchers

Partnerfy implements a covenant system where:

1. **Voucher UTXOs** are locked under Simplicity covenants
2. When a participant spends a voucher:
   - Outputs must pay to approved partner addresses, the promoter (refund path), or a 2-of-m multisig
   - Any change output returning to the participant is **automatically locked again by the same covenant**
3. The covenant repeats recursively for all descendant outputs, ensuring funds remain within the system

### Actors

| Role | Description |
|------|-------------|
| **Promoter** | Deploys the Simplicity covenant, funds voucher pool, distributes voucher UTXOs |
| **Participant** | Receives vouchers, redeems them at partner locations, inherits covenant on change |
| **Partner** | Accepts redemption transactions, validates them, and receives funds |

## Technology Stack

| Layer | Function | Tools |
|-------|----------|-------|
| **Frontend/UI** | Desktop application UI | [Dioxus](https://dioxuslabs.com) |
| **Logic/Core** | RPC management, transaction assembly, Simplicity witness handling | Rust, `elements` crate |
| **Node** | On-chain communication | `elementsd` (Liquid Testnet) |
| **Covenant Tooling** | Build, compile, and inspect Simplicity covenants | [SimplicityHL](https://github.com/ElementsProject/simplicity), [hal-simplicity](https://github.com/Blockstream/hal-simplicity) |

## Project Structure

```
Partnerfy/
├── partnerfy_app/          # Main application (Dioxus desktop app)
│   ├── src/                # Source code
│   │   ├── app_core/       # Core business logic (RPC, transaction building, witness handling)
│   │   ├── views/          # UI components (landing, P2MS, voucher workflows)
│   │   └── components/     # Reusable UI components
│   ├── assets/             # Static assets (CSS, images)
│   └── Cargo.toml          # Rust dependencies
├── p2ms.simf               # Simplicity source for 2-of-3 multisig
├── cov_p2ms.simf           # Simplicity source for covenant-based voucher
├── elements.conf.example   # Elements node configuration template
├── WORKFLOW.md             # Detailed workflow documentation
└── INSTRUCTIONS.md         # Technical implementation details
```

## Quick Start

For detailed setup, build, and run instructions, see the [**Application README**](partnerfy_app/README.md).

### Prerequisites

- **Rust** (latest stable)
- **Dioxus CLI** (`cargo install dioxus-cli`)
- **Elements Core** (`elementsd`, `elements-cli`)
- **SimplicityHL Compiler** (`simc`)
- **hal-simplicity** (specific branch: `2025-10/pset-signer`)

### Quick Run

1. **Setup Elements node** (see `elements.conf.example`)
2. **Start elementsd** on Liquid Testnet
3. **Build and run**:
   ```bash
   cd partnerfy_app
   cargo run --release
   ```

## Workflows

### 1. Multisig (P2MS) Workflow

Create a 2-of-3 multisig contract where funds require 2 out of 3 signatures to spend.

**Steps:**
1. Generate P2MS Simplicity source file
2. Compile Simplicity source
3. Create contract address
4. Fund contract via faucet
5. Create spending PSET
6. Sign and finalize transaction
7. Broadcast transaction

### 2. Voucher (Covenant) Workflow

Create a covenant-enforced voucher where change outputs automatically inherit the covenant.

**Steps:**
1. Generate voucher Simplicity source file (with covenant)
2. Compile and create contract address
3. Fund contract via faucet
4. Create spending PSET (must create exactly 3 outputs: payment, recursive covenant, fee)
5. Sign and finalize (covenant verifies structure)
6. Broadcast transaction

## Documentation

- **[Application README](partnerfy_app/README.md)** - Detailed setup, build, run, and test instructions
- **[WORKFLOW.md](WORKFLOW.md)** - Complete workflow analysis and command sequences
- **[INSTRUCTIONS.md](INSTRUCTIONS.md)** - Technical implementation details

## Resources

- **Liquid Testnet Faucet**: https://liquidtestnet.com/faucet
- **Liquid Testnet Explorer**: https://blockstream.info/liquidtestnet
- **Elements Core**: https://github.com/ElementsProject/elements
- **SimplicityHL**: https://github.com/ElementsProject/simplicity
- **hal-simplicity**: https://github.com/Blockstream/hal-simplicity
- **Dioxus**: https://dioxuslabs.com

## Security Notes

⚠️ **Important**: This application is designed for **Liquid Testnet only**. Never use on mainnet until thoroughly tested.

- Always test on Liquid Testnet first
- Store private keys securely (encrypt locally, never share)
- Validate witness correctness before broadcasting
- Verify covenant structure for voucher contracts
- Keep transaction logs for off-chain records
