import BN from "bn.js";
import { Account, Contract } from "near-api-js";
import {
    PauseData,
    TransferNftData,
    UnpauseData,
    WhitelistData,
} from "../encode";
import { Token, TokenMetadata } from "./xpnft";

interface InitParam {
    args: {
        group_key: number[];
        fee_pk: number[];
    };
}

interface WhitelistParam {
    args: {
        data: {
            action_id: string;
            token_contract: string;
        };
        sig_data: string;
    };
}

interface PauseParam {
    args: {
        data: {
            action_id: string;
        };
        sig_data: string;
    };
}

interface UnpauseParam {
    args: {
        data: {
            action_id: string;
        };
        sig_data: string;
    };
}

interface TransferNftParam {
    args: {
        data: {
            action_id: string;
            mint_with: string;
            token_id: string;
            owner_id: string;
            token_metadata: TokenMetadata;
        };
        sig_data: string;
    };
    gas: number;
    amount: string;
}

interface WithdrawNftParam {
    args: {
        token_contract: string;
        token_id: string;
        chain_nonce: number;
        to: string;
        amt: string;
        sig_data: string;
    };
    gas: number;
}

interface FreezeNftParam {
    args: {
        token_contract: string;
        token_id: string;
        chain_nonce: number;
        to: string;
        mint_with: string;
        amt: string;
        sig_data: string;
    };
    gas: number;
}

interface UnfreezeNftParam {
    args: {};
}

interface BridgeContract extends Contract {
    initialize(param: InitParam): Promise<any>;
    get_group_key(): Promise<number[]>;
    is_paused(): Promise<any>;
    is_whitelist(param: { contract_id: string }): Promise<boolean>;
    validate_whitelist(param: WhitelistParam): Promise<void>;
    validate_pause(param: PauseParam): Promise<void>;
    validate_unpause(param: UnpauseParam): Promise<void>;
    validate_transfer_nft(param: TransferNftParam): Promise<Token>;
    withdraw_nft(param: WithdrawNftParam): Promise<any>;
    freeze_nft(param: FreezeNftParam): Promise<any>;
    validate_unfreeze_nft(param: UnfreezeNftParam): Promise<any>;
}

export interface NearProvider {
    account: Account;
    contractId: string;
}

export class BridgeHelper {
    private contract: BridgeContract;

    constructor(contractId: string, signer: Account) {
        this.contract = new Contract(signer, contractId, {
            viewMethods: ["get_group_key", "is_paused", "is_whitelist"],
            changeMethods: [
                "initialize",
                "validate_pause",
                "validate_unpause",
                "validate_withdraw_fees",
                "validate_update_group_key",
                "validate_whitelist",
                "validate_blacklist",
                "validate_transfer_nft",
                "withdraw_nft",
                "freeze_nft",
                "validate_unfreeze_nft",
            ],
        }) as BridgeContract;
    }

    getContractId() {
        return this.contract.contractId;
    }

    async getGroupKey() {
        return await this.contract.get_group_key();
    }

    async isPaused() {
        return await this.contract.is_paused();
    }

    async isWhitelist(contractId: string) {
        return await this.contract.is_whitelist({
            contract_id: contractId,
        });
    }

    async initialize(groupKey: Uint8Array, fee_pk: Uint8Array) {
        return await this.contract.initialize({
            args: {
                group_key: Array.from(groupKey),
                fee_pk: Array.from(fee_pk),
            },
        });
    }

    async whitelist(data: WhitelistData, signature: Uint8Array) {
        return await this.contract.validate_whitelist({
            args: {
                data: {
                    action_id: data.actionId.toString(),
                    token_contract: data.tokenContract,
                },
                sig_data: Buffer.from(signature).toString("base64"),
            },
        });
    }

    async pause(data: PauseData, signature: Uint8Array) {
        return await this.contract.validate_pause({
            args: {
                data: {
                    action_id: data.actionId.toString(),
                },
                sig_data: Buffer.from(signature).toString("base64"),
            },
        });
    }

    async unpause(data: UnpauseData, signature: Uint8Array) {
        return await this.contract.validate_unpause({
            args: {
                data: {
                    action_id: data.actionId.toString(),
                },
                sig_data: Buffer.from(signature).toString("base64"),
            },
        });
    }

    async transferNft(data: TransferNftData, signature: Uint8Array) {
        return await this.contract.validate_transfer_nft({
            args: {
                data: {
                    action_id: data.actionId.toString(),
                    mint_with: data.mintWith,
                    token_id: data.tokenId,
                    owner_id: data.tokenOwnerId,
                    token_metadata: {
                        title: data.tokenMetadata.title,
                        description: data.tokenMetadata.description,
                        media: data.tokenMetadata.media,
                        media_hash: data.tokenMetadata.mediaHash,
                        copies: data.tokenMetadata.copies,
                        issued_at: data.tokenMetadata.issuedAt,
                        expires_at: data.tokenMetadata.expiresAt,
                        starts_at: data.tokenMetadata.startsAt,
                        updated_at: data.tokenMetadata.updatedAt,
                        extra: data.tokenMetadata.extra,
                        reference: data.tokenMetadata.reference,
                        reference_hash: data.tokenMetadata.referenceHash,
                    },
                },
                sig_data: Buffer.from(signature).toString("base64"),
            },
            gas: 300_000_000_000_000,
            amount: "7000000000000000000000",
        });
    }

    async withdrawNft(
        collection: string,
        tokenId: string,
        chainNoce: number,
        to: string,
        amt: BN,
        signature: Uint8Array
    ) {
        return await this.contract.withdraw_nft({
            args: {
                token_contract: collection,
                token_id: tokenId,
                chain_nonce: chainNoce,
                to,
                amt: amt.toString(),
                sig_data: Buffer.from(signature).toString("base64"),
            },
            gas: 300_000_000_000_000,
        });
    }

    async freezeNft(
        collection: string,
        tokenId: string,
        chainNonce: number,
        to: string,
        mintWith: string,
        amt: BN,
        signature: Uint8Array
    ) {
        return await this.contract.freeze_nft({
            args: {
                token_contract: collection,
                token_id: tokenId,
                chain_nonce: chainNonce,
                to,
                mint_with: mintWith,
                amt: amt.toString(),
                sig_data: Buffer.from(signature).toString("base64"),
            },
            gas: 300_000_000_000_000,
        });
    }
}
