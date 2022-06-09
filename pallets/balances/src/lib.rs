#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    traits::Get,
};
pub use pallet::*;
use primitives::{
    assets::{AssetGetter, OnAssetCreate},
    currency::Currency,
    prices::PriceGetter,
};
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Zero},
    transaction_validity::InvalidTransaction,
    DispatchError,
};

type AssetIdOf<T> = <<T as Config>::Assets as primitives::assets::AssetGetter>::AssetId;

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
            + MaxEncodedLen;
        type Assets: AssetGetter<AssetError = DispatchError>;
        type Prices: PriceGetter<
            AssetId = AssetIdOf<Self>,
            Balance = Self::Balance,
            PriceError = DispatchError,
        >;
        type TreasuryModuleId: Get<Self::AccountId>;
        type InitialAssetIssuance: Get<Self::Balance>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Accounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AssetIdOf<T>,
        T::Balance,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub treasury: Vec<AssetIdOf<T>>,
        pub balances: Vec<(T::AccountId, Vec<(AssetIdOf<T>, T::Balance)>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                treasury: vec![],
                balances: vec![],
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let treasury = T::TreasuryModuleId::get();
            let balance = T::InitialAssetIssuance::get();
            for asset in &self.treasury {
                Accounts::<T>::insert(&treasury, asset, balance);
            }

            for (who, balances) in &self.balances {
                for (asset, balance) in balances {
                    let _ = <Pallet<T> as Currency>::mint(who, asset, *balance);
                }
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TransactionPayment {
            who: T::AccountId,
            fee: T::Balance,
            tip: T::Balance,
        },
        Minted {
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        },
        Burnt {
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        },
        Transfer {
            from: T::AccountId,
            to: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        Debt,
        Overflow,
        EmptyTreasury,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            <Self as Currency>::mint(&who, &asset, amount)?;
            Self::deposit_event(Event::<T>::Minted { who, asset, amount });
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            <Self as Currency>::burn(&who, &asset, amount)?;
            Self::deposit_event(Event::<T>::Burnt { who, asset, amount });
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            <Self as Currency>::transfer(&from, &to, &asset, amount)?;
            Self::deposit_event(Event::<T>::Transfer {
                from,
                to,
                asset,
                amount,
            });
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn add(who: &T::AccountId, asset: &AssetIdOf<T>, amount: T::Balance) -> DispatchResult {
        Accounts::<T>::try_mutate(who, asset, |balance| {
            *balance = balance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
            Ok(())
        })
    }

    fn sub(who: &T::AccountId, asset: &AssetIdOf<T>, amount: T::Balance) -> DispatchResult {
        Accounts::<T>::try_mutate(who, asset, |balance| {
            *balance = balance.checked_sub(&amount).ok_or(Error::<T>::Debt)?;
            Ok(())
        })
    }

    fn treasury_add(asset: &AssetIdOf<T>, amount: T::Balance) -> DispatchResult {
        Self::add(&T::TreasuryModuleId::get(), asset, amount)
    }

    fn treasury_sub(asset: &AssetIdOf<T>, amount: T::Balance) -> DispatchResult {
        Self::sub(&T::TreasuryModuleId::get(), asset, amount)
            .map_err(|_| Error::<T>::EmptyTreasury.into())
    }
}

impl<T: Config> Currency for Pallet<T> {
    type AccountId = T::AccountId;
    type Balance = T::Balance;
    type AssetId = AssetIdOf<T>;
    type CurrencyError = DispatchError;

    fn mint(
        who: &T::AccountId,
        asset: &AssetIdOf<T>,
        amount: T::Balance,
    ) -> Result<(), DispatchError> {
        T::Assets::check(asset)?;
        Self::treasury_sub(asset, amount)?;
        Self::add(who, asset, amount)?;
        Ok(())
    }

    fn burn(
        who: &T::AccountId,
        asset: &AssetIdOf<T>,
        amount: T::Balance,
    ) -> Result<(), DispatchError> {
        T::Assets::check(asset)?;
        Self::sub(who, asset, amount)?;
        Self::treasury_add(asset, amount)?;
        Ok(())
    }

    fn transfer(
        from: &T::AccountId,
        to: &T::AccountId,
        asset: &AssetIdOf<T>,
        amount: T::Balance,
    ) -> Result<(), DispatchError> {
        T::Assets::check(asset)?;
        Self::sub(from, asset, amount)?;
        Self::add(to, asset, amount)?;
        Ok(())
    }

    fn balance(who: &T::AccountId, asset: &AssetIdOf<T>) -> T::Balance {
        Accounts::<T>::get(who, asset)
    }

    fn total_in_stable(who: &Self::AccountId) -> Self::Balance {
        Accounts::<T>::iter_prefix(who).fold(T::Balance::zero(), |acc, (asset, balance)| {
            if let Ok(amount) = T::Prices::to_stable_amount(&asset, balance) {
                acc + amount
            } else {
                acc
            }
        })
    }

    type BalancesIter =
        frame_support::storage::PrefixIterator<(T::AccountId, AssetIdOf<T>, T::Balance)>;

    fn iter_balances() -> Self::BalancesIter {
        Accounts::<T>::iter()
    }
}

impl<T: Config + pallet_transaction_payment::Config>
    pallet_transaction_payment::OnChargeTransaction<T> for Pallet<T>
{
    type Balance = T::Balance;
    type LiquidityInfo = Option<T::Balance>;

    fn withdraw_fee(
        who: &T::AccountId,
        _call: &T::Call,
        _dispatch_info: &sp_runtime::traits::DispatchInfoOf<T::Call>,
        fee: T::Balance,
        _tip: T::Balance,
    ) -> Result<Self::LiquidityInfo, frame_support::unsigned::TransactionValidityError> {
        if fee.is_zero() {
            return Ok(None);
        }

        Self::sub(who, &T::Assets::get_main(), fee).map_err(|_| InvalidTransaction::Payment)?;

        Ok(Some(fee))
    }

    fn correct_and_deposit_fee(
        who: &<T>::AccountId,
        _dispatch_info: &sp_runtime::traits::DispatchInfoOf<<T>::Call>,
        _post_info: &sp_runtime::traits::PostDispatchInfoOf<<T>::Call>,
        corrected_fee: T::Balance,
        tip: T::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), frame_support::unsigned::TransactionValidityError> {
        if let Some(paid) = already_withdrawn {
            let refund_amount = paid - corrected_fee;
            Self::treasury_add(&T::Assets::get_main(), corrected_fee)
                .map_err(|_| InvalidTransaction::Payment)?;
            Self::add(who, &T::Assets::get_main(), refund_amount)
                .map_err(|_| InvalidTransaction::Payment)?;
        }

        Self::deposit_event(Event::TransactionPayment {
            who: who.clone(),
            fee: corrected_fee,
            tip,
        });
        Ok(())
    }
}

impl<T: Config> OnAssetCreate<AssetIdOf<T>> for Pallet<T> {
    fn on_asset_create(asset: &AssetIdOf<T>) -> Option<()> {
        let treasury = T::TreasuryModuleId::get();
        let balance = T::InitialAssetIssuance::get();
        Accounts::<T>::insert(&treasury, asset, balance);
        Some(())
    }
}
