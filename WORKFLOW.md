# P2MS Workflow Analysis

## Overview

The P2MS (Pay-to-Multisig) workflow creates a 2-of-3 multisig contract on Liquid Testnet using Simplicity covenants. Funds can only be spent when 2 out of 3 participants provide their signatures.

## Complete Workflow Steps

### Step 0: Compile Simplicity Source (Optional)

**Purpose**: Compile SimplicityHL source file to base64 program.

**CLI Command**:
```bash
simc <input.simf>
```

**Implementation**: `hal_wrapper.rs::compile_simf()`
- Executes: `simc <input_path>`
- Output: Base64-encoded program (last line of stdout)

---

### Step 1: Create P2MS Contract Address

**Purpose**: Generate Liquid Testnet address from compiled program.

**CLI Command**:
```bash
hal-simplicity simplicity info "<program_base64>"
```

**Implementation**: `hal_wrapper.rs::get_covenant_info()`
- Output: JSON with `cmr` and `liquid_testnet_address_unconf`

**Example Output**:
```json
{
  "cmr": "af5b897effb80a06fa19362347b7807dc0e774eaf4271d6526545965b44ddc3e",
  "liquid_testnet_address_unconf": "tex1pfsldwgyf5e2gmg0lvm87zz2zqswk4h2prs63lzxlsgz5lsurgy8s3az3wv"
}
```

---

### Step 2: Fund Contract Address via Faucet

**Purpose**: Send testnet L-BTC to contract address.

**HTTP Request**:
```bash
curl "https://liquidtestnet.com/faucet?address=<contract_address>&action=lbtc"
```

**Implementation**: `p2ms.rs::fund_via_faucet()`
- Method: HTTP GET using `reqwest::Client`
- Parses HTML response with regex to extract transaction ID
- Patterns: `r"transaction\s+([a-f0-9]{64})"` or `r"txid[:\s]+([a-f0-9]{64})"`

**Output**: Funding transaction ID (txid), VOUT (typically 0), amount

---

### Step 3: Create Spending PSET

#### 3.1: Wait for UTXO and Get UTXO Data

**CLI Command**:
```bash
elements-cli gettxout <funding_txid> 0
```

**Implementation**: `elements_rpc.rs::get_txout()`
- Polls up to 20 times (5 second intervals) waiting for UTXO
- Falls back to Blockstream API if RPC fails:
  ```bash
  curl "https://blockstream.info/liquidtestnet/api/tx/<txid>"
  ```

**Output**: JSON with `scriptPubKey.hex`, `asset`, `value` (in BTC, converted to sats)

#### 3.2: Create Base PSET

**CLI Command**:
```bash
elements-cli createpsbt '[{"txid":"<txid>","vout":0}]' '[{"<destination>":<amount>},{"fee":<fee_amount>}]'
```

**Implementation**: `elements_rpc.rs::create_pset()`
- Calculates fee: `fee_sats = utxo_value_sats - spend_amount_sats`
- Validates minimum fee (100 sats)
- Output: Base64-encoded PSET

#### 3.3: Update PSET with Simplicity Data

**CLI Command**:
```bash
hal-simplicity simplicity pset update-input <pset> 0 -i <scriptPubKey_hex>:<asset>:<value_sats> -c <cmr> -p <internal_key>
```

**Implementation**: `hal_wrapper.rs::update_pset_input()`
- **CRITICAL**: Value must be in sats (not BTC) for correct message hash
- Internal key: Default `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
- Output: Updated PSET (JSON with `pset` field)

---

### Step 4: Sign and Finalize Transaction

#### 4.1: Sign with Private Keys

**CLI Command** (for each private key):
```bash
hal-simplicity simplicity sighash <pset> 0 <cmr> -x <privkey_hex>
```

**Implementation**: `hal_wrapper.rs::sighash_and_sign()`
- Strips `0x` prefix from private key if present
- Output: JSON with `signature` field (hex string without 0x prefix)
- Requires at least 2 signatures for 2-of-3 multisig

**Key Mapping**:
- `privkey_1` → Position 0 (pk1 = 0x79be667e... = 1*G)
- `privkey_2` → Position 1 (pk2 = 0xc6047f94... = 2*G)
- `privkey_3` → Position 2 (pk3 = 0xf9308a01... = 3*G)

#### 4.2: Update Witness File

**Witness File Format** (`p2ms.wit`):
```json
{
  "MAYBE_SIGS": {
    "value": "[Some(0x<sig1>), None, Some(0x<sig3>)]",
    "type": "[Option<Signature>; 3]"
  }
}
```

**Implementation**: `p2ms.rs::sign_and_finalize()`
- Updates witness file with signatures in correct positions
- Ensures at least 2 signatures are present

#### 4.3: Compile Program with Witness

**CLI Command**:
```bash
simc <input.simf> <witness.wit>
```

**Implementation**: `hal_wrapper.rs::compile_simf_with_witness()`
- Output format:
  ```
  Program:
  <program_base64>
  Witness:
  <witness_base64>
  ```

#### 4.4: Finalize PSET with Simplicity

**CLI Command**:
```bash
hal-simplicity simplicity pset finalize <pset> 0 <program_base64> <witness_base64>
```

**Implementation**: `hal_wrapper.rs::finalize_pset_with_witness()`
- Output: JSON with `pset` field (finalized PSET)
- Validates signatures match public keys
- Error: "Jet failed during execution" if signatures don't match

#### 4.5: Finalize PSBT

**CLI Command**:
```bash
elements-cli finalizepsbt "<pset>"
```

**Implementation**: `elements_rpc.rs::finalize_pset()`
- Output: JSON with `hex` field (raw transaction hex)
- Error if PSET is incomplete or invalid

---

### Step 5: Broadcast Transaction

**CLI Command** (Primary):
```bash
elements-cli sendrawtransaction "<tx_hex>"
```

**RPC Call** (Alternative):
```json
{
  "method": "sendrawtransaction",
  "params": ["<tx_hex>"]
}
```

**HTTP API** (Fallback):
```bash
curl -X POST "https://blockstream.info/liquidtestnet/api/tx" -d "<tx_hex>"
```

**Implementation**: `elements_rpc.rs::send_raw_transaction()`
- Tries RPC first, then Blockstream API
- Output: Transaction ID (txid)

---

## Complete Command Sequence

```bash
# Step 0: Compile Simplicity source
simc p2ms.simf
# Output: Base64 program

