# Partnerfy

**Partnerfy** is a desktop app built for working with Simplicity contracts on Liquid Testnet. It provides two main workflows: **Multisig (P2MS)** and **Voucher (P2MS with Covenant)** for creating and managing covenant-based transactions.

## Overview

This is a Dioxus desktop application (Rust) that communicates with Elements (Liquid) via RPC and integrates SimplicityHL and hal-simplicity for covenant creation and validation. The app allows you to:

- Generate and compile Simplicity source files for 2-of-3 multisig contracts
- Create contract addresses and fund them via the Liquid Testnet faucet
- Build, sign, and broadcast spending transactions
- Work with covenants that enforce specific output structures

## Prerequisites

Before running Partnerfy, you need to install and configure the following:

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

You need Elements Core to run a Liquid Testnet node:

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

I use a specific branch of hal-simplicity that includes PSET signing functionality. Install it as follows:

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

**Important:** Make sure `hal-simplicity` is in your PATH. This app expects to find it as `hal-simplicity` in your system PATH.

### 6. Setup Elements Node

1. Create Elements configuration directory:
   
   **macOS:**
   ```bash
   mkdir -p ~/Library/Application\ Support/Elements
   cp elements.conf.example ~/Library/Application\ Support/Elements/elements.conf
   ```
   
   **Linux:**
   ```bash
   mkdir -p ~/.elements
   cp elements.conf.example ~/.elements/elements.conf
   ```

