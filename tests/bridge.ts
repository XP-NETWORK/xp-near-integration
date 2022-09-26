import assert from "assert";
import * as fs from "fs";
import { createHash } from "crypto";
import { serialize } from "@dao-xyz/borsh";
import * as ed from "@noble/ed25519";
import { Worker } from "near-workspaces";
import { Account, connect, keyStores, Near } from "near-api-js";
import { BridgeHelper, XpnftHelper } from "../src/helper";
import BN from "bn.js";
import {
    PauseData,
    TokenMetadataData,
    TransferNftData,
    UnpauseData,
    WhitelistData,
} from "../src/encode";

describe("bridge", async () => {
    let worker: Worker;
    let nearConnection: Near;

    let xpnftAcc: Account;
    let bridgeAcc: Account;
    let collectionOwnerAcc: Account;
    let nftOwnerAcc: Account;

    let pk: Uint8Array;
    let sk: Uint8Array;

    before(async () => {
        // Init the worker and start a Sandbox server
        worker = await Worker.init();

        const root = worker.rootAccount;

        const xpnft = await root.createSubAccount("xpnft");
        const collectionOwner = await root.createSubAccount("owner");
        const nftOwner = await root.createSubAccount("nft-owner");
        const xpbridge = await root.createSubAccount("xpbridge");

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
            xpbridge.accountId,
            await xpbridge.getKey()
        );

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

        bridgeAcc = await nearConnection.account(xpbridge.accountId);
        await bridgeAcc.deployContract(
            fs.readFileSync(
                __dirname +
                    "/../contract/target/wasm32-unknown-unknown/release/xpbridge.wasm"
            )
        );

        collectionOwnerAcc = await nearConnection.account(
            collectionOwner.accountId
        );
        nftOwnerAcc = await nearConnection.account(nftOwner.accountId);

        sk = ed.utils.randomPrivateKey();
        pk = await ed.getPublicKey(sk);
    });

    it("initialize xpnft", async () => {
        const xpnftHelper = new XpnftHelper(
            xpnftAcc.accountId,
            collectionOwnerAcc
        );
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        await xpnftHelper.initialize(bridgeHelper.getContractId(), {
            spec: "nft-1.0.0",
            name: "xpnft",
            symbol: "XPNFT",
            icon: null,
            base_uri: null,
            reference: null,
            reference_hash: null,
        });
    });

    it("initialize bridge", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        await bridgeHelper.initialize(pk);

        const storedPk = await bridgeHelper.getGroupKey();
        assert.ok(Buffer.from(pk).equals(Buffer.from(storedPk)));
    });

    it("whitelist nft", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        const actionId = new BN(0);
        const data = new WhitelistData({
            actionId,
            mintWith: xpnftAcc.accountId,
        });
        const message = serialize(data);
        const msgHash = createHash("SHA256").update(message).digest();
        const signature = await ed.sign(msgHash, sk);
        await bridgeHelper.whitelist(data, signature);

        const flag = await bridgeHelper.isWhitelist(xpnftAcc.accountId);
        assert.ok(flag);
    });

    it("pause bridge", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        const actionId = new BN(1);
        const data = new PauseData({ actionId });
        const message = serialize(data);
        const msgHash = createHash("SHA256").update(message).digest();
        const signature = await ed.sign(msgHash, sk);
        await bridgeHelper.pause(data, signature);

        const flag = await bridgeHelper.isPaused();
        assert.ok(flag);
    });

    it("unpause bridge", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        const actionId = new BN(2);
        const data = new UnpauseData({ actionId });
        const message = serialize(data);
        const msgHash = createHash("SHA256").update(message).digest();
        const signature = await ed.sign(msgHash, sk);
        await bridgeHelper.unpause(data, signature);

        const flag = await bridgeHelper.isPaused();
        assert.ok(!flag);
    });

    it("transfer nft:0", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        const actionId = new BN(3);
        const data = new TransferNftData({
            actionId,
            mintWith: xpnftAcc.accountId,
            tokenId: "0",
            tokenOwnerId: nftOwnerAcc.accountId,
            tokenMetadata: new TokenMetadataData({
                title: "Olympus Mons",
                description: "The tallest mountain in the charted solar system",
                media: null,
                mediaHash: null,
                copies: 10000,
                issuedAt: null,
                expiresAt: null,
                startsAt: null,
                updatedAt: null,
                extra: null,
                reference: null,
                referenceHash: null,
            }),
        });
        const message = serialize(data);
        const msgHash = createHash("SHA256").update(message).digest();

        const signature = await ed.sign(msgHash, sk);
        const res = await bridgeHelper.transferNft(data, signature);
        console.log(res);
    });

    it("withdraw nft:0", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, nftOwnerAcc);
        const chainNonce = 0;
        const to = "example_address";
        const amt = new BN(1_000_000_000_000);
        await bridgeHelper.withdrawNft(
            xpnftAcc.accountId,
            "0",
            chainNonce,
            to,
            amt
        );
    });

    after(async () => {
        await worker.tearDown().catch((error) => {
            console.log("Failed to stop the sandbox:", error);
        });
    });
});
