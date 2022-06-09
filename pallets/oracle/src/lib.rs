#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    ensure,
    traits::{Get, Hooks, Randomness},
    weights::Weight,
};
pub use pallet::*;
use primitives::{assets::AssetGetter, prices::PriceGetter};
use sp_runtime::{
    traits::{CheckedDiv, One, TrailingZeroInput, Zero},
    DispatchError, DispatchResult, FixedPointNumber, FixedPointOperand, KeyTypeId,
};

type AssetIdOf<T> = <<T as Config>::Assets as primitives::assets::AssetGetter>::AssetId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"orac");

pub mod crypto {
    use sp_runtime::app_crypto::{app_crypto, sr25519};

    use super::KEY_TYPE;
    app_crypto!(sr25519, KEY_TYPE);

    pub type AuthoritySignature = sp_core::sr25519::Signature;
    pub type AuthorityId = sp_core::sr25519::Public;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Balance: Parameter
            + Member
            + sp_runtime::traits::AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + FixedPointOperand;
        type Price: Parameter
            + Member
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + FixedPointNumber;
        type Assets: AssetGetter<AssetError = DispatchError>;
        type StableAsset: Get<AssetIdOf<Self>>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type Precision: Get<Self::Price>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Prices<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, T::Price, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub prices: Vec<(AssetIdOf<T>, T::Price)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { prices: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Prices::<T>::insert(T::StableAsset::get(), T::Price::one());
            for (asset, price) in &self.prices {
                Prices::<T>::insert(asset, price);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        UpdatePrice {
            asset: AssetIdOf<T>,
            price: T::Price,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoPrice,
        SetPriceForStableAsset,
        SetZeroPrice,
        Math,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn force_set_price(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            price: T::Price,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::set_price(asset, price)?;
            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            if (n % T::BlockNumber::from(5u8)).is_zero() {
                for asset in T::Assets::get_all_in_ord() {
                    if asset == T::StableAsset::get() {
                        continue;
                    }
                    let price = Self::gen_price(("pallet-oracle", n, &asset));
                    if let Err(e) = Self::set_price(asset, price) {
                        frame_support::runtime_print!("SetPriceError: {:?}", e);
                    }
                }
                10_000
            } else {
                0
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    fn set_price(asset: AssetIdOf<T>, price: T::Price) -> DispatchResult {
        ensure!(
            asset != T::StableAsset::get() || <T::Price as One>::is_one(&price),
            Error::<T>::SetPriceForStableAsset,
        );
        ensure!(
            !<T::Price as Zero>::is_zero(&price),
            Error::<T>::SetZeroPrice,
        );
        T::Assets::check(&asset)?;
        Prices::<T>::insert(&asset, &price);

        Self::deposit_event(Event::<T>::UpdatePrice { asset, price });
        Ok(())
    }

    fn gen_price<S: Encode>(seed: S) -> T::Price {
        let mut i = 0u32;
        loop {
            let bytes = T::Randomness::random(&(&seed, i).encode()[..]).0;
            let mut bytes = bytes.as_ref()[0..8].to_vec();
            bytes[7] &= 0x0f;
            if let Ok(delta) = <T::Price as Decode>::decode(&mut TrailingZeroInput::new(&bytes[..]))
            {
                if delta < <T::Price as One>::one() {
                    let precision = T::Precision::get();
                    break <T::Price as One>::one() + (delta * precision) / precision;
                }
            }
            i += 1;
        }
    }
}

impl<T: Config> PriceGetter for Pallet<T> {
    type AssetId = AssetIdOf<T>;
    type Balance = T::Balance;
    type Price = T::Price;
    type PriceError = DispatchError;

    fn get(id: &AssetIdOf<T>) -> Result<T::Price, DispatchError> {
        if id == &T::StableAsset::get() {
            Ok(T::Price::one())
        } else {
            Prices::<T>::get(id).ok_or(Error::<T>::NoPrice.into())
        }
    }

    fn base_asset() -> AssetIdOf<T> {
        T::StableAsset::get()
    }

    fn exchange(
        from: &AssetIdOf<T>,
        to: &AssetIdOf<T>,
        amount: T::Balance,
    ) -> Result<T::Balance, DispatchError> {
        let price = (Self::get(from)?, Self::get(to)?);
        let rel_price = price.0.checked_div(&price.1).ok_or(Error::<T>::Math)?;
        Ok(rel_price.saturating_mul_int(amount))
    }
}
