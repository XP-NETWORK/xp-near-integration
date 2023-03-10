import { field, fixedArray, option, vec } from "@dao-xyz/borsh";
import BN from "bn.js";

export class WhitelistData {
    @field({ type: "u128" })
    actionId: BN;
    @field({ type: "string" })
    tokenContract: string;

    constructor(data: WhitelistData) {
        Object.assign(this, data);
    }
}

export class PauseData {
    @field({ type: "u128" })
    actionId: BN;

    constructor(data: PauseData) {
        Object.assign(this, data);
    }
}

export class UnpauseData {
    @field({ type: "u128" })
    actionId: BN;

    constructor(data: UnpauseData) {
        Object.assign(this, data);
    }
}

// export class UpdateGroupkeyData {
//     @field({ type: "u64" })
//     actionId: BN;
//     @field({ type: fixedArray("u8", 32) })
//     newKey: number[];
// }

// export class CreatorData {
//     @field({ type: fixedArray("u8", 32) })
//     key: number[];
//     @field({ type: "u8" })
//     share: number;
// }

export class TokenMetadataData {
    @field({ type: option("String") })
    title: string | undefined;
    @field({ type: option("String") })
    description: string | undefined;
    @field({ type: option("String") })
    media: string | undefined;
    @field({ type: option(vec("u8")) })
    mediaHash: Uint8Array | undefined;
    @field({ type: option("u64") })
    copies: number | undefined;
    @field({ type: option("String") })
    issuedAt: string | undefined;
    @field({ type: option("String") })
    expiresAt: string | undefined;
    @field({ type: option("String") })
    startsAt: string | undefined;
    @field({ type: option("String") })
    updatedAt: string | undefined;
    @field({ type: option("String") })
    extra: string | undefined;
    @field({ type: option("String") })
    reference: string | undefined;
    @field({ type: option(vec("u8")) })
    referenceHash: Uint8Array | undefined;

    constructor(data: TokenMetadataData) {
        Object.assign(this, data);
    }
}

export class TransferNftData {
    @field({ type: "u128" })
    actionId: BN;
    @field({ type: "String" })
    mintWith: string;
    @field({ type: "String" })
    tokenId: string;
    @field({ type: "String" })
    tokenOwnerId: string;
    @field({ type: TokenMetadataData })
    tokenMetadata: TokenMetadataData;

    constructor(data: TransferNftData) {
        Object.assign(this, data);
    }
}

export class WithdrawNftData {
    @field({ type: "string" })
    tokenContract: string;
    @field({ type: "string" })
    tokenId: string;
    @field({ type: "u8" })
    chainNonce: number;
    @field({ type: "string" })
    to: string;
    @field({ type: "u128" })
    amt: BN;

    constructor(data: WithdrawNftData) {
        Object.assign(this, data);
    }
}

export class FreezeNftData {
    @field({ type: "string" })
    tokenContract: string;
    @field({ type: "string" })
    tokenId: string;
    @field({ type: "u8" })
    chainNonce: number;
    @field({ type: "string" })
    to: string;
    @field({ type: "string" })
    mintWith: string;
    @field({ type: "u128" })
    amt: BN;

    constructor(data: FreezeNftData) {
        Object.assign(this, data);
    }
}

export class TransferTxData {
    @field({ type: 'u128' })
    value: BN | undefined;
    @field({ type: 'u8' })
    fromChain: number | undefined;
    @field({ type: 'u8' })
    toChain: number | undefined;
    @field({ type: 'string' })
    to: string | undefined;

    constructor(data: TransferTxData) {
        Object.assign(this, data);
    }
}