import { field, fixedArray, option, vec } from "@dao-xyz/borsh";
import BN from "bn.js"

export class WhitelistData {
    @field({ type: "u128" })
    actionId: BN;
    @field({ type: "string" })
    mintWith: string;

    constructor(actionId: BN, mintWith: string) {
        this.actionId = actionId
        this.mintWith = mintWith
    }
}

// export class PauseData {
//     @field({ type: "u64" })
//     actionId: BN;
// }

// export class UnpauseData {
//     @field({ type: "u64" })
//     actionId: BN;
// }

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

// export class TransferNftData {
//     @field({ type: "u64" })
//     actionId: BN;
//     @field({ type: "u64" })
//     chainNonce: BN;
//     @field({ type: "String" })
//     name: string;
//     @field({ type: "String" })
//     symbol: string;
//     @field({ type: "String" })
//     uri: string;
//     @field({ type: fixedArray("u8", 32) })
//     owner: number[];
//     @field({ type: option(fixedArray("u8", 32)) })
//     collection: number[] | undefined;
//     @field({ type: option("u16") })
//     sellerFeeBasisPoints: number | undefined;
//     @field({ type: option(vec(CreatorData)) })
//     creators: CreatorData[] | undefined;
// }

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
