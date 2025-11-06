# P2MS Workflow Instructions

## Overview

This document describes the **Pay-to-Multisig (P2MS)** workflow implementation in the Partnerfy desktop application. The P2MS system uses **Simplicity covenants** to create a 2-of-3 multisig contract on Liquid Testnet, where funds can only be spent when 2 out of 3 participants provide their signatures.

## Technology Stack

### Backend Components

1. **Dioxus** - Rust framework for building desktop UIs
2. **Elements Core (elementsd)** - Liquid blockchain node daemon
3. **elements-cli** - Command-line interface for Elements RPC operations
4. **SimplicityHL (simc)** - High-level language compiler for Simplicity programs
5. **hal-simplicity** - CLI tool for Simplicity program operations (info, witness generation)
6. **Rust Standard Library** - For process execution and async operations
7. **reqwest** - HTTP client for API calls (faucet, Blockstream API)
8. **serde_json** - JSON serialization/deserialization
9. **tokio** - Async runtime for Rust

### Key Technologies

- **Simplicity**: A low-level programming language for Bitcoin-like blockchains
- **PSBT (Partially Signed Bitcoin Transaction)**: Format for exchanging unsigned/partially signed transactions
- **Liquid Testnet**: Test network for Liquid blockchain
- **UTXO Model**: Unspent Transaction Output model used by Bitcoin/Liquid

## P2MS Workflow

The P2MS workflow consists of the following steps:

### Step 0: Compile Simplicity Source (Optional)

**Purpose**: Compile a SimplicityHL source file (`.simf`) into a base64-encoded program.

**Command Used**:
```bash
simc <input.simf>
```

**Implementation**:
- Location: `src/app_core/hal_wrapper.rs::compile_simf()`
- Executes: `simc <input_path>`
- Output: Base64-encoded program string (extracted from stdout)
- Example output:
  ```
  Program:
  5lk2l5vmZ++dy7rFWgYpXOhwsHApv82y3OKNlZ8oFbFvgXmARacYEf5RB7X1tMEVAbpXAfNhcd45LjO88p6usCblccJ7lBgtPyYRQDJLGGIJJonwvxOqRTamOQiwbfM2EMA+InecBt8gyCoWRAoQY4oNggUIOOQKE2AACEGGHIMMFgFpHxOQKEGHG4AccgwVJ4CBOKD8JNwsUH1HCrYwEJFB+NQQDaBwIfhWmNBCBQgwzMAMAKwCD8UGCo/FYIBuC4IAwDxcBxkBxuQKDcam5BnGHHG5CHGHCxC1gOAIFBuQh+SRxhxxx+ShxhxwsgtoDiFAoTYAQIQKDcmjjcnBRm2ggDjpIoA5GA1gcBA4jA4zA5cgcvwOYMkUAclgcxY=
  ```

**Input**: Path to `.simf` file (e.g., `/path/to/p2ms.simf`)

**Output**: Base64-encoded compiled program

---

### Step 1: Create P2MS Contract Address

**Purpose**: Generate a Liquid Testnet address from the compiled Simplicity program.

**Command Used**:
```bash
hal-simplicity simplicity simplicity info "<program_base64>"
```

**Implementation**:
- Location: `src/app_core/hal_wrapper.rs::get_covenant_info()`
- Executes: `hal-simplicity simplicity simplicity info <program_base64>`
- Output: JSON object containing:
  - `cmr`: Commitment Merkle Root (contract identifier)
  - `liquid_testnet_address_unconf`: Liquid Testnet address for unconfidential outputs
  - `liquid_address_unconf`: Liquid mainnet address (if needed)
  - Other metadata (jets, commit_decode, type_arrow, etc.)

