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
cargo build --all --target wasm32-unknown-unknown --release
```

## Compiling both contracts to WASM

```bash
./build.sh
```

## Creating accounts

NB! An account can deploy only one smart contract on NEAR.

Create as many accounts as you want to deploy contracts.

The account's accountId is the address of the contract.

### To create an account on the testnet:

```bash
yarn create_testnet <creatorAccountId> <newAccountId> <amount>
```

### To create an account on the mainnet:

```bash
yarn create_mainnet <creatorAccountId> <newAccountId> <amount>
```

Example:

```
yarn create_testnet dimabrook-testnet.testnet xpnft.testnet 20
yarn create_testnet dimabrook-testnet.testnet xpbridge.testnet 20
```

Funding testnet accounts:

Get 20 testnet Near every hour.

```
https://near-faucet.io/
```

### Checking that the accounts were created

Example:

```
near keys xpnft.testnet
```

### Getting the account keypairs

```
cd ~/.near_credentials/testnet/
nano <accountId>.json
```


### Deploying the smart contract

Login with `near-cli`

```
near login
```

Deploying xpnft and bridge contract

```
near deploy --wasmFile target/wasm32-unknown-unknown/release/xpnft.wasm --accountId YOUR_ACCOUNT_HERE
near deploy --wasmFile target/wasm32-unknown-unknown/release/xpbridge.wasm --accountId YOUR_ACCOUNT_HERE
```

Switching chains (The default network for near-cli is testnet):

```
export NEAR_ENV=mainnet
```

### Initialize

```
yarn setup
```

### Whitelist NFT Contract

```
yarn whitelist <contractId> <actionId>
```