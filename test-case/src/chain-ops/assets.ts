import { PrimitivesAssetsAssetData } from "@polkadot/types/lookup";
import { logger } from "../logger";
import { HackathonApi as Api } from "./api";
import { HackathonKeyring as Keyring } from "./keyring";

export type AssetId = string;
export type AssetData = {
  decimals: number;
};

export async function assets(
  api: Api
): Promise<{ id: AssetId; data: AssetData }[]> {
  return (await api.query.assets.assets.entries()).map(([s, d]) => {
    return {
      id: (s.toHuman() as string[])[0],
      data: {
        decimals: d.unwrap().decimals.toNumber(),
      },
    };
  });
}

export async function getAsset(
  api: Api,
  name: AssetId
): Promise<AssetData | undefined> {
  const maybeValue = await api.query.assets.assets(name);
  if (maybeValue.isSome) {
    const value = maybeValue.value as PrimitivesAssetsAssetData;
    return <AssetData>{
      decimals: value.decimals.toNumber(),
    };
  } else {
    return undefined;
  }
}

export async function createAsset(
  api: Api,
  keyring: Keyring,
  name: AssetId,
  data: AssetData
): Promise<void> {
  const result = api.signAndWatch(
    api.tx.sudo.sudo(api.tx.assets.create(name, data)),
    keyring.sudo()!
  );
  logger.trace(result);
}

export async function removeAsset(
  api: Api,
  keyring: Keyring,
  name: string
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.sudo.sudo(api.tx.assets.remove(name)),
    keyring.sudo()!
  );
  logger.trace(result);
}