**Example Output**:
```json
{
  "jets": "core",
  "commit_base64": "5lk2l5vmZ++dy7rFWgYpXOhwsHApv82y3OKNlZ8oFbFvgXmARacYEf5RB7X1tMEVAbpXAfNhcd45LjO88p6usCblccJ7lBgtPyYRQDJLGGIJJonwvxOqRTamOQiwbfM2EMA+InecBt8gyCoWRAoQY4oNggUIOOQKE2AACEGGHIMMFgFpHxOQKEGHG4AccgwVJ4CBOKD8JNwsUH1HCrYwEJFB+NQQDaBwIfhWmNBCBQgwzMAMAKwCD8UGCo/FYIBuC4IAwDxcBxkBxuQKDcam5BnGHHG5CHGHCxC1gOAIFBuQh+SRxhxxx+ShxhxwsgtoDiFAoTYAQIQKDcmjjcnBRm2ggDjpIoA5GA1gcBA4jA4zA5cgcvwOYMkUAclgcxY=",
  "cmr": "af5b897effb80a06fa19362347b7807dc0e774eaf4271d6526545965b44ddc3e",
  "liquid_testnet_address_unconf": "tex1pfsldwgyf5e2gmg0lvm87zz2zqswk4h2prs63lzxlsgz5lsurgy8s3az3wv",
  "is_redeem": false
}
```

**Input**: Base64-encoded compiled Simplicity program

**Output**: 
- Contract address (Liquid Testnet)
- CMR (Commitment Merkle Root)

---

### Step 2: Fund Contract Address via Faucet

**Purpose**: Send testnet L-BTC to the contract address using the Liquid Testnet faucet.

**API Call Used**:
```bash
curl "https://liquidtestnet.com/faucet?address=<contract_address>&action=lbtc"
```

**Implementation**:
- Location: `src/views/p2ms.rs::fund_via_faucet()`
- Method: HTTP GET request using `reqwest::Client`
- URL: `https://liquidtestnet.com/faucet?address={address}&action=lbtc`
- Response: HTML page containing transaction ID
- Parsing: Uses regex to extract transaction ID from HTML response
  - Pattern: `r"transaction\s+([a-f0-9]{64})"`
  - Alternative pattern: `r"txid[:\s]+([a-f0-9]{64})"`

**Input**: Contract address (from Step 1)

**Output**: 
- Funding transaction ID (txid)
- VOUT index (typically 0)
- Amount (0.001 L-BTC = 100,000 sats)

**Example Response Parsing**:
```html
Sent 100000 sats to address tex1p... with transaction abc123def456...
```
Extracted txid: `abc123def456...`

---

### Step 3: Create Spending PSBT

**Purpose**: Create a Partially Signed Bitcoin Transaction (PSBT) to spend from the contract address.

#### 3.1: Create Base PSBT

**Command Used**:
```bash
elements-cli createpsbt '[{"txid":"<txid>","vout":<vout>}]' '[{"<destination_address>":<amount>}]'
```

**Implementation**:
- Location: `src/app_core/elements_rpc.rs::create_pset()`
- Executes: `elements-cli createpsbt <inputs_json> <outputs_json>`
- Input format:
  - Inputs: JSON array of objects with `txid` and `vout`
  - Outputs: JSON array of objects with address as key and amount as value
- Output: Base64-encoded PSBT string

**Example Command**:
```bash
elements-cli createpsbt '[{"txid":"abc123...","vout":0}]' '[{"tex1q...":0.0005}]'
```

**Input**:
- Funding transaction ID (txid)
- VOUT index
- Destination address
- Amount to send

**Output**: Base64-encoded PSBT

#### 3.2: Get UTXO Data

**Command Used**:
```bash
elements-cli gettxout <txid> <vout>
```

**Implementation**:
- Location: `src/app_core/elements_rpc.rs::get_txout()`
- Executes: `elements-cli gettxout <txid> <vout>`
- Output: JSON object containing UTXO details:
  - `scriptPubKey`: Script public key (hex)
  - `asset`: Asset tag
  - `value`: Value in satoshis

