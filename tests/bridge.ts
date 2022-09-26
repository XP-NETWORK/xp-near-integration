import * as fs from 'fs'
import * as ed from "@noble/ed25519";
import { Worker } from 'near-workspaces';
import { Account, connect, keyStores, Near } from "near-api-js";
import { BridgeHelper, XpnftHelper } from '../src/helper';

describe("bridge", async () => {
    let worker: Worker;
    let nearConnection: Near;

    let collectionOwnerAcc: Account;
    let nftOwnerAcc: Account;

    let xpnftContract: XpnftHelper;
    let bridgeContract: BridgeHelper;

    let pk: Uint8Array;

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
        bridgeContract = new BridgeHelper(collectionOwnerAcc, xpbridge.accountId)

        const sk = ed.utils.randomPrivateKey()
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
    })

    after(async () => {
        await worker.tearDown().catch((error) => {
            console.log("Failed to stop the sandbox:", error)
        })
    })
})