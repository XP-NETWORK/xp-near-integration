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
    @field({ type: option("string") })
    title: string | undefined;
    @field({ type: option("string") })
    description: string | undefined;
    @field({ type: option("string") })
    media: string | undefined;
    @field({ type: option(vec("u8")) })
    mediaHash: Uint8Array | undefined;
    @field({ type: option("u64") })
    copies: number | undefined;
    @field({ type: option("string") })
    issuedAt: string | undefined;
    @field({ type: option("string") })
    expiresAt: string | undefined;
    @field({ type: option("string") })
    startsAt: string | undefined;
    @field({ type: option("string") })
    updatedAt: string | undefined;
    @field({ type: option("string") })
    extra: string | undefined;
    @field({ type: option("string") })
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
    @field({ type: "string" })
    mintWith: string;
    @field({ type: "string" })
    tokenId: string;
    @field({ type: "string" })
    tokenOwnerId: string;
    @field({ type: TokenMetadataData })
    tokenMetadata: TokenMetadataData;

    constructor(data: TransferNftData) {
        Object.assign(this, data);
    }
}

// export class UnfreezeNftData {
//     @field({ type: "u64" })
//     actionId: BN;
//     @field({ type: fixedArray("u8", 32) })
//     receiver: number[];
//     @field({ type: fixedArray("u8", 32) })
//     mint: number[];
// }

// export class WithdrawFeesData {
//     @field({ type: "u64" })
//     actionId: BN;
// }
