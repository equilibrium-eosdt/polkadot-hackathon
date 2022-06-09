import { AddressOrPair } from "@polkadot/api/types";
import { logger } from "../logger";
import { HackathonApi as Api } from "./api";
import { HackathonKeyring as Keyring } from "./keyring";

export async function getBalances(
  api: Api,
  who: string
): Promise<{ [currency: string]: bigint }> {
  let balances: { [currency: string]: bigint } = {};
  (await api.query.balances.accounts.entries(who)).forEach(
    ([k, v]) => (balances[(k.toHuman() as string[])[1]] = v.toBigInt())
  );
  return balances;
}

export async function transfer(
  api: Api,
  from: AddressOrPair,
  to: string,
  currency: string,
  amount: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.balances.transfer(to, currency, amount),
    from
  );
  logger.trace(result);
}

export async function mint(
  api: Api,
  keyring: Keyring,
  who: string,
  currency: string,
  amount: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.sudo.sudo(api.tx.balances.mint(who, currency, amount)),
    keyring.sudo()!
  );
  logger.trace(result);
}

export async function burn(
  api: Api,
  keyring: Keyring,
  who: string,
  currency: string,
  amount: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.sudo.sudo(api.tx.balances.mint(who, currency, amount)),
    keyring.sudo()!
  );
  logger.trace(result);
}
