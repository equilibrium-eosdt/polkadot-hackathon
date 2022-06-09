import { ApiPromise } from "@polkadot/api";
import {
  AddressOrPair,
  SignerOptions,
  SubmittableExtrinsic,
} from "@polkadot/api/types";
import { ISubmittableResult } from "@polkadot/types/types";
import { logger } from "../logger";
import { fromJS, Map } from "immutable";
import {
  AssetData,
  AssetId,
  assets,
  createAsset,
  getAsset,
  removeAsset,
} from "./assets";
import { burn, getBalances, mint, transfer } from "./balances";
import { deposit, issue, issueSudo, withdraw } from "./distribution";
import { HackathonKeyring } from "./keyring";

export interface HackathonApi extends ApiPromise {
  nonces: Map<AddressOrPair, number>;
  signAndSend<R extends ISubmittableResult = ISubmittableResult>(
    tx: SubmittableExtrinsic<"promise", R>,
    account: AddressOrPair,
    options?: Partial<SignerOptions>
  ): Promise<void>;
  signAndWatch<R extends ISubmittableResult = ISubmittableResult>(
    tx: SubmittableExtrinsic<"promise", R>,
    account: AddressOrPair,
    options?: Partial<SignerOptions>
  ): Promise<R>;
  waitBlock(): Promise<number>;
  waitBlocks(deltaBlocks: number): Promise<number>;
  initNonce(account: AddressOrPair, nonce: number): void;

  createAsset(
    keyring: HackathonKeyring,
    name: string,
    data: AssetData
  ): Promise<void>;
  removeAsset(keyring: HackathonKeyring, name: string): Promise<void>;
  assets(): Promise<{ id: AssetId; data: AssetData }[]>;
  getAsset(name: string): Promise<AssetData | undefined>;

  transfer(
    from: AddressOrPair,
    to: string,
    currency: string,
    amount: bigint
  ): Promise<void>;
  mint(
    keyring: HackathonKeyring,
    who: string,
    currency: string,
    amount: bigint
  ): Promise<void>;
  burn(
    keyring: HackathonKeyring,
    who: string,
    currency: string,
    amount: bigint
  ): Promise<void>;
  getBalance(who: string, currency: string): Promise<bigint>;
  getBalances(who: string): Promise<{ [currency: string]: bigint }>;

  deposit(who: AddressOrPair, currency: string, amount: bigint): Promise<void>;
  withdraw(who: AddressOrPair, currency: string): Promise<void>;
  issue(who: AddressOrPair, currency: string, amount: bigint): Promise<void>;
  issueSudo(
    keyring: HackathonKeyring,
    currency: string,
    amount: bigint
  ): Promise<void>;
}

let nonces = Map<AddressOrPair, number>();

