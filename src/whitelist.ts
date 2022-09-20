import dotenv from 'dotenv'
import * as ed from '@noble/ed25519';
import { connect, keyStores, KeyPair } from "near-api-js"
import * as fs from 'fs'
import { BridgeHelper } from './helper';
import { WhitelistData } from './encode';
import BN from 'bn.js';
import { serialize } from '@dao-xyz/borsh';

dotenv.config();

(async () => {
    const myKeyStore = new keyStores.InMemoryKeyStore();
    const keyPair = KeyPair.fromString(process.env.BRIDGE_ACCOUNT_PRIVATE_KEY || "")
    await myKeyStore.setKey(process.env.NEAR_ENV || "testnet", process.env.BRIDGE_ACCOUNT_ID || "", keyPair)

    const connectionConfig = {
        networkId: process.env.NEAR_ENV || "testnet",
        keyStore: myKeyStore, // first create a key store 
        nodeUrl: `https://rpc.${process.env.NEAR_ENV || "testnet"}.near.org`,
        headers: {}
    };
    const nearConnection = await connect(connectionConfig);

    const account = await nearConnection.account(process.env.BRIDGE_ACCOUNT_ID || "")
    const helper = new BridgeHelper({
        account,
        contractId: process.env.BRIDGE_ACCOUNT_ID || ""
    })
    try {
        const ed25519PrivateKey = Buffer.from(process.env.ED25519_PRIVATE_KEY || "", "hex")
        
        const mintWith = process.argv[2]
        const actionId = process.argv[3]
        const data = new WhitelistData(new BN(actionId), mintWith)
        const message = serialize(data)
        const signature = await ed.sign(message, ed25519PrivateKey)
        await helper.whitelist(mintWith, parseInt(actionId), signature)
    } catch (e) {
        console.error(e)
    }
})()