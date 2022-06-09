import { KeyringPair } from "@polkadot/keyring/types";
import { logger } from "../logger";
import { HackathonApi } from "../chain-ops/api";
import { HackathonKeyring } from "../chain-ops/keyring";

export async function finish(
  api: HackathonApi,
  keyring: HackathonKeyring,
  accounts: KeyringPair[],
  balances: [KeyringPair, string, bigint][],
  assets: string[]
): Promise<void> {
  logger.info("Burning balances")
  await Promise.all(
    balances.map(async ([acc, asset, amount]) => {
      await api.burn(keyring, acc.address, asset, amount);
    })
  );

  logger.info("Removing assets")
  await Promise.all(
    assets.map(async (asset) => {
      await api.removeAsset(keyring, asset);
    })
  );
}
