import { Account, Contract } from "near-api-js";

interface ContractMetadata {
    spec: string,
    name: string,
    symbol: string,
    icon: string | null,
    base_uri: string | null,
    reference: string | null,
    reference_hash: Uint8Array | null,
}

interface InitParam {
    args: {
        owner_id: string,
        metadata: ContractMetadata
    }
}

interface TokenMetadata {
    title: string | null,
    description: string | null,
    media: string | null,
    media_hash: Uint8Array | null,
    copies: number | null,
    issued_at: string | null,
    expires_at: string | null,
    starts_at: string | null,
    updated_at: string | null,
    extra: string | null,
    reference: string | null,
    reference_hash: Uint8Array | null,
}

interface MintParam {
    args: {
        token_id: string,
        token_owner_id: string,
        token_metadata: TokenMetadata,
    },
    amount: string,
}

interface Token {
    token_id: string,
    owner_id: string,
    metadata: TokenMetadata,
    approved_account_ids: any
}

interface BurnParam {
    args: {
        token_id: string,
        from: string,
    }
}

interface XpnftContract extends Contract {
    initialize(param: InitParam): Promise<void>,
    nft_mint(param: MintParam): Promise<Token>,
    nft_burn(param: BurnParam): Promise<any>,
    nft_token(param: { token_id: string }): Promise<Token>,
}

export class XpnftHelper {
    private contract: XpnftContract

    constructor(signer: Account, contractId: string) {
        this.contract = new Contract(signer, contractId, {
            viewMethods: [
                "nft_token"
            ],
            changeMethods: [
                "initialize",
                "nft_mint",
                "nft_burn",
            ]
        }) as XpnftContract
    }

    async initialize(ownerId: string, metadata: ContractMetadata) {
        return await this.contract.initialize({
            args: {
                owner_id: ownerId,
                metadata: metadata
            }
        })
    }

    async mint(tokenId: string, tokenOwnerId: string, metadata: TokenMetadata) {
        return await this.contract.nft_mint({
            args: {
                token_id: tokenId,
                token_owner_id: tokenOwnerId,
                token_metadata: metadata
            },
            amount: '6150000000000000000000'
        })
    }

    async burn(tokenId: string, tokenOwnerId: string) {
        return await this.contract.nft_burn({
            args: {
                token_id: tokenId,
                from: tokenOwnerId,
            }
        })
    }

    async getTokenData(tokenId: string) {
        return await this.contract.nft_token({
            token_id: tokenId
        })
    }
}