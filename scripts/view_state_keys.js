const nearAPI = require("near-api-js");
require("dotenv").config();
async function main() {
    const { connect, keyStores, KeyPair } = nearAPI;
    const myKeyStore = new keyStores.InMemoryKeyStore();

    // private key of your account
    const PRIVATE_KEY = "";
    // creates a public / private key pair using the provided private key
    const keyPair = KeyPair.fromString(PRIVATE_KEY);
    // adds the keyPair you created to keyStore
    await myKeyStore.setKey("mainnet", "xpnft-royalty.near", keyPair);

    const config = {
        myKeyStore,
        networkId: "mainnet",
        nodeUrl: "https://archival-rpc.mainnet.near.org/",
        walletUrl: "https://wallet.mainnet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
        explorerUrl: "https://explorer.mainnet.near.org",
    };
    const near = await connect(config);
    const response = await near.connection.provider.query({
        request_type: "view_state",
        finality: "final",
        account_id: process.env.XPNFT_ACCOUNT_ID, // replace the name of the contract accordingly
        prefix_base64: "",
    });
    console.log(
        JSON.stringify({
            // TODO add calc size of data for limit burning 200TGas for one call on contract
            keys: response.values.map((it) => it.key),
        })
    );
}

main().catch((reason) => {
    console.error(reason);
});
