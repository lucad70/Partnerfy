# Partnerfy Application

**Partnerfy** is a desktop application for creating and managing Simplicity covenant-based contracts on Liquid Testnet. It provides two main workflows: **Multisig (P2MS)** and **Voucher (P2MS with Covenant)**.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Building](#building)
- [Running](#running)
- [Testing](#testing)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Project Structure](#project-structure)

## Overview

This is a **Dioxus desktop application** (Rust) that communicates with Elements (Liquid) via RPC and integrates SimplicityHL and hal-simplicity for covenant creation and validation.

**Key Features:**
- Generate and compile Simplicity source files for 2-of-3 multisig contracts
- Create contract addresses and fund them via the Liquid Testnet faucet
- Build, sign, and broadcast spending transactions
- Work with covenants that enforce specific output structures
- Automatic covenant inheritance for change outputs (voucher workflow)

## Prerequisites

Before running Partnerfy, ensure you have the following installed:

### Required Software

1. **Rust** (latest stable version)
2. **Dioxus CLI**
3. **Elements Core** (`elementsd` and `elements-cli`)
4. **SimplicityHL Compiler** (`simc`)
5. **hal-simplicity** (specific branch with PSET signing support)

## Installation

### 1. Install Rust

Install the latest stable version of Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

### 2. Install Dioxus CLI

Install the Dioxus CLI tool:

```bash
cargo install dioxus-cli
```

Verify installation:
```bash
dx --version
```

### 3. Install Elements Core (elementsd)

You need Elements Core to run a Liquid Testnet node.

**macOS (using Homebrew):**
```bash
brew install elements
```

**Linux (from source):**
```bash
git clone https://github.com/ElementsProject/elements.git
cd elements
./autogen.sh
./configure
make
sudo make install
```

Verify installation:
```bash
elementsd --version
elements-cli --version
```

### 4. Install SimplicityHL Compiler (simc)

Install the SimplicityHL compiler:

```bash
git clone https://github.com/ElementsProject/simplicity.git
cd simplicity
cargo build --release
sudo cp target/release/simc /usr/local/bin/
```

Verify installation:
```bash
simc --version
```

### 5. Install hal-simplicity

**Important**: This app requires a specific branch of hal-simplicity that includes PSET signing functionality.

```bash
git clone https://github.com/apoelstra/hal-simplicity.git
cd hal-simplicity
git checkout 2025-10/pset-signer
cargo build --locked --release
sudo cp target/release/hal-simplicity /usr/local/bin/
```

Verify installation:
```bash
hal-simplicity --version
```

**Note**: Ensure `hal-simplicity` is in your PATH. The app expects to find it as `hal-simplicity` in your system PATH.

## Configuration

### Setup Elements Node

1. **Create Elements configuration directory:**

   **macOS:**
   ```bash
   mkdir -p ~/Library/Application\ Support/Elements
   cp ../elements.conf.example ~/Library/Application\ Support/Elements/elements.conf
   ```

   **Linux:**
   ```bash
   mkdir -p ~/.elements
   cp ../elements.conf.example ~/.elements/elements.conf
   ```

2. **Edit the configuration file** (`elements.conf`) with your settings. The default configuration for Liquid Testnet:
   ```ini
   chain=liquidtestnet
   
   [liquidtestnet]
   daemon=1
   server=1
   listen=1
   txindex=1
   addnode=liquid.network:18444
   addnode=liquid-testnet.blockstream.com:18891
   addnode=liquidtestnet.com:18891
   rpcuser=user
   rpcpassword=password
   rpcport=18891
   ```

3. **Start elementsd:**

   **macOS:**
   ```bash
   elementsd -conf=~/Library/Application\ Support/Elements/elements.conf
   ```

   **Linux:**
   ```bash
   elementsd -conf=~/.elements/elements.conf
   ```

4. **Verify connection:**
   
   **macOS:**
   ```bash
   elements-cli -conf=~/Library/Application\ Support/Elements/elements.conf getblockchaininfo
   ```
   
   **Linux:**
   ```bash
   elements-cli -conf=~/.elements/elements.conf getblockchaininfo
   ```

5. **Get testnet funds:**
   - Visit: https://liquidtestnet.com/faucet
   - Request testnet LBTC to your wallet address

### RPC Configuration

The app uses the following default RPC settings (configured in `src/app_core/models.rs`):

- **RPC Host:** `localhost`
- **RPC Port:** `18891` (Liquid Testnet default)
- **RPC User:** `user`
- **RPC Password:** `password`

These match the default values in `elements.conf.example`. If you change your Elements RPC credentials, you'll need to modify the `Settings::default()` in `src/app_core/models.rs` or add environment variable support.

### Required Command-Line Tools

The app expects the following command-line tools to be available in your PATH:

- `elements-cli` - for PSET operations
- `hal-simplicity` - for Simplicity covenant operations
- `simc` - for compiling Simplicity source files

Verify all tools are accessible:
```bash
which elements-cli
which hal-simplicity
which simc
```

## Building

Build the application in release mode:

```bash
cd partnerfy_app
cargo build --release
```

The binary will be created at `target/release/partnerfy_app`.

**Note**: The first build may take several minutes as it compiles all dependencies.

## Running

### Development Mode

Run the application in development mode:

```bash
cargo run
```

### Release Mode

Run the application in release mode (recommended for testing):

```bash
cargo run --release
```

### Run Built Binary

Or run the built binary directly:

```bash
./target/release/partnerfy_app
```

The application will open in a desktop window. If the RPC client fails to initialize, you'll see an error message. Ensure `elementsd` is running before starting the app.

## Testing

Before using the app, ensure all prerequisites are met:

### Pre-Flight Checklist

1. **✅ elementsd is running:**
   
   **macOS:**
   ```bash
   elements-cli -conf=~/Library/Application\ Support/Elements/elements.conf getblockchaininfo
   ```
   
   **Linux:**
   ```bash
   elements-cli -conf=~/.elements/elements.conf getblockchaininfo
   ```
   
   You should see blockchain information (chain, blocks, etc.).

2. **✅ All tools are in PATH:**
   ```bash
   which elements-cli
   which hal-simplicity
   which simc
   ```
   
   All commands should return paths to the binaries.

3. **✅ You have testnet funds:**
   - Get LBTC from the Liquid Testnet faucet: https://liquidtestnet.com/faucet
   - Verify funds in your wallet:
     ```bash
     elements-cli -conf=~/.elements/elements.conf listunspent
     ```

4. **✅ RPC connection works:**
   - The app will attempt to connect to `localhost:18891` on startup
   - If connection fails, check your `elements.conf` settings

### Running the App

Once all checks pass:

```bash
cargo run --release
```

The application window should open. Navigate through the workflows using the landing page.

## Usage

The app provides two main workflows accessible from the landing page:

### Multisig (P2MS) Workflow

Create a 2-of-3 multisig contract where funds require 2 out of 3 signatures to spend.

**Step-by-step:**

1. **Generate P2MS Simplicity Source File**
   - Navigate to the P2MS workflow page
   - Enter output path for `.simf` file (e.g., `p2ms.simf`)
   - Provide three 32-byte public keys (64 hex characters each)
   - Click "Generate p2ms.simf File"

2. **Compile Simplicity Source (Optional)**
   - Enter path to `.simf` file
   - Click "Compile .simf File"
   - Or paste a pre-compiled program directly

3. **Create P2MS Contract Address**
   - Paste compiled program (base64)
   - Click "Create Contract Address"
   - Copy the contract address and CMR (Commitment Merkle Root)

4. **Fund Contract Address via Faucet**
   - Enter amount (default: 0.001 L-BTC)
   - Click "Fund via Faucet"
   - Wait for funding transaction confirmation
   - Note the funding transaction ID

5. **Create Spending PSET**
   - Enter destination address and amount
   - Provide internal key (default provided)
   - Click "Create and Update PSET"
   - The app will wait for UTXO confirmation and create the PSET

6. **Sign and Finalize Transaction**
   - Provide witness file path (`.wit`)
   - Enter at least 2 of 3 private keys (matching the public keys from step 1)
   - Click "Sign and Finalize Transaction"
   - The app will compile the program with witness and finalize the PSET

7. **Broadcast Transaction**
   - Click "Broadcast Transaction"
   - View transaction on Blockstream explorer: https://blockstream.info/liquidtestnet

### Voucher (P2MS with Covenant) Workflow

Similar to P2MS, but with covenant enforcement that automatically locks change outputs.

**Step-by-step:**

1. **Generate Voucher Simplicity Source File**
   - Navigate to the Voucher workflow page
   - Creates `cov_p2ms.simf` with covenant structure
   - Covenant enforces exactly 3 outputs: payment, recursive covenant, and fee

2. **Compile and Create Contract Address**
   - Same as P2MS workflow (steps 2-3)

3. **Fund Contract Address**
   - Same as P2MS workflow (step 4)

4. **Create Spending PSET**
   - Must create exactly 3 outputs:
     - Output 0: Payment to destination
     - Output 1: Recursive covenant (change)
     - Output 2: Fee output
   - The covenant will verify this structure during finalization

5. **Sign and Finalize**
   - Same as P2MS workflow (step 6)
   - Covenant verifies 3-output structure during finalization

6. **Broadcast Transaction**
   - Same as P2MS workflow (step 7)
   - Change automatically returns to the same covenant

## Troubleshooting

### RPC Connection Failed

**Symptoms:** App shows "Failed to initialize RPC client" on startup.

**Solutions:**
- Ensure `elementsd` is running: `elements-cli getblockchaininfo`
- Check RPC credentials in `~/.elements/elements.conf` (or `~/Library/Application Support/Elements/elements.conf` on macOS)
- Verify RPC port matches configuration (18891 for testnet)
- Check that `rpcuser` and `rpcpassword` match the defaults (`user`/`password`) or update `Settings::default()` in `src/app_core/models.rs`
- Test RPC connection manually:
  ```bash
  curl --user user:password --data-binary '{"jsonrpc":"1.0","id":"test","method":"getblockchaininfo","params":[]}' -H 'content-type: text/plain;' http://localhost:18891
  ```

### hal-simplicity Not Found

**Symptoms:** Error when trying to create contract address or finalize PSET.

**Solutions:**
- Verify installation: `which hal-simplicity`
- Check that you built from the correct branch: `2025-10/pset-signer`
- Ensure binary is in PATH: `ls -la /usr/local/bin/hal-simplicity`
- Try running directly: `/usr/local/bin/hal-simplicity --version`
- Reinstall if needed:
  ```bash
  cd hal-simplicity
  git checkout 2025-10/pset-signer
  cargo build --locked --release
  sudo cp target/release/hal-simplicity /usr/local/bin/
  ```

### simc Not Found

**Symptoms:** Error when trying to compile Simplicity source files.

**Solutions:**
- Verify installation: `which simc`
- Check PATH: `echo $PATH`
- Ensure binary is in PATH: `ls -la /usr/local/bin/simc`
- Reinstall if needed:
  ```bash
  cd simplicity
  cargo build --release
  sudo cp target/release/simc /usr/local/bin/
  ```

### elements-cli Not Found

**Symptoms:** Error when trying to create PSET or broadcast transactions.

**Solutions:**
- Verify installation: `which elements-cli`
- Check PATH: `echo $PATH`
- Common locations: `/usr/local/bin`, `/usr/bin`, `~/.cargo/bin`
- On macOS with Homebrew: `/opt/homebrew/bin` or `/usr/local/bin`
- Add to PATH if needed:
  ```bash
  export PATH="/usr/local/bin:$PATH"
  ```

### Covenant Compilation Errors

**Symptoms:** Error when compiling `.simf` files.

**Solutions:**
- Check SimplicityHL syntax in your `.simf` file
- Ensure you're using a compatible version of SimplicityHL
- Verify public keys are exactly 64 hex characters (32 bytes)
- Check for syntax errors in the Simplicity source
- Refer to SimplicityHL documentation for correct syntax

### PSET Operations Fail

**Symptoms:** Error when creating or updating PSET.

**Solutions:**
- Ensure PSET is properly formatted (base64)
- Verify all required UTXO data is present
- Check that transaction inputs/outputs are valid
- Ensure elements-cli version is compatible
- Verify UTXO exists and is confirmed:
  ```bash
  elements-cli gettxout <txid> <vout>
  ```

### Signature Errors

**Symptoms:** Error "Jet failed during execution" when finalizing PSET.

**Solutions:**
- Verify private keys match the public keys in your contract
- Ensure you have at least 2 valid signatures for 2-of-3 multisig
- Check that signatures are PSET-specific (don't modify PSET after signing)
- Verify witness file format is correct (JSON with `MAYBE_SIGS` field)
- Ensure signatures are in the correct positions in the witness array

### UTXO Not Found

**Symptoms:** Error when trying to create spending PSET after funding.

**Solutions:**
- Wait for transaction confirmation (may take 1-2 minutes)
- Verify funding transaction exists:
  ```bash
  elements-cli getrawtransaction <txid> true
  ```
- Check transaction on explorer: https://blockstream.info/liquidtestnet/tx/<txid>
- The app will automatically retry up to 20 times (5 second intervals)

## Project Structure

```
partnerfy_app/
├── src/
│   ├── app_core/           # Core business logic
│   │   ├── elements_rpc.rs    # Elements RPC client
│   │   ├── tx_builder.rs      # Transaction construction
│   │   ├── witness.rs         # Witness generation
│   │   ├── hal_wrapper.rs     # hal-simplicity CLI wrapper
│   │   └── models.rs          # Data models and settings
│   ├── views/              # UI components
│   │   ├── landing.rs         # Landing page
│   │   ├── p2ms.rs           # P2MS workflow page
│   │   ├── voucher.rs        # Voucher workflow page
│   │   ├── instructions.rs   # Instructions page
│   │   └── navbar.rs         # Navigation
│   ├── components/         # Reusable UI components
│   │   ├── echo.rs
│   │   └── hero.rs
│   └── main.rs             # App entry point
├── assets/                 # Static assets
│   ├── styling/            # CSS files
│   ├── favicon.ico
│   └── tailwind.css
├── Cargo.toml              # Rust dependencies
├── Dioxus.toml            # Dioxus configuration
└── README.md              # This file
```

## Resources

- **Liquid Testnet Faucet**: https://liquidtestnet.com/faucet
- **Liquid Testnet Explorer**: https://blockstream.info/liquidtestnet
- **Elements RPC Documentation**: https://elementsproject.org/en/doc/0.21.0.2/rpc/
- **Simplicity Documentation**: https://docs.liquid.net
- **hal-simplicity (fork)**: https://github.com/apoelstra/hal-simplicity (branch: `2025-10/pset-signer`)
- **SimplicityHL Compiler**: https://github.com/ElementsProject/simplicity
- **Elements Core**: https://github.com/ElementsProject/elements
- **Dioxus**: https://dioxuslabs.com

## Security Notes

⚠️ **Important**: This application is designed for **Liquid Testnet only**. Never use on mainnet until thoroughly tested.

- **Always test on Liquid Testnet first** - Never use mainnet until thoroughly tested
- **Store private keys securely** - Encrypt locally, never share them
- **Validate witness correctness** - Check signatures before broadcasting
- **Verify covenant structure** - For Voucher contracts, ensure 3 outputs are correct
- **Keep transaction logs** - Maintain off-chain records of all operations

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
