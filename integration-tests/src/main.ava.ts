import { Worker, NearAccount } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async (t) => {
  // Init the worker and start a Sandbox server
  const worker = await Worker.init();

  // Deploy contract
  const root = worker.rootAccount;
  const nft = await root.createSubAccount('xpnft');
  // Get wasm file path from package.json test script in folder above
  await nft.deploy(
    process.argv[2],
  );

  const bridge = await root.createSubAccount('xpbridge');
  // Get wasm file path from package.json test script in folder above
  await bridge.deploy(
    process.argv[3],
  );

  // Save state for test runs, it is unique for each test
  t.context.worker = worker;
  t.context.accounts = { root, nft, bridge };

  await root.call(
    nft,
    'new',
    {
      owner_id: root,
      metadata: {
        spec: "nft-1.0.0",
        name: "Xp NFT",
        symbol: "XPNFT",
      }
    }
  )

  // await root.call(
  //   bridge,
  //   'new',
  //   {
  //     owner_id: root,
  //     metadata: {
  //       spec: "nft-1.0.0",
  //       name: "Xp NFT",
  //       symbol: "XPNFT",
  //     }
  //   }
  // )
});

test.afterEach(async (t) => {
  // Stop Sandbox server
  await t.context.worker.tearDown().catch((error) => {
    console.log('Failed to stop the Sandbox:', error);
  });
});

test('mint & burn nft', async (t) => {
  const { root, nft } = t.context.accounts;
  await root.call(
    nft,
    'nft_mint',
    {
      token_id: '0',
      token_owner_id: root,
      token_metadata: {
        title: "Olympus Mons",
        description: "The tallest mountain in the charted solar system"
      }
    },
    { attachedDeposit: '7000000000000000000000' }  // 7 * 10 ^ 21 yoctoNEAR to cover storage
  );
  const token: any = await nft.view('nft_token', { token_id: '0' })
  t.is(token.owner_id, root.accountId)

  await root.call(
    nft,
    'nft_burn',
    {
      token_id: '0',
      from: root,
    },
  );
});