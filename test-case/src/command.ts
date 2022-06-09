import { Command, flags } from "@oclif/command";
import { logger } from "./logger/index";
import { ApiPromise, WsProvider } from "@polkadot/api";
import assert from "assert";
import * as definitions from "./interfaces/definitions";
import { wrapApi } from "./chain-ops/api";
import { HackathonKeyring as Keyring } from "./chain-ops/keyring";
import { updatePrices } from "./suite/prices";
import { loop } from "./suite";

const requiredNode = "Hack-a-node";

export class HackathonTestCase extends Command {
  static description = "Hackathon test case";

  static flags = {
    version: flags.version({ char: "v" }),
    help: flags.help({ char: "h" }),
    clients: flags.integer({
      char: "N",
      description: "Initial amount of clients in distribution pallet",
      default: 10,
      required: true,
    }),
    assets: flags.integer({
      char: "M",
      description: "Initial amount of assets in oracle pallet",
      default: 20,
      required: true,
    }),
  };

  async run(): Promise<void> {
    const {
      flags: { clients, assets },
    } = this.parse();

    const types = Object.values(definitions).reduce(
      (res, { types }): object => ({ ...res, ...types }),
      {}
    );
    logger.debug(types);
    let api = await ApiPromise.create({
      provider: new WsProvider("ws://127.0.0.1:9944"),
      throwOnConnect: true,
      types,
    });

    const [chain, nodeName, nodeVersion, properties] = await Promise.all([
      api.rpc.system.chain().then((s) => s.toString()),
      api.rpc.system.name().then((s) => s.toString()),
      api.rpc.system.version().then((s) => s.toString()),
      api.rpc.system.properties(),
    ]);
    if (nodeName != requiredNode) {
      throw new Error(
        `Unsupported node: ${nodeName}, ${requiredNode} required`
      );
    }

    logger.info("Connected to chain");
    logger.info({ chain, nodeName, nodeVersion, properties });

    let ss58Format = undefined;
    if (properties.ss58Format.isSome) {
      ss58Format = Number(properties.ss58Format.unwrap().toBigInt());
    }
    let keyring = new Keyring("//Alice", {
      type: "sr25519",
      ss58Format,
    });
    assert.equal(
      (await api.query.sudo.key()).toHuman(),
      keyring.sudo()!.address,
      "Wrong sudo seed"
    );

    let wrappedApi = wrapApi(api);
    const unsub = await updatePrices(wrappedApi);
    try {
      await loop(wrappedApi, keyring, { clients, assets });
    } finally {
      unsub();
      await api.disconnect();
    }
  }
}
