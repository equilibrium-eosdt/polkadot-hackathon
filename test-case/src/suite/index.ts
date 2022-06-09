import { HackathonApi as Api } from "../chain-ops/api";
import { HackathonKeyring as Keyring } from "../chain-ops/keyring";
import { logger } from "../logger";
import { TestParams } from "../params";
import { check, iteration } from "./iteration";
import { init } from "./init";
import { finish } from "./finish";

export const mainAsset = "coin";
export const usdAsset = "usd";
export const oneToken = 1_000_000_000_000n;

export async function loop(
  api: Api,
  keyring: Keyring,
  params: TestParams
): Promise<void> {
  for (;;) {
    logger.info(
      `*** Test case is starting for N=${params.clients}, M=${params.assets} ***`
    );
    const result = await suite(api, keyring, params);
    if (!result) {
      break;
    }
    params.clients += 10;
  }
}

export async function suite(
  api: Api,
  keyring: Keyring,
  params: TestParams
): Promise<boolean> {
  let { accounts, balances, assets } = await init(api, keyring, params);

  let [bads, goods] = [0, 0];
  for (let it = 1; it <= 5; it++) {
    logger.info(`Iteration ${it}`);
    const { deltas, isGood } = await iteration(api, keyring, accounts, assets);
    if (!isGood) {
      logger.error(`Iteration ${it} failed`);
      bads++;
    } else {
      logger.info(`Iteration ${it} finished successefuly`);
      goods++;
    }
    ({ balances } = await check(api, accounts, assets, balances, deltas));
  }

  await finish(api, keyring, accounts, balances, assets);

  if (bads == 0) {
    logger.info(
      `*** Test case finished successefuly for N=${params.clients}, M=${params.assets} ***`
    );
  } else if (goods > bads) {
    logger.warn(
      `*** Test case finished sub-successefuly (${JSON.stringify({
        goods,
        bads,
      })}) for N=${params.clients}, M=${params.assets} ***`
    );
  } else if (goods > 0) {
    logger.warn(
      `*** Test case finished unsuccessefuly (${JSON.stringify({
        goods,
        bads,
      })}) for N=${params.clients}, M=${params.assets} ***`
    );
  } else {
    throw new Error(
      `*** Test case failed for N=${params.clients}, M=${params.assets} ***`
    );
  }

  return true;
}
