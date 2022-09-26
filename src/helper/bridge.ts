import BN from "bn.js";
import { Account, Contract } from "near-api-js";

interface InitParam {
    args: {
        group_key: number[]
    }
}

interface WhitelistParam {
    args: {
        data: {
            action_id: string,
            contract_id: string,
            mint_with: string,
        },
        sig_data: string
    }
}

interface PauseParam {
    args: {
        data: {
            action_id: string,
        },
        sig_data: string
    }
}

interface UnpauseParam {
    args: {
        data: {
            action_id: string,
        },
        sig_data: string
    }
}

interface BridgeContract extends Contract {
    initialize(param: InitParam): Promise<any>,
    get_group_key(): Promise<number[]>,
    is_paused(): Promise<any>,
    is_whitelist(param: { contract_id: string }): Promise<boolean>,
    validate_whitelist(param: WhitelistParam): Promise<void>,
    validate_pause(param: PauseParam): Promise<void>,
    validate_unpause(param: UnpauseParam): Promise<void>,
}

export interface NearProvider {
    account: Account;
    contractId: string;
}

export class BridgeHelper {
    private contract: BridgeContract

    constructor(signer: Account, contractId: string) {
        this.contract = new Contract(signer, contractId, {
            viewMethods: [
                "get_group_key",
                "is_paused",
                "is_whitelist",
            ],
            changeMethods: [
                "initialize",
                "validate_pause",
                "validate_unpause",
                "validate_withdraw_fees",
                "validate_update_group_key",
                "validate_whitelist",
                "validate_transfer_nft",
                "withdraw_nft",
                "freeze_nft",
                "validate_unfreeze_nft"
            ]
        }) as BridgeContract
    }

    getContractId() {
        return this.contract.contractId
    }

    async getGroupKey() {
        return await this.contract.get_group_key()
    }

    async isPaused() {
        return await this.contract.is_paused()
    }

    async initialize(groupKey: Uint8Array) {
        return await this.contract.initialize({
            args: {
                group_key: Array.from(groupKey)
            }
        })
    }

    async whitelist(nftContractId: string, actionId: BN, signature: Uint8Array) {
        return await this.contract.validate_whitelist({
            args: {
                data: {
                    action_id: actionId.toString(),
                    contract_id: this.contract.contractId,
                    mint_with: nftContractId,
                },
                sig_data: Buffer.from(signature).toString("base64")
            }
        })
    }

    async pause(actionId: BN, signature: Uint8Array) {
        return await this.contract.validate_pause({
            args: {
                data: {
                    action_id: actionId.toString(),
                },
                sig_data: Buffer.from(signature).toString("base64")
            }
        })
    }

    async unpause(actionId: BN, signature: Uint8Array) {
        return await this.contract.validate_unpause({
            args: {
                data: {
                    action_id: actionId.toString(),
                },
                sig_data: Buffer.from(signature).toString("base64")
            }
        })
    }

    async isWhitelist(contractId: string) {
        return await this.contract.is_whitelist({
            contract_id: contractId
        })
    }
}