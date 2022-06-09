import Keyring from "@polkadot/keyring";
import {
  KeyringOptions,
  KeyringPair,
  KeyringPair$Meta,
} from "@polkadot/keyring/types";
import { mnemonicGenerate } from "@polkadot/util-crypto";
import { KeypairType } from "@polkadot/util-crypto/types";

export class HackathonKeyring extends Keyring {
  protected names: Map<string, KeyringPair>;
  protected sudoKey?: KeyringPair;

  constructor(sudoSuri?: string, options?: KeyringOptions) {
    super(options);
    this.names = new Map();
    if (sudoSuri) {
      this.sudoKey = this.addFromUri(sudoSuri, { name: "sudo" });
    }
  }

  getPairByName(name: string): KeyringPair | undefined {
    const pair = this.names.get(name);
    if (pair) {
      return pair;
    } else {
      return undefined;
    }
  }
  createKeyringPair(name: string): KeyringPair {
    const pair = this.addFromUri(mnemonicGenerate(), { name });
    this.names.set(name, pair);
    return pair;
  }
  sudo(): KeyringPair | undefined {
    return this.sudoKey;
  }
  addFromUri(
    suri: string,
    meta?: KeyringPair$Meta,
    type?: KeypairType
  ): KeyringPair {
    const pair = super.addFromUri(suri, meta, type);
    this.names.set(pair.meta.name as string, pair);
    return pair;
  }
}
