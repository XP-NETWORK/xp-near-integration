import dotenv from 'dotenv'
import { connect, keyStores, KeyPair } from "near-api-js"
import * as fs from 'fs'

dotenv.config();

(async () => {
    const myKeyStore = new keyStores.InMemoryKeyStore();

    const bridgeKeyPair = KeyPair.fromString(process.env.BRIDGE_ACCOUNT_PRIVATE_KEY || "")
    await myKeyStore.setKey(process.env.NEAR_ENV || "testnet", process.env.BRIDGE_ACCOUNT_ID || "", bridgeKeyPair)

    const xpnftKeyPair = KeyPair.fromString(process.env.XPNFT_ACCOUNT_PRIVATE_KEY || "")
    await myKeyStore.setKey(process.env.NEAR_ENV || "testnet", process.env.XPNFT_ACCOUNT_ID || "", xpnftKeyPair)

    const connectionConfig = {
        networkId: process.env.NEAR_ENV || "testnet",
        keyStore: myKeyStore, // first create a key store 
        nodeUrl: `https://rpc.${process.env.NEAR_ENV || "testnet"}.near.org`,
        headers: {}
    };
    const nearConnection = await connect(connectionConfig);

    const bridgeAccount = await nearConnection.account(process.env.BRIDGE_ACCOUNT_ID || "")
    try {
        const { transaction } = await bridgeAccount.deployContract(fs.readFileSync(__dirname + "/../target/wasm32-unknown-unknown/release/bridge.wasm"))
        console.log(transaction)
    } catch (e) {
        console.error(e)
    }

    const account = await nearConnection.account(process.env.XPNFT_ACCOUNT_ID || "")
    try {
        const { transaction } = await account.deployContract(fs.readFileSync(__dirname + "/../target/wasm32-unknown-unknown/release/xpnft.wasm"))
        console.log(transaction)
    } catch (e) {
        console.error(e)
    }
})()