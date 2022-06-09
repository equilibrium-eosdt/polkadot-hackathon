import { logger } from "./logger/index";
import "./interfaces/augment-api";
import "./interfaces/augment-types";
import { HackathonTestCase } from "./command";

async function main(): Promise<void> {
  await HackathonTestCase.run();
}

main().catch((err: any) => {
  if (err instanceof Error) {
    logger.error(err);
  } else if (err instanceof Object) {
    logger.error(JSON.stringify(err, undefined, 2));
  } else {
    logger.error({ err, msg: "Unknown error" });
  }
});
