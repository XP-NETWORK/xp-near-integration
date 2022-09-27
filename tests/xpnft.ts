import * as fs from "fs";
import { Worker } from "near-workspaces";
import { Account, connect, keyStores, Near } from "near-api-js";
import { XpnftHelper } from "../src/helper";

describe("xpnft", async () => {
    let worker: Worker;
    let nearConnection: Near;

    let xpnftAcc: Account;
    let collectionOwnerAcc: Account;
    let nftOwnerAcc: Account;
    let bobAcc: Account;

    before(async () => {
        // Init the worker and start a Sandbox server
        worker = await Worker.init();

        const root = worker.rootAccount;
        const xpnft = await root.createSubAccount("xpnft");
        const collectionOwner = await root.createSubAccount("owner");
        const nftOwner = await root.createSubAccount("nft-owner");
        const bob = await root.createSubAccount("bob")

        const myKeyStore = new keyStores.InMemoryKeyStore();
        await myKeyStore.setKey("local", xpnft.accountId, await xpnft.getKey());
        await myKeyStore.setKey(
            "local",
            collectionOwner.accountId,
            await collectionOwner.getKey()
        );
        await myKeyStore.setKey(
            "local",
            nftOwner.accountId,
            await nftOwner.getKey()
        );
        await myKeyStore.setKey(
            "local",
            bob.accountId,
            await bob.getKey()
        )

        nearConnection = await connect({
            networkId: "local",
            nodeUrl: worker.provider.connection.url,
            keyStore: myKeyStore,
            headers: {},
        });

        xpnftAcc = await nearConnection.account(xpnft.accountId);
        await xpnftAcc.deployContract(
            fs.readFileSync(
                __dirname +
                    "/../contract/target/wasm32-unknown-unknown/release/xpnft.wasm"
            )
        );

        collectionOwnerAcc = await nearConnection.account(
            collectionOwner.accountId
        );
        nftOwnerAcc = await nearConnection.account(nftOwner.accountId);
        bobAcc = await nearConnection.account(bob.accountId)
    });

    it("initialize collection", async () => {
        const xpnftHelper = new XpnftHelper(xpnftAcc.accountId, collectionOwnerAcc);
        await xpnftHelper.initialize(collectionOwnerAcc.accountId, {
            spec: "nft-1.0.0",
            name: "xpnft",
            symbol: "XPNFT",
            icon: null,
            base_uri: null,
            reference: null,
            reference_hash: null,
        });
    });

    it("mint NFT:0", async () => {
        const xpnftHelper = new XpnftHelper(xpnftAcc.accountId, collectionOwnerAcc);

        await xpnftHelper.mint("0", nftOwnerAcc.accountId, {
            title: "Olympus Mons",
            description: "The tallest mountain in the charted solar system",
            media: null,
            media_hash: null,
            copies: 10000,
            issued_at: null,
            expires_at: null,
            starts_at: null,
            updated_at: null,
            extra: null,
            reference: null,
            reference_hash: null,
        });
    });

    it("transfer NFT:0", async () => {
        const xpnftHelper = new XpnftHelper(xpnftAcc.accountId, nftOwnerAcc);
        await xpnftHelper.transfer(bobAcc.accountId, "0")
    })

    it("burn NFT:0", async () => {
        const xpnftHelper = new XpnftHelper(xpnftAcc.accountId, collectionOwnerAcc);
        await xpnftHelper.burn("0", bobAcc.accountId);
    });

    after(async () => {
        await worker.tearDown().catch((error) => {
            console.log("Failed to stop the sandbox:", error);
        });
    });
});