2. Edit the configuration file with your settings. The default configuration for Liquid Testnet:
   ```
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

4. Start elementsd:
   
   **macOS:**
   ```bash
   elementsd -conf=~/Library/Application\ Support/Elements/elements.conf
   ```
   
   **Linux:**
   ```bash
   elementsd -conf=~/.elements/elements.conf
   ```

5. Verify connection:
   
   **macOS:**
   ```bash
   elements-cli -conf=~/Library/Application\ Support/Elements/elements.conf getblockchaininfo
   ```
   
   **Linux:**
   ```bash
   elements-cli -conf=~/.elements/elements.conf getblockchaininfo
   ```

6. Get testnet funds:
   - Visit: https://liquidtestnet.com/faucet
   - Request testnet LBTC to your wallet address

## Environment Variables

My app uses the following default RPC settings (configured in code):

- **RPC Host:** `localhost`
- **RPC Port:** `18891` (Liquid Testnet default)
- **RPC User:** `user`
- **RPC Password:** `password`

These match the default values in `elements.conf.example`. If you change your Elements RPC credentials, you'll need to modify the `Settings::default()` in `src/app_core/models.rs` or add environment variable support.

My app expects the following command-line tools to be available in your PATH:
- `elements-cli` - for PSET operations
- `hal-simplicity` - for Simplicity covenant operations
- `simc` - for compiling Simplicity source files

## Building

Build the application:

```bash
cd partnerfy_app
cargo build --release
```

The binary will be created at `target/release/partnerfy_app`.

## Running

Run the application:

```bash
cargo run --release
```

Or run the built binary directly:
```bash
./target/release/partnerfy_app
```

## Testing

Before using the app, ensure:

1. **elementsd is running:**
   
   **macOS:**
   ```bash
   elements-cli -conf=~/Library/Application\ Support/Elements/elements.conf getblockchaininfo
   ```
   
   **Linux:**
   ```bash
   elements-cli -conf=~/.elements/elements.conf getblockchaininfo
   ```

2. **All tools are in PATH:**
   ```bash
   which elements-cli
   which hal-simplicity
   which simc
   ```

3. **You have testnet funds:**
   - Get LBTC from the Liquid Testnet faucet: https://liquidtestnet.com/faucet

4. **Run the app:**
   ```bash
   cargo run --release
   ```

## Usage

The app provides two main workflows accessible from the landing page:

### Multisig (P2MS) Workflow

1. **Generate P2MS Simplicity Source File**
   - Enter output path for `.simf` file
   - Provide three 32-byte public keys (64 hex characters each)
   - Click "Generate p2ms.simf File"

2. **Compile Simplicity Source (Optional)**
   - Enter path to `.simf` file
   - Click "Compile .simf File"
   - Or paste a pre-compiled program directly

3. **Create P2MS Contract Address**
   - Paste compiled program (base64)
   - Click "Create Contract Address"
   - Copy the contract address and CMR

4. **Fund Contract Address via Faucet**
   - Enter amount (default: 0.001 L-BTC)
   - Click "Fund via Faucet"
   - Wait for funding transaction confirmation

5. **Create Spending PSET**
   - Enter destination address and amount
   - Provide internal key (default provided)
   - Click "Create and Update PSET"

6. **Sign and Finalize Transaction**
   - Provide witness file path (`.wit`)
   - Enter at least 2 of 3 private keys
   - Click "Sign and Finalize Transaction"

7. **Broadcast Transaction**
   - Click "Broadcast Transaction"
   - View transaction on Blockstream explorer

### Voucher (P2MS with Covenant) Workflow

Similar to P2MS, but with covenant enforcement:

1. **Generate Voucher Simplicity Source File**
   - Creates `cov_p2ms.simf` with covenant structure
   - Covenant enforces exactly 3 outputs: payment, recursive covenant, and fee

2. **Compile and Create Contract Address**
   - Same as P2MS workflow

3. **Fund Contract Address**
   - Same as P2MS workflow

4. **Create Spending PSET**
   - Must create exactly 3 outputs:
     - Output 0: Payment to destination
     - Output 1: Recursive covenant (change)
     - Output 2: Fee output

5. **Sign and Finalize**
   - Covenant verifies 3-output structure during finalization

6. **Broadcast Transaction**
   - Change automatically returns to the same covenant

## Project Structure

```
partnerfy_app/
├── src/
│   ├── app_core/         # Core business logic
│   │   ├── elements_rpc.rs    # Elements RPC client
│   │   ├── tx_builder.rs     # Transaction construction
│   │   ├── witness.rs        # Witness generation
│   │   ├── hal_wrapper.rs    # hal-simplicity CLI wrapper
│   │   └── models.rs         # Data models
│   ├── views/            # UI components
│   │   ├── landing.rs         # Landing page
│   │   ├── p2ms.rs           # P2MS workflow page
│   │   ├── voucher.rs        # Voucher workflow page
│   │   ├── instructions.rs   # Instructions page
│   │   └── navbar.rs         # Navigation
│   ├── components/       # Reusable UI components
│   └── main.rs           # App entry point
├── assets/               # Static assets (CSS, images)
└── Cargo.toml           # Rust dependencies
```

## Troubleshooting

### RPC Connection Failed

- Ensure `elementsd` is running: `elements-cli getblockchaininfo`
- Check RPC credentials in `~/.elements/elements.conf` (or `~/Library/Application Support/Elements/elements.conf` on macOS)
- Verify RPC port matches configuration (18891 for testnet)
- Check that `rpcuser` and `rpcpassword` match the defaults or update the code

### hal-simplicity Not Found

- Verify installation: `which hal-simplicity`
- Check that you built from the correct branch: `2025-10/pset-signer`
- Ensure binary is in PATH: `ls -la /usr/local/bin/hal-simplicity`
- Try running directly: `/usr/local/bin/hal-simplicity --version`

### simc Not Found

- Verify installation: `which simc`
- Check PATH: `echo $PATH`
- Ensure binary is in PATH: `ls -la /usr/local/bin/simc`
- Reinstall if needed: `sudo cp target/release/simc /usr/local/bin/`

### elements-cli Not Found

- Verify installation: `which elements-cli`
- Check PATH: `echo $PATH`
- Common locations: `/usr/local/bin`, `/usr/bin`, `~/.cargo/bin`

### Covenant Compilation Errors

- Check SimplicityHL syntax in your `.simf` file
- Ensure you're using a compatible version of SimplicityHL
- Verify public keys are exactly 64 hex characters (32 bytes)
- Refer to SimplicityHL documentation for correct syntax

### PSET Operations Fail

- Ensure PSET is properly formatted (base64)
- Verify all required UTXO data is present
- Check that transaction inputs/outputs are valid
- Ensure elements-cli version is compatible

### Signature Errors

- Verify private keys match the public keys in your contract
- Ensure you have at least 2 valid signatures for 2-of-3 multisig
- Check that signatures are PSET-specific (don't modify PSET after signing)
- Verify witness file format is correct

## Security Notes

- **Always test on Liquid Testnet first** - Never use mainnet until thoroughly tested
- **Store private keys securely** - Encrypt locally, never share them
- **Validate witness correctness** - Check signatures before broadcasting
- **Verify covenant structure** - For Voucher contracts, ensure 3 outputs are correct
- **Keep transaction logs** - Maintain off-chain records of all operations

## Resources

- **Liquid Testnet Faucet**: https://liquidtestnet.com/faucet
- **Liquid Testnet Explorer**: https://blockstream.info/liquidtestnet
- **Elements RPC Documentation**: https://elementsproject.org/en/doc/0.21.0.2/rpc/
- **Simplicity Documentation**: https://docs.liquid.net
- **hal-simplicity (my fork)**: https://github.com/apoelstra/hal-simplicity (branch: `2025-10/pset-signer`)
- **SimplicityHL Compiler**: https://github.com/ElementsProject/simplicity
- **Elements Core**: https://github.com/ElementsProject/elements
- **Dioxus**: https://dioxuslabs.com

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
