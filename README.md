# Nevo edit

> Decentralized Donation Pools on Stellar

Nevo empowers users to create secure donation pools on the Stellar blockchain using Soroban smart contracts. Each pool has a unique address, supporting public or private contributions in XLM, USDC, or custom assets. Funds earn DeFi yields until disbursed to verified causes, with full transparency via Stellar's ledger.

## What is Nevo?

Nevo is a decentralized platform that reimagines charitable giving through blockchain technology. Create transparent donation pools, accept multiple assets, and let idle funds generate yields while maintaining complete control over disbursements.

## Why Soroban?

We built Nevo on Soroban, Stellar's smart contract platform, because it offers:

- **Low-cost transactions**: ~$0.00001 per transaction
- **Lightning-fast finality**: 3-5 second confirmation times
- **Built-in multi-asset support**: Native XLM, USDC, and custom tokens
- **Battle-tested infrastructure**: Leverage Stellar's proven network
- **Smart contract flexibility**: Complex logic with Rust security
- **Seamless DeFi integration**: Access Stellar's growing DeFi ecosystem

## Features

- Create donation pools with unique Stellar addresses
- Accept XLM, USDC, or custom Stellar assets
- Public or private contribution modes
- Automated DeFi yield generation on idle funds
- Transparent disbursements to verified causes
- Full transaction history on Stellar's ledger

## Why Nevo?

Traditional donation platforms charge high fees and lack transparency. Nevo solves this by:

- **Minimizing costs**: Stellar's tiny fees mean more money reaches causes
- **Ensuring transparency**: Every transaction is publicly verifiable
- **Maximizing impact**: DeFi yields grow donation pools over time
- **Building trust**: Smart contracts automate fund management
- **Enabling accessibility**: No traditional banking required

## Getting Started

### Prerequisites

- Node.js >= 16.x
- Rust (for smart contract development)
- Stellar account (testnet or mainnet)

### Development

```bash
# Clone the repository
git clone https://github.com/Web3Novalabs/Nevo.git
cd Nevo

# Smart contract development
cd contract
cargo build --target wasm32-unknown-unknown --release

# Frontend development
cd frontend
npm install
npm run dev
```

## Contributing

We welcome contributions! Here's how to get involved:

1. Check out our [issues](https://github.com/Web3Novalabs/Nevo/issues)
2. Fork the repository
3. Create your feature branch
4. Make your changes
5. Submit a pull request

## Community

Join our community (coming soon)

## License

MIT License - see [LICENSE](LICENSE) for details

---

Built with ❤️ on Stellar
