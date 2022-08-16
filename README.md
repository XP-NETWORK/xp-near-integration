# NEAR Integration with Xp.network Bridge

## Build Sandbox

```
git clone https://github.com/near/nearcore
cd nearcore
make sandbox
```

update environment variables: `export NEAR_SANDBOX_BIN_PATH="/Users/rocalex/nearcore/sandbox/debug/neard"`

### Test the code

```
cargo test -- --nocapture
```

### Compile the code

```
cargo build --target wasm32-unknown-unknown --release
```

### Deploying the smart contract

Login with `near-cli`

```
near login
```

Deploying the contract

```
near deploy --wasmFile target/wasm32-unknown-unknown/release/xp_bridge.wasm --accountId YOUR_ACCOUNT_HERE
```

Switching chains (The default network for near-cli is testnet):

```
export NEAR_ENV=mainnet
```
