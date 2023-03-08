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
    FreezeNftData,
    PauseData,
    TokenMetadataData,
    TransferNftData,
    UnpauseData,
    WhitelistData,
    WithdrawNftData,
} from "../src/encode";

describe("bridge", async () => {
    let worker: Worker;
    let nearConnection: Near;

    let xpnftAcc: Account;
    let bridgeAcc: Account;
    let collectionOwnerAcc: Account;
    let collectionOwnerAcc2: Account;
    let nftOwnerAcc: Account;
    let nftOwnerAcc2: Account;

    let pk: Uint8Array;
    let sk: Uint8Array;

    let fee_pk: Uint8Array;
    let fee_sk: Uint8Array;

    before(async () => {
        // Init the worker and start a Sandbox server
        worker = await Worker.init();

        const root = worker.rootAccount;

        const xpnft = await root.createSubAccount("xpnft");
        const collectionOwner = await root.createSubAccount("owner");
        const collectionOwner2 = await root.createSubAccount("owner2");
        const nftOwner = await root.createSubAccount("nft-owner");
        const nftOwner2 = await root.createSubAccount("nft-owner2");
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
            collectionOwner2.accountId,
            await collectionOwner2.getKey()
        );
        await myKeyStore.setKey(
            "local",
            nftOwner.accountId,
            await nftOwner.getKey()
        );
        await myKeyStore.setKey(
            "local",
            nftOwner2.accountId,
            await nftOwner2.getKey()
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
        collectionOwnerAcc2 = await nearConnection.account(
            collectionOwner2.accountId
        );
        await collectionOwnerAcc2.deployContract(
            fs.readFileSync(
                __dirname +
                    "/../contract/target/wasm32-unknown-unknown/release/xpnft.wasm"
            )
        );
        nftOwnerAcc = await nearConnection.account(nftOwner.accountId);
        nftOwnerAcc2 = await nearConnection.account(nftOwner2.accountId);

        sk = ed.utils.randomPrivateKey();
        pk = await ed.getPublicKey(sk);

        fee_sk = ed.utils.randomPrivateKey();
        fee_pk = await ed.getPublicKey(fee_sk);
    });

    it("initialize xpnft", async () => {
        const xpnftHelper = new XpnftHelper(
            xpnftAcc.accountId,
            collectionOwnerAcc
        );
        await xpnftHelper.initialize(bridgeAcc.accountId, {
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

        await bridgeHelper.initialize(pk, fee_pk);

        const storedPk = await bridgeHelper.getGroupKey();
        assert.ok(Buffer.from(pk).equals(Buffer.from(storedPk)));
    });

    it("whitelist nft", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, bridgeAcc);

        const actionId = new BN(0);
        const data = new WhitelistData({
            actionId,
            tokenContract: xpnftAcc.accountId,
        });
        const message = serialize(data);
        const context = Buffer.from("WhitelistNft");
        const msgHash = createHash("SHA256")
            .update(context)
            .update(message)
            .digest();
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
        const context = Buffer.from("SetPause");
        const msgHash = createHash("SHA256")
            .update(context)
            .update(message)
            .digest();
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
        const context = Buffer.from("SetUnpause");
        const msgHash = createHash("SHA256")
            .update(context)
            .update(message)
            .digest();
        const signature = await ed.sign(msgHash, sk);
        await bridgeHelper.unpause(data, signature);

        const flag = await bridgeHelper.isPaused();
        assert.ok(!flag);
    });

    it("transfer wrapped_nft:0", async () => {
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
        const context = Buffer.from("ValidateTransferNft");
        const msgHash = createHash("SHA256")
            .update(context)
            .update(message)
            .digest();

        const signature = await ed.sign(msgHash, sk);
        const res = await bridgeHelper.transferNft(data, signature);
        console.log(res);
    });

    it("withdraw wrapped_nft:0", async () => {
        const bridgeHelper = new BridgeHelper(bridgeAcc.accountId, nftOwnerAcc);
        const chainNonce = 0;
        const to = "example_address";
        const amt = new BN(1_000_000_000_000);

        const data = new WithdrawNftData({
            tokenContract: xpnftAcc.accountId,
            tokenId: "0",
            chainNonce: 0,
            to: to,
            amt: amt,
        });
        const message = serialize(data);
        const msgHash = createHash("SHA256").update(message).digest();

        const signature = await ed.sign(msgHash, sk);

        await bridgeHelper.withdrawNft(
            xpnftAcc.accountId,
            "0",
            chainNonce,
            to,
            amt,
            signature
        );
    });

    it("initialize common_nft", async () => {
        const nftHelper = new XpnftHelper(
            collectionOwnerAcc2.accountId,
            collectionOwnerAcc2
        );
        await nftHelper.initialize(collectionOwnerAcc2.accountId, {
            spec: "nft-1.0.0",
            name: "rarible",
            symbol: "RAR",
            icon: null,
            base_uri: null,
            reference: null,
            reference_hash: null,
        });
    });

    it("mint common_nft:0", async () => {
        let nftHelper = new XpnftHelper(
            collectionOwnerAcc2.accountId,
            collectionOwnerAcc2
        );
        await nftHelper.mint("0", nftOwnerAcc2.accountId, {
            title: "Lockheed Martin",
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

        nftHelper = new XpnftHelper(
            collectionOwnerAcc2.accountId,
            nftOwnerAcc2
        );
        await nftHelper.approve("0", bridgeAcc.accountId);
    });

    it("freeze nft:0", async () => {
        const chainNonce = 0;
        const to = "example_address";
        const amt = new BN(1_000_000_000_000);

        const bridgeHelper = new BridgeHelper(
            bridgeAcc.accountId,
            nftOwnerAcc2
        );

        const data = new FreezeNftData({
            tokenContract: xpnftAcc.accountId,
            tokenId: "0",
            chainNonce: chainNonce,
            to: to,
            mintWith: "foreign_nft_contract",
            amt: amt,
        });
        const message = serialize(data);
        const msgHash = createHash("SHA256").update(message).digest();

        const signature = await ed.sign(msgHash, sk);

        await bridgeHelper.freezeNft(
            collectionOwnerAcc2.accountId,
            "0",
            chainNonce,
            to,
            "foreign_nft_contract",
            amt,
            signature
        );
    });

    after(async () => {
        await worker.tearDown().catch((error) => {
            console.log("Failed to stop the sandbox:", error);
        });
    });
});
