# Partnerfy

**Partnerfy** is a **desktop app** that issues, manages, and redeems **covenant-based vouchers** on **Liquid Testnet**.  
Each voucher is a UTXO locked under a **Simplicity covenant**, ensuring that **any change output** created when the participant spends the voucher **inherits the same spending conditions** as the parent (i.e., remains locked by the same covenant).

## Overview

This system is implemented as a **Dioxus** desktop application (Rust) communicating with **Elements (Liquid)** via RPC and integrating **SimplicityHL** and **hal-simplicity** for covenant creation and validation.

### Core Concept

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
|------|-------------|
| **Promoter** | Deploys the Simplicity covenant, funds voucher pool, distributes voucher UTXOs. |
| **Participant** | Receives vouchers, redeems them at partner locations, inherits covenant on change. |
| **Partner** | Accepts redemption transactions, validates them, and receives funds. |

## Prerequisites

### Required Software

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Elements Core** (elementsd)
   - Download from: https://github.com/ElementsProject/elements
   - Or use package manager: `brew install elements` (macOS)

3. **SimplicityHL Compiler** (simc)
   - Download from: https://github.com/ElementsProject/simplicity
   - Build from source or use pre-built binaries

4. **hal-simplicity**
   - Download from: https://github.com/Blockstream/hal-simplicity
   - Required for covenant info and witness generation

### Setup Elements Node

1. Create Elements configuration file:
   ```bash
   mkdir -p ~/.elements
   cp elements.conf.example ~/.elements/elements.conf
   # Edit ~/.elements/elements.conf with your settings
   ```

2. Start elementsd:
   ```bash
   elementsd -conf=~/.elements/elements.conf
   ```

3. Verify connection:
   ```bash
   elements-cli -conf=~/.elements/elements.conf getblockchaininfo
   ```

4. Get testnet funds:
   - Visit: https://liquidtestnet.com/faucet
   - Request testnet LBTC to your wallet

## Building

```bash
cd partnerfy_app
cargo build --release
```

## Running

```bash
cargo run --release
```

Or run the built binary:
```bash
./target/release/partnerfy_app
```

## Usage

### Promoter Flow

1. **Compile Covenant**
   - Write SimplicityHL source in `voucher.simf`
   - Compile with: `simc voucher.simf -o voucher.base64`
   - Load the compiled covenant in the Promoter panel

2. **Load Covenant Info**
   - Click "Load Covenant Info" to extract address and script details
   - Copy the covenant address

3. **Fund Covenant Pool**
   - Enter funding amount
   - Click "Fund Covenant" to send LBTC to the covenant address

4. **Issue Vouchers**
   - Enter comma-separated voucher amounts (e.g., "0.01, 0.01, 0.01")
   - Click "Create Vouchers" to split the funding UTXO into vouchers
   - Assign vouchers to participants (off-chain mapping)

### Participant Flow

1. **Import Voucher**
   - Import voucher UTXO information (txid:vout) and covenant details

2. **Redeem Voucher**
   - Select a voucher from your list
   - Enter partner address and redemption amount
   - Click "Redeem Voucher" to build the transaction
   - Sign the transaction with your witness
   - Send to partner for co-signature

### Partner Flow

1. **Set Partner Address**
   - Enter your P2PKH address in the Partner panel

2. **Verify Transaction**
   - Receive transaction hex from participant
   - Paste into the "Transaction Hex" field
   - Click "Validate Transaction" to check covenant compliance

3. **Broadcast Transaction**
   - If validation passes, click "Broadcast Transaction"
   - Transaction will be sent to the network

## Configuration

Default RPC settings (Liquid Testnet):
- Host: `localhost`
- Port: `18884`
- User: `user`
- Password: `pass`

To change settings, modify the `Settings::default()` in `src/core/models.rs` or add a configuration file loader.

## Project Structure

```
partnerfy_app/
├── src/
│   ├── core/           # Core business logic
│   │   ├── elements_rpc.rs  # Elements RPC client
│   │   ├── tx_builder.rs    # Transaction construction
│   │   ├── witness.rs       # Witness generation
│   │   ├── hal_wrapper.rs   # hal-simplicity CLI wrapper
│   │   └── models.rs        # Data models
│   ├── views/          # UI components
│   │   ├── promoter.rs      # Promoter panel
│   │   ├── participant.rs   # Participant panel
│   │   ├── partner.rs       # Partner panel
│   │   └── navbar.rs        # Navigation
│   └── main.rs         # App entry point
├── voucher.simf       # Simplicity covenant source (template)
└── Cargo.toml
```

## Testing

Run on Liquid Testnet first:

```bash
# Ensure elementsd is running on testnet
elements-cli getblockchaininfo

# Run the app
cargo run --release
```

## Security Notes

- **Always test with Liquid Testnet before mainnet**
- Store private keys securely, encrypted locally
- Validate witness correctness before broadcast
- Keep an off-chain log of vouchers and redemption events
- Verify covenant recursion: inspect each child UTXO's script matches parent's covenant hash

## Troubleshooting

### RPC Connection Failed
- Ensure `elementsd` is running
- Check RPC credentials in `~/.elements/elements.conf`
- Verify RPC port matches configuration (18884 for testnet)

### hal-simplicity Not Found
- Ensure `hal-simplicity` is in your PATH
- Or specify path in the app configuration

### Covenant Compilation Errors
- Check SimplicityHL syntax in `voucher.simf`
- Ensure you're using a compatible version of SimplicityHL
- Refer to SimplicityHL documentation for correct syntax

## Resources

- **Liquid Testnet Faucet**: https://liquidtestnet.com/faucet
- **Liquid Testnet Explorer**: https://blockstream.info/liquidtestnet
- **Elements Docs (RPC)**: https://elementsproject.org/en/doc/0.21.0.2/rpc/
- **Simplicity Docs**: https://docs.liquid.net
- **hal-simplicity GitHub**: https://github.com/Blockstream/hal-simplicity
- **SimplicityHL Compiler**: https://github.com/ElementsProject/simplicity

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
