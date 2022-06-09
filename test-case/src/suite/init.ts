import { KeyringPair } from "@polkadot/keyring/types";
import { HackathonApi } from "../chain-ops/api";
import { AssetData, AssetId } from "../chain-ops/assets";
import { HackathonKeyring } from "../chain-ops/keyring";
import { TestParams } from "../params";
import { mainAsset, oneToken, usdAsset } from ".";
import { logger } from "../logger";

function genAsset(idx: number): string {
  if (idx < 26) {
    return (idx + 10).toString(36).toUpperCase();
  } else {
    return genAsset(Math.floor(idx / 26)).concat(genAsset(idx % 26));
  }
}

export async function init(
  api: HackathonApi,
  keyring: HackathonKeyring,
  params: TestParams
): Promise<{
  accounts: KeyringPair[];
  balances: [KeyringPair, AssetId, bigint][];
  assets: string[];
}> {
  const sudo = keyring.sudo()!;
  const { nonce: sudoNonce } = await api.query.system.account(sudo.address);
  api.initNonce(sudo, sudoNonce.toNumber());

  const defaultData = { decimals: 12 } as AssetData;
  logger.info("Creating assets");
  const assets = await Promise.all(
    [...new Array(params.assets)].map(async (item, idx) => {
      const asset = genAsset(idx);
      if ((await api.getAsset(asset)) == undefined) {
        await api.createAsset(keyring, asset, defaultData);
      }
      return asset;
    })
  );

  const accounts = [...new Array(params.clients)].map((item, idx) => {
    return keyring.createKeyringPair(`#${idx}`);
  });
  logger.info("Minting to accounts");
  for (let i = 0; i < params.clients / 100; ++i) {
    const accountsChunk = accounts.slice(i * 100, (i + 1) * 100);
    await Promise.all(
      accountsChunk.map(async (acc) => {
        const [{ nonce }] = await Promise.all([
          api.query.system.account(acc.address),
          api.mint(keyring, acc.address, mainAsset, 10n * oneToken),
          api.mint(keyring, acc.address, usdAsset, 10n * oneToken),
          ...assets.map((asset) =>
            api.mint(keyring, acc.address, asset, 10n * oneToken)
          ),
        ]);
        api.initNonce(acc, nonce.toNumber());
      })
    );
  }

  const balances = await Promise.all(
    accounts.flatMap((acc) =>
      assets.map(async (asset) => {
        return [acc, asset, await api.getBalance(acc.address, asset)] as [
          KeyringPair,
          AssetId,
          bigint
        ];
      })
    )
  );

  return { accounts, balances, assets };
}
