import { AddressOrPair } from "@polkadot/api/types";
import { logger } from "../logger";
import { HackathonApi as Api } from "./api";
import { HackathonKeyring as Keyring } from "./keyring";

export async function getDeposit(
  api: Api,
  who: string,
  currency: string
): Promise<bigint> {
  return (await api.query.distribution.deposits(who, currency)).toBigInt();
}

export async function deposit(
  api: Api,
  who: AddressOrPair,
  currency: string,
  amount: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.distribution.deposit(currency, amount),
    who
  );
  logger.trace(result);
}

export async function withdraw(
  api: Api,
  who: AddressOrPair,
  currency: string
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.distribution.withdraw(currency),
    who
  );
  logger.trace(result);
}

export async function issueSudo(
  api: Api,
  keyring: Keyring,
  currency: string,
  amount: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.sudo.sudo(api.tx.distribution.issue(currency, amount)),
    keyring.sudo()!
  );
  logger.trace(result);
}

export async function issue(
  api: Api,
  who: AddressOrPair,
  currency: string,
  amount: bigint
): Promise<void> {
  const result = await api.signAndWatch(
    api.tx.distribution.issue(currency, amount),
    who
  );
  logger.trace(result);
}