**Fallback**: If `elements-cli gettxout` fails, the app falls back to Blockstream API:
```bash
curl "https://blockstream.info/liquidtestnet/api/tx/<txid>"
```

**Output**: UTXO data (scriptPubKey, asset, value)

#### 3.3: Update PSBT with UTXO Data

**Command Used**:
```bash
elements-cli utxoupdatepsbt "<psbt>"
```

**Implementation**:
- Location: `src/app_core/elements_rpc.rs::update_psbt_utxo()`
- Executes: `elements-cli utxoupdatepsbt <psbt>`
- Purpose: Updates the PSBT with UTXO information from the blockchain
- Output: Updated PSBT (base64-encoded)

**Input**: Base PSBT from Step 3.1

**Output**: Updated PSBT with UTXO data

---

### Step 4: Sign and Finalize PSBT

**Purpose**: Sign the PSBT with required signatures and finalize it into a raw transaction.

#### 4.1: Sign PSBT (External)

**Note**: The current implementation expects users to sign the PSBT externally using their wallet software. The PSBT is exported from the app, signed by participants, and then imported back.

**Workflow**:
1. Export PSBT from app
2. Import PSBT into wallet (e.g., Elements Core wallet, hardware wallet)
3. Sign with required private keys (2 of 3 participants)
4. Export signed PSBT
5. Import signed PSBT back into app

#### 4.2: Finalize PSBT

**Command Used**:
```bash
elements-cli finalizepsbt "<psbt>"
```

**Implementation**:
- Location: `src/app_core/elements_rpc.rs::finalize_pset()`
- Executes: `elements-cli finalizepsbt <psbt>`
- Output: JSON object with:
  - `hex`: Raw transaction hex (if finalized successfully)
  - `complete`: Boolean indicating if PSBT is complete

**Example Output**:
```json
{
  "hex": "0200000001...",
  "complete": true
}
```

**Input**: Fully signed PSBT

**Output**: Raw transaction hex

---

### Step 5: Broadcast Transaction

**Purpose**: Broadcast the finalized transaction to the Liquid Testnet network.

**RPC Call Used**:
```json
{
  "method": "sendrawtransaction",
  "params": ["<tx_hex>"]
}
```

**Implementation**:
- Location: `src/app_core/elements_rpc.rs::send_raw_transaction()`
- Method: JSON-RPC call to `elementsd`
- Endpoint: `http://<rpc_user>:<rpc_password>@<rpc_host>:<rpc_port>`
- Command: `sendrawtransaction <tx_hex>`

**Alternative CLI Command**:
```bash
elements-cli sendrawtransaction "<tx_hex>"
```

**Input**: Raw transaction hex (from Step 4.2)

**Output**: Transaction ID (txid) of the broadcast transaction

---

## Complete Command Sequence

Here is the complete sequence of commands for a P2MS workflow:

```bash
# Step 0: Compile Simplicity source
simc p2ms.simf
# Output: Base64 program

# Step 1: Get contract address
hal-simplicity simplicity simplicity info "<base64_program>"
# Output: JSON with address and CMR

# Step 2: Fund via faucet (via HTTP API)
curl "https://liquidtestnet.com/faucet?address=<contract_address>&action=lbtc"
# Output: HTML with transaction ID

# Step 3.1: Create base PSBT
elements-cli createpsbt '[{"txid":"<funding_txid>","vout":0}]' '[{"<destination>":<amount>}]'
# Output: Base64 PSBT

# Step 3.2: Get UTXO data
elements-cli gettxout <funding_txid> 0
# Output: JSON with scriptPubKey, asset, value

# Step 3.3: Update PSBT with UTXO data
elements-cli utxoupdatepsbt "<base_psbt>"
# Output: Updated base64 PSBT

# Step 4: Finalize PSBT (after external signing)
elements-cli finalizepsbt "<signed_psbt>"
# Output: JSON with transaction hex

# Step 5: Broadcast transaction
elements-cli sendrawtransaction "<tx_hex>"
# Output: Transaction ID
```

