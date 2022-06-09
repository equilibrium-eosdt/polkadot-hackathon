import { KeyringPair } from "@polkadot/keyring/types";
import { randomInt } from "crypto";
import { logger } from "../logger";
// import { oneToken } from ".";
import { HackathonApi } from "../chain-ops/api";
import { AssetId } from "../chain-ops/assets";
import { HackathonKeyring } from "../chain-ops/keyring";
import { convertToStable, updatePricesEvent as updatePrices } from "./prices";
import assert from "assert";

const eps = 100n;
let isGood: boolean;

async function strictWaitBlock(
  api: HackathonApi,
  deltaBlocks: number,
  maxBlockDuration: number
): Promise<void> {
  async function timeoutPromise<R>(p: Promise<R>, ms: number): Promise<R> {
    let timerId: NodeJS.Timeout | undefined;
    const timer = new Promise<R>((_, rj) => {
      timerId = setTimeout(() => {
        // rj(new Error(`timeout: ${timerId}:${ms}`));
        logger.warn(`timeout: ${timerId}:${ms}`);
        isGood = false;
      }, ms);
    });

    const value = await Promise.race([p, timer]);
    clearTimeout(timerId!);
    return value;
  }

  for (let i = 0; i < deltaBlocks; ++i) {
    await timeoutPromise(api.waitBlock(), maxBlockDuration);
  }
}

export async function iteration(
  api: HackathonApi,
  keyring: HackathonKeyring,
  accounts: KeyringPair[],
  assets: AssetId[]
): Promise<{ deltas: [KeyringPair, AssetId, bigint][], isGood: boolean }> {
  let deposits = accounts.flatMap((acc) =>
    assets.map(
      (asset) =>
        [acc, asset, BigInt(randomInt(1_000_000_000_000))] as [
          KeyringPair,
          AssetId,
          bigint
        ]
    )
  );

  const totalForEach: [KeyringPair, [AssetId, bigint][]][] = [];
  deposits.forEach(([account, asset, amount]) => {
    const foundIdx = totalForEach.findIndex(([acc]) => acc == account);
    if (foundIdx != -1) {
      totalForEach[foundIdx][1].push([asset, amount]);
    } else {
      totalForEach.push([account, [[asset, amount]]]);
    }
  });

  let totalDeposits: { [asset: AssetId]: bigint } = {};
  for (const [_, asset, amount] of deposits) {
    totalDeposits[asset] = (totalDeposits[asset] || 0n) + amount;
  }

  logger.info("Deposit to distribution");
  for (let i = 0; i < deposits.length / 2000; ++i) {
    const depositsChunk = deposits.slice(i * 2000, (i + 1) * 2000);
    await Promise.all(
      depositsChunk.map(async ([acc, asset, amount]) => {
        await api.deposit(acc, asset, amount);
      })
    );
  }

  const toIssue = assets.map(
    (asset) =>
      [asset, BigInt(randomInt(5_000_000_000_000) + 5_000_000_000_000)] as [
        AssetId,
        bigint
      ]
  );

  await updatePrices();

  isGood = true;

  const issueInStable = convertToStable(toIssue);
  logger.info(`Issue ${issueInStable} $usd`);
  await Promise.all([
    strictWaitBlock(api, 3, 2_050),
    ...toIssue.map(([issueAsset, issueAmount]) =>
      api.issueSudo(keyring, issueAsset, issueAmount)
    ),
  ]);

  const totalInStable = convertToStable(Object.entries(totalDeposits));
  const totalForEachInStable = totalForEach.map(
    ([acc, balances]) =>
      [acc, convertToStable(balances)] as [KeyringPair, bigint]
  );

  await updatePrices();

  logger.info("Withdraw from distribution");
  for (let i = 0; i < deposits.length / 2000; ++i) {
    const depositsChunk = deposits.slice(i * 2000, (i + 1) * 2000);
    await Promise.all(
      depositsChunk.map(async ([acc, asset, _]) => {
        await api.withdraw(acc, asset);
      })
    );
  }

  logger.info(`Iteration complete`);

  const deltas = toIssue.flatMap(([asset, amount]) => {
    return totalForEachInStable.map(([acc, amountInStable]) => {
      const delta = (amount * amountInStable) / totalInStable;
      return [acc, asset, delta] as [KeyringPair, AssetId, bigint];
    });
  });

  return { deltas, isGood };
}

export async function check(
  api: HackathonApi,
  accounts: KeyringPair[],
  assets: AssetId[],
  balances: [KeyringPair, AssetId, bigint][],
  deltas: [KeyringPair, AssetId, bigint][]
): Promise<{ balances: [KeyringPair, AssetId, bigint][] }> {
  logger.info("Start checking balances");
  let newBalances = [];

  for (const acc of accounts) {
    for (const asset of assets) {
      const actualBalance = await api.getBalance(acc.address, asset);
      const balance = balances.find(
        ([accD, assetD]) => accD == acc && assetD == asset
      ) ?? [acc, asset, 0n];
      const delta = deltas.find(
        ([accD, assetD]) => accD == acc && assetD == asset
      ) ?? [acc, asset, 0n];

      assert(
        balance[2] + delta[2] - actualBalance < eps &&
          balance[2] + delta[2] - actualBalance > -eps,
        new Error(
          JSON.stringify({
            acc,
            asset,
            balance: balance[2].toString(),
            delta: delta[2].toString(),
            actualBalance: actualBalance.toString(),
          })
        )
      );

      newBalances.push([acc, asset, actualBalance] as [
        KeyringPair,
        AssetId,
        bigint
      ]);
    }
  }

  logger.info("Balances are checked");
  return { balances: newBalances };
}
