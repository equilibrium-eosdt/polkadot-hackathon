#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchError, traits::Get};
pub use pallet::*;
use primitives::assets::AssetGetter;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::assets::OnAssetCreate;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type AssetId: Parameter
            + Member
            + Ord
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + TryFrom<Vec<u8>>;
        type AssetData: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
        type MainAsset: Get<Self::AssetId>;
        type OnAssetCreate: OnAssetCreate<Self::AssetId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Assets<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetData, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub assets: Vec<(T::AssetId, T::AssetData)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { assets: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (id, data) in &self.assets {
                Assets::<T>::insert(id, data);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created { id: T::AssetId, data: T::AssetData },
        Removed { id: T::AssetId, data: T::AssetData },
    }

    #[pallet::error]
    pub enum Error<T> {
        WrongName,
        FailedAssetCreateHook,
        NotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create(
            origin: OriginFor<T>,
            name: Vec<u8>,
            data: T::AssetData,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let id = T::AssetId::try_from(name).map_err(|_| Error::<T>::WrongName)?;
            Assets::<T>::insert(&id, &data);
            T::OnAssetCreate::on_asset_create(&id).ok_or(Error::<T>::FailedAssetCreateHook)?;

            Self::deposit_event(Event::Created { id, data });
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn remove(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let id = T::AssetId::try_from(name).map_err(|_| Error::<T>::WrongName)?;
            let data = Assets::<T>::take(&id).ok_or(Error::<T>::NotFound)?;

            Self::deposit_event(Event::Removed { id, data });
            Ok(().into())
        }
    }
}

impl<T: Config> AssetGetter for Pallet<T> {
    type AssetId = T::AssetId;
    type AssetData = T::AssetData;
    type AssetError = DispatchError;

    fn get(id: &T::AssetId) -> Result<T::AssetData, DispatchError> {
        Assets::<T>::get(id).ok_or(Error::<T>::NotFound.into())
    }

    fn get_main() -> T::AssetId {
        T::MainAsset::get()
    }

    fn get_all() -> Vec<T::AssetId> {
        Assets::<T>::iter_keys().collect()
    }
}
