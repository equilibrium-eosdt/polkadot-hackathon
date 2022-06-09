import assert from "assert";
import { logger } from "../logger";
import { createDefer, HackathonApi as Api } from "../chain-ops/api";
import { AssetId } from "../chain-ops/assets";
import { UnsubscribePromise } from "@polkadot/api/types";

export let prices: { [asset: string]: bigint } = {
  usd: 1_000_000_000_000_000_000n,
};

export function convertToStable(balance: [AssetId, bigint][]): bigint {
  return balance
    .map(([asset, balance]) => {
      return (balance * prices[asset]) / prices.usd;
    })
    .reduce((a, b) => a + b);
}

let priceDefer = createDefer();
export async function updatePricesEvent(): Promise<void> {
  logger.info("Waiting for updatePrice event");
  await priceDefer.p;
}

export async function updatePrices(api: Api): Promise<UnsubscribePromise> {
  return await api.query.system.events((events) => {
    let isUpdated = false;
    for (const { event } of events) {
      if (api.events.oracle.UpdatePrice.is(event)) {
        isUpdated = true;
        const [assetRaw, priceRaw] = event.data;
        const asset = assetRaw.toHuman() as string;
        const price = priceRaw.toBigInt();
        assert.notEqual(
          prices[asset],
          price,
          new Error(`Price for ${asset} should change`)
        );
        prices[asset] = price;
      }
    }
    if (isUpdated) {
      const fmtPrices = Object.fromEntries(
        Object.entries(prices).map(([asset, price]) => {
          return [asset, Number(price / 1_000_000_000_000n) / 1_000_000] as [
            string,
            number
          ];
        })
      );
      logger.info(`Prices updated: ${JSON.stringify(fmtPrices)}`);
      priceDefer.rs();
      priceDefer = createDefer();
    }
  });
}
