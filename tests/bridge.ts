import assert from 'assert';
import * as fs from 'fs'
import { createHash } from "crypto";
import { serialize } from '@dao-xyz/borsh';
import * as ed from "@noble/ed25519";
import { Worker } from 'near-workspaces';
import { Account, connect, keyStores, Near } from "near-api-js";
import { BridgeHelper, XpnftHelper } from '../src/helper';
import BN from 'bn.js';
import { WhitelistData } from '../src/encode';

describe("bridge", async () => {
    let worker: Worker;
    let nearConnection: Near;

    let collectionOwnerAcc: Account;
    let nftOwnerAcc: Account;

    let xpnftContract: XpnftHelper;
    let bridgeContract: BridgeHelper;

    let pk: Uint8Array;
    let sk: Uint8Array;

    before(async () => {
        // Init the worker and start a Sandbox server
        worker = await Worker.init();

        const root = worker.rootAccount;

        const xpnft = await root.createSubAccount('xpnft');
        const collectionOwner = await root.createSubAccount("owner");
        const nftOwner = await root.createSubAccount("nft-owner");
        const xpbridge = await root.createSubAccount("xpbridge")

        const myKeyStore = new keyStores.InMemoryKeyStore();
        await myKeyStore.setKey("local", xpnft.accountId, await xpnft.getKey())
        await myKeyStore.setKey("local", collectionOwner.accountId, await collectionOwner.getKey())
        await myKeyStore.setKey("local", nftOwner.accountId, await nftOwner.getKey())
        await myKeyStore.setKey("local", xpbridge.accountId, await xpbridge.getKey())

        nearConnection = await connect({
            networkId: 'local',
            nodeUrl: worker.provider.connection.url,
            keyStore: myKeyStore,
            headers: {}
        })

        const xpnftAcc = await nearConnection.account(xpnft.accountId)
        await xpnftAcc.deployContract(fs.readFileSync(
            __dirname + '/../contract/target/wasm32-unknown-unknown/release/xpnft.wasm'
        ))

        const bridgeAcc = await nearConnection.account(xpbridge.accountId)
        await bridgeAcc.deployContract(fs.readFileSync(
            __dirname + '/../contract/target/wasm32-unknown-unknown/release/xpbridge.wasm'
        ))

        collectionOwnerAcc = await nearConnection.account(collectionOwner.accountId)
        nftOwnerAcc = await nearConnection.account(nftOwner.accountId)

        xpnftContract = new XpnftHelper(collectionOwnerAcc, xpnft.accountId)
        bridgeContract = new BridgeHelper(bridgeAcc, xpbridge.accountId)

        sk = ed.utils.randomPrivateKey()
        pk = await ed.getPublicKey(sk)
    })

    it("initialize xpnft", async () => {
        await xpnftContract.initialize(collectionOwnerAcc.accountId, {
            spec: "nft-1.0.0",
            name: "xpnft",
            symbol: "XPNFT",
            icon: null,
            base_uri: null,
            reference: null,
            reference_hash: null
        })
    })

    it("initialize bridge", async () => {
        await bridgeContract.initialize(pk)

        const storedPk = await bridgeContract.getGroupKey()
        assert.ok(Buffer.from(pk).equals(Buffer.from(storedPk)))
    })

    it("whitelist nft", async () => {
        const actionId = new BN(0);
        const data = new WhitelistData(actionId, bridgeContract.getContractId(), xpnftContract.getContractId())
        const message = serialize(data)
        const msgHash = createHash("SHA256").update(message).digest();
        const signature = await ed.sign(msgHash, sk)
        await bridgeContract.whitelist(xpnftContract.getContractId(), actionId, signature)

        const flag = await bridgeContract.isWhitelist(xpnftContract.getContractId())
        assert.ok(flag)
    })

    after(async () => {
        await worker.tearDown().catch((error) => {
            console.log("Failed to stop the sandbox:", error)
        })
    })
})