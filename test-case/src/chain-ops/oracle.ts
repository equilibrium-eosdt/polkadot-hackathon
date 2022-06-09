import { logger } from "../logger";
import { HackathonApi as Api } from "./api";
import { HackathonKeyring as Keyring } from "./keyring";

export async function getPrice(
  api: Api,
  currency: string
): Promise<bigint | undefined> {
  const maybeValue = await api.query.oracle.prices(currency);
  if (maybeValue.isSome) {
    return maybeValue.unwrap().toBigInt();
  } else {
    return undefined;
  }
}

export async function forceSetPrice(
  api: Api,
  keyring: Keyring,
  currency: string,
  price: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.oracle.forceSetPrice(currency, price),
    keyring.sudo()!
  );
  logger.trace(result);
}
