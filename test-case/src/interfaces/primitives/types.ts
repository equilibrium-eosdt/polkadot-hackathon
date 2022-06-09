// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import type { Bytes, Struct, u128, u8 } from '@polkadot/types-codec';
import type { FixedU128 } from '@polkadot/types/interfaces/runtime';

/** @name AssetData */
export interface AssetData extends Struct {
  readonly decimals: u8;
}

/** @name AssetId */
export interface AssetId extends Bytes {}

/** @name Balance */
export interface Balance extends u128 {}

/** @name Price */
export interface Price extends FixedU128 {}

export type PHANTOM_PRIMITIVES = 'primitives';