export function wrapApi(api: ApiPromise): HackathonApi {
  return <HackathonApi>{
    nonces,
    consts: { ...api.consts },
    errors: { ...api.errors },
    events: { ...api.events },
    query: { ...api.query },
    rpc: { ...api.rpc },
    tx: { ...api.tx },
    async signAndSend<R extends ISubmittableResult = ISubmittableResult>(
      tx: SubmittableExtrinsic<"promise", R>,
      account: AddressOrPair,
      options?: Partial<SignerOptions>
    ): Promise<void> {
      const signedTx = await tx.signAsync(account, {
        ...options,
        nonce: resolveNonce(this, account),
      });
      await signedTx.send();
    },
    async signAndWatch<R extends ISubmittableResult = ISubmittableResult>(
      tx: SubmittableExtrinsic<"promise", R>,
      account: AddressOrPair,
      options?: Partial<SignerOptions>
    ): Promise<R> {
      const signedTx = await tx.signAsync(account, {
        ...options,
        nonce: resolveNonce(this, account),
      });

      const defer = createDefer<R>();
      const unsub = await signedTx.send((result) => {
        if (result.status.isInvalid) {
          defer.rj(new Error(`Invalid transaction: ${signedTx}`));
        } else if (result.isInBlock) {
          if (result.dispatchError) {
            defer.rj(result.dispatchError);
          }

          for (const { event } of result.events) {
            if (
              api.events.sudo.Sudid.is(event) ||
              api.events.sudo.SudoAsDone.is(event)
            ) {
              const result = event.data[0];
              if (result.isErr) {
                defer.rj(result.asErr);
              }
            }
          }

          unsub();
          defer.rs(result);
        }
      });

      return defer.p;
    },
    async waitBlock(): Promise<number> {
      let newBlock = false;

      const defer = createDefer<number>();
      const unsub = await api.rpc.chain.subscribeNewHeads((header) => {
        const newBlockNumber = header.number.toNumber();
        if (newBlock) {
          unsub();
          defer.rs(newBlockNumber);
        } else {
          logger.info(`currBlockNumber: ${newBlockNumber}`);
          newBlock = true;
        }
      });

      return defer.p;
    },
    async waitBlocks(deltaBlocks: number): Promise<number> {
      const currentBlock = (
        await api.rpc.chain.getBlock()
      ).block.header.number.toNumber();

      const defer = createDefer<number>();
      const unsub = await api.rpc.chain.subscribeNewHeads((header) => {
        const newBlockNumber = header.number.toNumber();
        // logger.info(`currBlockNumber: ${newBlockNumber}`);
        if (newBlockNumber - currentBlock >= deltaBlocks) {
          unsub();
          defer.rs(newBlockNumber);
        }
      });

      return defer.p;
    },
    initNonce(account: AddressOrPair, nonce: number) {
      initNonce(this, account, nonce);
    },
    async createAsset(
      keyring: HackathonKeyring,
      name: string,
      data: AssetData
    ): Promise<void> {
      await createAsset(this, keyring, name, data);
    },
    async removeAsset(keyring: HackathonKeyring, name: string): Promise<void> {
      await removeAsset(this, keyring, name);
    },
    async assets(): Promise<{ id: AssetId; data: AssetData }[]> {
      return await assets(this);
    },
    async getAsset(name: string): Promise<AssetData | undefined> {
      return await getAsset(this, name);
    },
    async transfer(
      from: AddressOrPair,
      to: string,
      currency: string,
      amount: bigint
    ): Promise<void> {
      await transfer(this, from, to, currency, amount);
    },
    async mint(
      keyring: HackathonKeyring,
      who: string,
      currency: string,
      amount: bigint
    ): Promise<void> {
      await mint(this, keyring, who, currency, amount);
    },
    async burn(
      keyring: HackathonKeyring,
      who: string,
      currency: string,
      amount: bigint
    ): Promise<void> {
      await burn(this, keyring, who, currency, amount);
    },
    async getBalance(who: string, currency: string): Promise<bigint> {
      return (await api.query.balances.accounts(who, currency)).toBigInt();
    },
    async getBalances(who: string): Promise<{ [currency: string]: bigint }> {
      return await getBalances(this, who);
    },
    async deposit(
      who: AddressOrPair,
      currency: string,
      amount: bigint
    ): Promise<void> {
      await deposit(this, who, currency, amount);
    },
    async withdraw(who: AddressOrPair, currency: string): Promise<void> {
      await withdraw(this, who, currency);
    },
    async issue(
      who: AddressOrPair,
      currency: string,
      amount: bigint
    ): Promise<void> {
      await issue(this, who, currency, amount);
    },
    async issueSudo(
      keyring: HackathonKeyring,
      currency: string,
      amount: bigint
    ): Promise<void> {
      await issueSudo(this, keyring, currency, amount);
    },
  };
}

export function createDefer<T = void>(): {
  rs: (value: T) => void;
  rj: (value: any) => void;
  p: Promise<T>;
} {
  const result = {} as {
    rs: (value: T) => void;
    rj: (value: any) => void;
    p: Promise<T>;
  };
  result.p = new Promise((rs, rj) => {
    result.rs = rs;
    result.rj = rj;
  });
  result.p.catch(() => {});
  return result;
}

function resolveNonce(api: HackathonApi, account: AddressOrPair): number {
  const acc = fromJS(account);
  if (api.nonces.has(acc)) {
    const currNonce = api.nonces.get(acc);
    api.nonces = api.nonces.set(acc, currNonce + 1);
    return currNonce;
  } else {
    throw new Error(
      `Nonce is not initialized for acc ${JSON.stringify(account)} !`
    );
  }
}

function initNonce(api: HackathonApi, account: AddressOrPair, nonce: number) {
  const acc = fromJS(account);
  api.nonces = api.nonces.set(acc, nonce);
}
