# NEAR Integration with Xp.network Bridge

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