# Nevo Setup Guide

Complete setup instructions for the Nevo decentralized donation platform.

## Prerequisites

### Required Tools

1. **Node.js** (v16 or higher)
   - Download from [nodejs.org](https://nodejs.org/)
   - Verify: `node --version`

2. **Rust** (for smart contract development)
   - Install via rustup: https://rustup.rs/
   - Windows: Download and run rustup-init.exe
   - Verify: `rustc --version`

3. **Soroban CLI**
   ```bash
   cargo install --locked soroban-cli
   ```

4. **wasm32 target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

5. **Git**
   - Download from [git-scm.com](https://git-scm.com/)

## Project Setup

### 1. Clone the Repository

```bash
git clone https://github.com/Web3Novalabs/Nevo.git
cd Nevo
```

### 2. Smart Contract Setup

```bash
cd contract

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# The compiled WASM will be at:
# target/wasm32-unknown-unknown/release/crowdfunding.wasm
```

### 3. Frontend Setup

```bash
cd frontend

# Install dependencies
npm install
# or
yarn install
# or
pnpm install

# Run development server
npm run dev

# Open http://localhost:3000 in your browser
```

## Smart Contract Deployment

### Testnet Deployment

1. **Create a Stellar Account**
   ```bash
   # Generate a new keypair
   soroban keys generate --global alice --network testnet
   
   # Fund the account (testnet only)
   soroban keys fund alice --network testnet
   ```

2. **Deploy the Contract**
   ```bash
   soroban contract deploy \
     --wasm target/wasm32-unknown-unknown/release/crowdfunding.wasm \
     --source alice \
     --network testnet
   
   # Save the returned contract ID
   export CONTRACT_ID=<your_contract_id>
   ```

3. **Initialize the Contract**
   ```bash
   # Deploy a test token first (or use existing USDC)
   soroban contract asset deploy \
     --asset native \
     --source alice \
     --network testnet
   
   export TOKEN_ID=<your_token_id>
   
   # Initialize the crowdfunding contract
   soroban contract invoke \
     --id $CONTRACT_ID \
     --source alice \
     --network testnet \
     -- initialize \
     --admin $(soroban keys address alice) \
     --token $TOKEN_ID \
     --fee 0
   ```

### Mainnet Deployment

⚠️ **Warning**: Mainnet deployment requires real XLM and should only be done after thorough testing.

```bash
# Generate mainnet keys
soroban keys generate --global mainnet-deployer --network mainnet

# Deploy (same commands as testnet, but use --network mainnet)
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/crowdfunding.wasm \
  --source mainnet-deployer \
  --network mainnet
```

## Frontend Configuration

### Environment Variables

Create a `.env.local` file in the `frontend` directory:

```env
# Stellar Network Configuration
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_STELLAR_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Contract Addresses
NEXT_PUBLIC_CROWDFUNDING_CONTRACT_ID=your_contract_id_here
NEXT_PUBLIC_TOKEN_CONTRACT_ID=your_token_id_here

# Optional: Analytics, etc.
NEXT_PUBLIC_GA_ID=your_google_analytics_id
```

For mainnet:
```env
NEXT_PUBLIC_STELLAR_NETWORK=mainnet
NEXT_PUBLIC_STELLAR_RPC_URL=https://soroban-mainnet.stellar.org
NEXT_PUBLIC_NETWORK_PASSPHRASE=Public Global Stellar Network ; September 2015
```

## Development Workflow

### Smart Contract Development

1. **Make changes** to contract code in `contract/contract/src/`
2. **Run tests**: `cargo test`
3. **Build**: `cargo build --target wasm32-unknown-unknown --release`
4. **Deploy to testnet** for testing
5. **Test with frontend** integration

### Frontend Development

1. **Start dev server**: `npm run dev`
2. **Make changes** to components in `frontend/src/`
3. **Test in browser** at http://localhost:3000
4. **Build for production**: `npm run build`
5. **Test production build**: `npm start`

## Testing

### Smart Contract Tests

```bash
cd contract

# Run all tests
cargo test

# Run specific test file
cargo test --test crowdfunding_test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_campaign
```

### Frontend Tests

```bash
cd frontend

# Run linter
npm run lint

# Type checking
npx tsc --noEmit
```

## Common Issues & Solutions

### Issue: Cargo not found
**Solution**: Install Rust via rustup.rs and restart your terminal

### Issue: wasm32 target not found
**Solution**: Run `rustup target add wasm32-unknown-unknown`

### Issue: Soroban CLI not found
**Solution**: Run `cargo install --locked soroban-cli`

### Issue: Contract deployment fails
**Solution**: 
- Ensure your account is funded (testnet: use `soroban keys fund`)
- Check network connectivity
- Verify RPC URL is correct

### Issue: Frontend can't connect to contract
**Solution**:
- Verify contract ID in `.env.local`
- Check network configuration matches deployment
- Ensure wallet is connected to correct network

## Project Structure

```
Nevo/
├── contract/              # Soroban smart contract
│   ├── contract/
│   │   ├── src/          # Contract source code
│   │   └── test/         # Contract tests
│   └── README.md
├── frontend/             # Next.js frontend
│   ├── src/
│   │   ├── app/         # Next.js app router pages
│   │   ├── components/  # React components
│   │   └── lib/         # Utilities
│   └── package.json
├── README.md            # Main project README
├── SETUP.md            # This file
└── contributor.md      # Contribution guidelines
```

## Next Steps

1. ✅ Complete smart contract setup
2. ✅ Deploy to testnet
3. ✅ Configure frontend environment
4. 🔄 Integrate frontend with smart contract
5. 🔄 Test end-to-end flows
6. 🔄 Deploy to mainnet (when ready)

## Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Documentation](https://developers.stellar.org/)
- [Next.js Documentation](https://nextjs.org/docs)
- [Stellar Wallets Kit](https://github.com/Creit-Tech/Stellar-Wallets-Kit)

## Support

- GitHub Issues: https://github.com/Web3Novalabs/Nevo/issues
- Documentation: See README.md files in each directory

## License

MIT License - see [LICENSE](LICENSE) for details
