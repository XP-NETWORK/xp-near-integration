# Deploying Contracts

## 0. Preparation

1. install [near-cli](https://github.com/near/near-cli) with npm
```bash
npm install -g near-cli
```
2. create a NEAR account in the wallet (if you don't have one)
3. Login to the account
```bash
near login
```

Use the account from the wallet to create the accounts for the contracts. One account on NEAR can deploy only one contract!!!

## 1. Create an account for the bridge contract

The bridge contract:
```bash
node ./scripts/index.js b6494959598c96cee8e4888034c38dac50f7acbc22ee6849a9f849ab695237df xpbridge.near 3.5
```

Now we have the account's `xpbridge.near` credentials stored in the `near-cli`. Now we can refer to this account by its name in the `near-cli`.

## 2. Create an account for the XPNFT contract

XPNFT contract:
```bash
node ./scripts/index.js b6494959598c96cee8e4888034c38dac50f7acbc22ee6849a9f849ab695237df xpnft.near 2.5
```

## 3. Create an account for the PriceOracle contract

XPNFT contract:
```bash
node ./scripts/index.js b6494959598c96cee8e4888034c38dac50f7acbc22ee6849a9f849ab695237df price_oracle.near 2.3
```
In case of errors, try updating your rust:
```bash
rustup upgrade nightly
```

## 4. Deploying the contracts

Build the contracts:
```bash
cd contract/
./build.sh
cd ..
export NEAR_ENV=mainnet
```

1. The bridge contract:
```bash
near deploy --accountId xpbridge.near --wasmFile ./contract/target/wasm32-unknown-unknown/release/xpbridge.wasm
```

2. The XPNFT contract:
```bash
near deploy --accountId xpnft.near --wasmFile ./contract/target/wasm32-unknown-unknown/release/xpnft.wasm
```

3. The Fee Oracle contract:
```bash
near deploy --accountId price_oracle.near --wasmFile ./contract/target/wasm32-unknown-unknown/release/currency_data_oracle.wasm
```

In case of errors like this: required to have 1446612516498895700000000 yoctoNEAR more add tokens to the contract account. To count how many to add divide the missing amount by 10^24.

## 4. Initializing the contracts

```ts
const GK = [...Buffer.from("replace-with-frost-group-key", "hex")];
console.log(GK.toString().replace(' ',''));
```

1. The bridge contract example:
```bash
near call xpbridge.near initialize '{"group_key":[!!!replace!!!], "fees_oracle":"price_oracle.near"}' --accountId xpbridge.near
```

1. XPNFT example:

```bash
near call xpnft.near initialize '{"owner_id":"xpbridge.near","metadata":{"spec":"nft-1.0.0","name":"StagingXPNFT","symbol":"SXPNFT"}}' --accountId xpnft.near
```

3. Fee Oracle example:

A. Generate an FROST Edwardson GK for the fee validator(s)

---------------------------
1. git clone https://github.com/XP-NETWORK/frost-dalek-js/  sha256 branch (!!!important!!!)
2. pnpm run build-examples
3. node dist/examples/central_dealer.js
4. then follow the prompts appropriately

Then convert to an array like this:
```ts
const GK = [...Buffer.from("replace-with-frost-group-key", "hex")];
console.log(GK.toString().replace(' ',''));
```

B. Run in the terminal
```bash
near call price_oracle.near initialize '{"group_key":[!!!replace!!!],"decimals":{},"price_data":{},"chain_tx_fee_data":{},"other_fees":{}}' --accountId price_oracle.near
```

near call prodfeeoracle.near initialize '{"group_key":[131,46,140,108,139,172,103,138,155,165,160,127,192,177,121,119,250,225,34,87,254,239,48,155,32,190,44,170,15,37,187,12],"decimals":{},"price_data":{},"chain_tx_fee_data":{},"other_fees":{}}' --accountId prodfeeoracle.near


### Cleaning a contract's code

1. Polulate with the contract's private key:

```ts title="./scripts/scripts/view_state_keys.js"
// private key of your account
const PRIVATE_KEY = "<contract-private-key-here>";
```

2. Populate with the contract's account

```ts title="./scripts/scripts/view_state_keys.js"
// adds the keyPair you created to keyStore
await myKeyStore.setKey("mainnet", "<contract-account>.near", keyPair);
```

```shell
near --accountId <contract-account>.near call <contract-account>.near clean --base64 "$(node ./scripts/view_state_keys.js | base64)" --gas 3
00000000000000
```