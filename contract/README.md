# Nevo Smart Contract

Soroban smart contract for decentralized donation pools on Stellar blockchain.

## Features

### Core Functionality
- **Campaign Management**: Create and manage fundraising campaigns with goals and deadlines
- **Pool Management**: Create donation pools with metadata, targets, and time limits
- **Contributions**: Accept donations in XLM, USDC, or custom Stellar assets
- **Refunds**: Contributors can request refunds under certain conditions
- **Cause Verification**: Admin can verify legitimate causes
- **Emergency Withdrawals**: Safety mechanism for emergency fund recovery

### Security & Control
- **Pause/Unpause**: Global contract pause for emergency situations
- **Module-Level Pause**: Independent pause controls for pools and campaigns
- **Admin Controls**: Protected admin functions with authentication
- **Fee Management**: Configurable platform fees
- **Emergency Contact**: Designated emergency contact for critical situations

### State Management
- Pool states: Active, Paused, Completed, Closed
- Campaign tracking with donor counts and contribution history
- Global statistics (total raised, active campaigns, etc.)

## Project Structure

```text
contract/
├── contract/
│   ├── src/
│   │   ├── base/
│   │   │   ├── errors.rs      # Error definitions
│   │   │   ├── events.rs      # Event emissions
│   │   │   ├── types.rs       # Data structures
│   │   │   └── mod.rs
│   │   ├── interfaces/
│   │   │   ├── crowdfunding.rs # Contract interface
│   │   │   └── mod.rs
│   │   ├── crowdfunding.rs    # Main contract implementation
│   │   ├── lib.rs
│   │   └── test.rs
│   ├── test/
│   │   ├── crowdfunding_test.rs  # Core functionality tests
│   │   ├── module_pause_test.rs  # Pause/unpause tests
│   │   ├── close_pool_test.rs    # Pool closure tests
│   │   ├── create_pool.rs        # Pool creation tests
│   │   ├── verify_cause.rs       # Cause verification tests
│   │   └── mod.rs
│   ├── Cargo.toml
│   └── Makefile
├── Cargo.toml
└── README.md
```

## Building

### Prerequisites
- Rust (latest stable)
- Soroban CLI
- wasm32-unknown-unknown target

### Build Commands

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Or use the Makefile
cd contract
make build
```

## Testing

The contract includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test crowdfunding_test
cargo test --test module_pause_test
cargo test --test close_pool_test
```

### Test Coverage
- Campaign creation and management
- Pool creation and state transitions
- Contribution and refund flows
- Pause/unpause functionality (global and module-level)
- Admin authorization
- Error handling and edge cases
- Event emission verification

## Deployment

```bash
# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/crowdfunding.wasm \
  --source <YOUR_SECRET_KEY> \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Initialize the contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_SECRET_KEY> \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --token <TOKEN_ADDRESS> \
  --fee 0
```

## Contract Interface

### Initialization
- `initialize(admin, token, fee)` - Initialize contract with admin and default token

### Campaign Functions
- `create_campaign(id, title, creator, goal, deadline, token)` - Create new campaign
- `get_campaign(id)` - Get campaign details
- `get_all_campaigns()` - List all campaign IDs
- `donate(campaign_id, donor, amount)` - Donate to campaign

### Pool Functions
- `save_pool(name, metadata, creator, target, deadline, duration, whitelist)` - Create pool
- `get_pool(pool_id)` - Get pool configuration
- `update_pool_state(pool_id, state)` - Update pool state
- `contribute(pool_id, contributor, token, amount, anonymous)` - Contribute to pool
- `refund(pool_id, contributor)` - Request refund
- `close_pool(pool_id, caller)` - Close a pool

### Admin Functions
- `pause()` / `unpause()` - Global pause control
- `pause_pools()` / `unpause_pools()` - Pool module pause
- `pause_campaigns()` / `unpause_campaigns()` - Campaign module pause
- `verify_cause(cause)` - Verify a cause address
- `set_creation_fee(fee)` - Update creation fee
- `withdraw_platform_fees(recipient, amount)` - Withdraw collected fees

### Query Functions
- `is_paused()` - Check global pause state
- `is_pools_paused()` - Check pools pause state
- `is_campaigns_paused()` - Check campaigns pause state
- `get_global_raised_total()` - Get total amount raised
- `get_donor_count(campaign_id)` - Get number of donors
- `is_cause_verified(cause)` - Check if cause is verified

## Error Handling

The contract includes comprehensive error handling:
- `NotInitialized` - Contract not initialized
- `AlreadyInitialized` - Contract already initialized
- `ContractPaused` - Global pause active
- `PoolsPaused` - Pools module paused
- `CampaignsPaused` - Campaigns module paused
- `InvalidTitle` / `InvalidGoal` / `InvalidDeadline` - Validation errors
- `CampaignNotFound` / `PoolNotFound` - Resource not found
- `InsufficientBalance` - Not enough funds
- And more...

## Events

The contract emits events for:
- Campaign creation
- Pool creation
- Contributions
- Refunds
- State changes
- Admin actions

## Security Considerations

1. **Admin Authentication**: All admin functions require proper authentication
2. **Pause Mechanisms**: Multiple levels of pause for emergency situations
3. **Input Validation**: All inputs are validated before processing
4. **State Transitions**: Pool states follow strict transition rules
5. **Emergency Withdrawals**: Time-locked emergency withdrawal mechanism

## License

MIT License - see [LICENSE](../LICENSE) for details
