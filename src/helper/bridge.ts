import { Account, Contract } from "near-api-js";

interface InitParam {
    args: {
        group_key: number[]
    }
}

interface WhitelistParam {
    args: {
        data: {
            action_id: number,
            mint_with: string,
        },
        sig_data: number[]
    }
}

interface BridgeContract extends Contract {
    initialize(param: InitParam): Promise<any>,
    get_group_key(): Promise<any>,
    is_paused(): Promise<any>,
    validate_whitelist(param: WhitelistParam): Promise<any>,
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
                "is_paused"
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

    async whitelist(nftContractId: string, actionId: number, signature: Uint8Array) {
        return await this.contract.validate_whitelist({
            args: {
                data: {
                    action_id: actionId,
                    mint_with: nftContractId,
                },
                sig_data: Array.from(signature)
            }
        })
    }
}