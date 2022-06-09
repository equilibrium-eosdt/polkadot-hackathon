#![cfg_attr(not(feature = "std"), no_std)]

use core::ops::Add;

use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, traits::Get};
use frame_system::{ensure_signed_or_root, offchain::*};
pub use pallet::*;
use primitives::{assets::AssetGetter, currency::Currency, prices::PriceGetter};
use sp_runtime::{traits::Zero, DispatchError, DispatchResult};
use sp_std::collections::btree_map::BTreeMap;

type AssetIdOf<T> = <<T as Config>::Assets as primitives::assets::AssetGetter>::AssetId;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Balance: Parameter
            + Member
            + sp_runtime::traits::AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + Into<u128>
            + From<u128>;
        type Assets: AssetGetter;
        type Currency: Currency<
            AccountId = Self::AccountId,
            AssetId = AssetIdOf<Self>,
            Balance = Self::Balance,
            CurrencyError = DispatchError,
        >;
        type Prices: PriceGetter<
            AssetId = AssetIdOf<Self>,
            Balance = Self::Balance,
            PriceError = DispatchError,
        >;
        type ModuleId: Get<Self::AccountId>;
        type TreasuryModuleId: Get<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Deposits<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        AssetIdOf<T>,
        T::Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type TotalDeposits<T: Config> =
        StorageMap<_, Blake2_128Concat, AssetIdOf<T>, T::Balance, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewDeposit {
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
            total: T::Balance,
        },
        Withdraw {
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        },
        Issued {
            asset: AssetIdOf<T>,
            amount: T::Balance,
        },
        Redistributed {
            who: T::AccountId,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoDeposit,
        BlockValidation,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[frame_support::transactional]
        #[pallet::weight(10_000)]
        pub fn deposit(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            T::Currency::burn(&who, &asset, amount)?;
            let total = Deposits::<T>::mutate(&who, &asset, |total| {
                *total += amount;
                *total
            });
            TotalDeposits::<T>::mutate(&asset, |total| *total += amount);

            Self::deposit_event(Event::<T>::NewDeposit {
                who,
                asset,
                amount,
                total,
            });
            Ok(().into())
        }

        #[frame_support::transactional]
        #[pallet::weight(10_000)]
        pub fn withdraw(origin: OriginFor<T>, asset: AssetIdOf<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let amount = Deposits::<T>::take(&who, &asset);
            ensure!(!amount.is_zero(), Error::<T>::NoDeposit);
            T::Currency::mint(&who, &asset, amount)?;
            TotalDeposits::<T>::mutate(&asset, |total| *total -= amount);

            Self::deposit_event(Event::<T>::Withdraw { who, asset, amount });
            Ok(().into())
        }

        #[frame_support::transactional]
        #[pallet::weight(10_000)]
        pub fn issue(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            amount: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let distribution = T::ModuleId::get();
            match ensure_signed_or_root(origin)? {
                Some(who) => T::Currency::transfer(&who, &distribution, &asset, amount)?,
                None => T::Currency::mint(&distribution, &asset, amount)?,
            };
            Self::deposit_event(Event::<T>::Issued { asset, amount });
            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(n: T::BlockNumber) -> Weight {
            let total_in_stable = Self::total_in_stable();
            if !total_in_stable.is_zero() {
                let mut issuances = BTreeMap::<AssetIdOf<T>, _>::new();
                for (who, asset) in Deposits::<T>::iter_keys() {
                    let deposit_in_stable = Self::deposit_in_stable(&who);
                    let issuance = issuances.entry(asset.clone()).or_insert_with(|| {
                        (
                            T::Currency::balance(&T::ModuleId::get(), &asset),
                            T::Balance::zero(),
                        )
                    });

                    if let Some(to_redistribute) =
                        Self::multiply_by_rational(issuance.0, deposit_in_stable, total_in_stable)
                    {
                        if !to_redistribute.is_zero() {
                            issuance.1 += to_redistribute;
                            let _ = Self::inner_redistribute(
                                who.clone(),
                                asset.clone(),
                                to_redistribute,
                                n,
                            );
                        }
                    }
                }

                let treasury = T::TreasuryModuleId::get();
                issuances.into_iter().for_each(
                    |(asset, (redistribution, actual_redistribution))| {
                        if actual_redistribution < redistribution {
                            let residue = redistribution - actual_redistribution;
                            let _ = Self::inner_redistribute(
                                treasury.clone(),
                                asset.clone(),
                                residue,
                                n,
                            );
                        }
                    },
                );
            }

            0
        }
    }
}

impl<T: Config> Pallet<T> {
    fn total_in_stable() -> T::Balance {
        TotalDeposits::<T>::iter()
            .filter_map(|(ref asset, amount)| T::Prices::to_stable_amount(asset, amount).ok())
            .fold(T::Balance::zero(), T::Balance::add)
    }

    fn deposit_in_stable(who: &T::AccountId) -> T::Balance {
        Deposits::<T>::iter_prefix(who)
            .filter_map(|(ref asset, amount)| T::Prices::to_stable_amount(asset, amount).ok())
            .fold(T::Balance::zero(), T::Balance::add)
    }

    fn multiply_by_rational(a: T::Balance, b: T::Balance, c: T::Balance) -> Option<T::Balance> {
        sp_runtime::helpers_128bit::multiply_by_rational(a.into(), b.into(), c.into())
            .ok()
            .map(u128::into)
    }

    fn inner_redistribute(
        who: T::AccountId,
        asset: AssetIdOf<T>,
        amount: T::Balance,
        _block: T::BlockNumber,
    ) -> DispatchResult {
        T::Currency::transfer(&T::ModuleId::get(), &who, &asset, amount)?;
        Self::deposit_event(Event::<T>::Redistributed { who, asset, amount });
        Ok(())
    }
}