---

## Architecture

### File Structure

```
partnerfy_app/
├── src/
│   ├── app_core/
│   │   ├── elements_rpc.rs      # Elements RPC client (PSBT operations)
│   │   ├── hal_wrapper.rs       # hal-simplicity wrapper (Simplicity operations)
│   │   └── models.rs            # Data models
│   └── views/
│       └── p2ms.rs              # P2MS workflow UI
├── p2ms.simf                     # Simplicity source (2-of-3 multisig)
└── Cargo.toml
```

### Key Components

1. **ElementsRPC** (`src/app_core/elements_rpc.rs`):
   - `create_pset()`: Creates base PSBT using `elements-cli createpsbt`
   - `update_psbt_utxo()`: Updates PSBT with UTXO data using `elements-cli utxoupdatepsbt`
   - `finalize_pset()`: Finalizes PSBT using `elements-cli finalizepsbt`
   - `get_txout()`: Gets UTXO data using `elements-cli gettxout`
   - `send_raw_transaction()`: Broadcasts transaction via RPC

2. **HalWrapper** (`src/app_core/hal_wrapper.rs`):
   - `compile_simf()`: Compiles SimplicityHL source using `simc`
   - `get_covenant_info()`: Gets contract info using `hal-simplicity simplicity simplicity info`

3. **P2MS View** (`src/views/p2ms.rs`):
   - Orchestrates the complete workflow
   - Handles UI interactions
   - Manages state between steps

---

## Error Handling

The implementation includes comprehensive error handling:

1. **Command Not Found**: Checks for `elements-cli` and `hal-simplicity` in PATH and common locations
2. **RPC Errors**: Provides troubleshooting steps for connection issues
3. **Invalid Input**: Validates transaction IDs, addresses, and amounts
4. **UTXO Not Found**: Falls back to Blockstream API if `elements-cli gettxout` fails
5. **PSBT Errors**: Provides detailed error messages for PSBT operations

---

## Security Considerations

1. **Testnet Only**: The current implementation is designed for Liquid Testnet
2. **Private Keys**: Private keys are never stored or transmitted by the app
3. **External Signing**: PSBT signing is done externally in user's wallet
4. **Validation**: All inputs are validated before processing
5. **Error Messages**: Detailed error messages help diagnose issues without exposing sensitive data

---

## Troubleshooting

### elements-cli Not Found
- Check installation: `which elements-cli`
- Verify PATH: `echo $PATH`
- Common locations: `/usr/local/bin`, `/usr/bin`, `~/.cargo/bin`

### hal-simplicity Not Found
- Check installation: `which hal-simplicity`
- Verify PATH or specify full path

### RPC Connection Failed
- Ensure `elementsd` is running: `elements-cli getblockchaininfo`
- Check RPC credentials in `~/.elements/elements.conf`
- Verify network (testnet vs mainnet)

### PSBT Operations Fail
- Ensure PSBT is properly formatted (base64)
- Verify all required UTXO data is present
- Check that transaction inputs/outputs are valid

---

## References

- **Elements Core**: https://github.com/ElementsProject/elements
- **SimplicityHL**: https://github.com/ElementsProject/simplicity
- **hal-simplicity**: https://github.com/Blockstream/hal-simplicity
- **Liquid Testnet Faucet**: https://liquidtestnet.com/faucet
- **Blockstream Explorer**: https://blockstream.info/liquidtestnet
- **Elements RPC Docs**: https://elementsproject.org/en/doc/0.21.0.2/rpc/

---

## Notes

- All PSBT operations use `elements-cli` directly (not RPC) for better reliability
- Simplicity operations use `hal-simplicity` CLI tool
- The app automatically finds commands in PATH or common installation locations
- Error messages include troubleshooting steps and command examples
- The workflow supports interactive signing where users export/import PSBTs