# Step 1: Get contract address
hal-simplicity simplicity info "<base64_program>"
# Output: JSON with address and CMR

# Step 2: Fund via faucet
curl "https://liquidtestnet.com/faucet?address=<contract_address>&action=lbtc"
# Output: HTML with transaction ID

# Step 3.1: Wait for UTXO (polling)
while ! elements-cli gettxout <funding_txid> 0 | grep . >/dev/null; do sleep 5; done

# Step 3.2: Get UTXO data
elements-cli gettxout <funding_txid> 0
# Output: JSON with scriptPubKey, asset, value

# Step 3.3: Create base PSET
elements-cli createpsbt '[{"txid":"<funding_txid>","vout":0}]' '[{"<destination>":<amount>},{"fee":<fee>}]'
# Output: Base64 PSET

# Step 3.4: Update PSET with Simplicity data
hal-simplicity simplicity pset update-input <base_pset> 0 -i <hex>:<asset>:<value_sats> -c <cmr> -p <internal_key>
# Output: JSON with updated PSET

# Step 4.1: Sign with private keys (at least 2 required)
hal-simplicity simplicity sighash <pset> 0 <cmr> -x <privkey_1>
hal-simplicity simplicity sighash <pset> 0 <cmr> -x <privkey_2>
# Output: JSON with signatures

# Step 4.2: Update witness file (manually or programmatically)
# Edit p2ms.wit to include signatures: [Some(0x<sig1>), None, Some(0x<sig2>)]

# Step 4.3: Compile with witness
simc p2ms.simf p2ms.wit
# Output: Program and Witness base64

# Step 4.4: Finalize PSET with Simplicity
hal-simplicity simplicity pset finalize <pset> 0 <program_base64> <witness_base64>
# Output: JSON with finalized PSET

# Step 4.5: Finalize PSBT
elements-cli finalizepsbt "<finalized_pset>"
# Output: JSON with transaction hex

# Step 5: Broadcast transaction
elements-cli sendrawtransaction "<tx_hex>"
# Output: Transaction ID
```

---

## Key Implementation Details

### Value Conversion

- **elements-cli gettxout**: Returns value in BTC (decimal, e.g., 0.001)
- **hal-simplicity**: Requires value in sats (integer, e.g., 100000)
- **Conversion**: `value_sats = value_btc * 100_000_000`

### Signature Requirements

- **2-of-3 multisig**: Requires exactly 2 valid signatures
- **Public keys** (hardcoded in p2ms.simf):
  - pk1: `0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798` (1*G)
  - pk2: `0xc6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5` (2*G)
  - pk3: `0xf9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9` (3*G)

### PSET Workflow

1. Create base PSET with `elements-cli createpsbt`
2. Update with Simplicity data using `hal-simplicity pset update-input`
3. Sign with `hal-simplicity sighash` (generates signatures)
4. Update witness file with signatures
5. Compile program with witness using `simc`
6. Finalize with `hal-simplicity pset finalize`
7. Finalize PSBT with `elements-cli finalizepsbt`

### Error Handling

- **UTXO not found**: Polls up to 20 times (100 seconds), then falls back to Blockstream API
- **Command not found**: Checks PATH and common locations, provides troubleshooting
- **Signature errors**: Validates private keys match public keys, provides detailed error messages
- **PSET errors**: Validates format, provides deserialization troubleshooting

---

## File Locations

- **Source Code**:
  - `partnerfy_app/src/views/p2ms.rs` - Main workflow UI
  - `partnerfy_app/src/app_core/hal_wrapper.rs` - hal-simplicity wrapper
  - `partnerfy_app/src/app_core/elements_rpc.rs` - Elements RPC client
- **Simplicity Files**:
  - `p2ms.simf` - Simplicity source (2-of-3 multisig)
  - `p2ms.wit` - Witness file template

---

## Dependencies

- **simc**: SimplicityHL compiler
- **hal-simplicity**: Simplicity program operations
- **elements-cli**: Elements/Liquid CLI tool
- **elementsd**: Elements daemon (for RPC calls)

---

## References

- Elements Core: https://github.com/ElementsProject/elements
- SimplicityHL: https://github.com/ElementsProject/simplicity
- hal-simplicity: https://github.com/Blockstream/hal-simplicity
- Liquid Testnet Faucet: https://liquidtestnet.com/faucet
- Blockstream Explorer: https://blockstream.info/liquidtestnet

