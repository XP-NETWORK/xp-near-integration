#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building contracts"
  exit 1
fi

echo ">> Deploying contracts"

# https://docs.near.org/tools/near-cli#near-dev-deploy
near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/xpnft.wasm
near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/xpbridge.wasm
