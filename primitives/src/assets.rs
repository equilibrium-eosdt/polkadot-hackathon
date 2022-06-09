use frame_support::pallet_prelude::*;
use serde::*;
use sp_std::{fmt, prelude::*};
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::String;

const MAX_ASSET_ID_LEN: u32 = 8;

pub fn tok() -> AssetId {
    unsafe { AssetId::new_unchecked(b"coin") }
}

pub fn usd() -> AssetId {
    unsafe { AssetId::new_unchecked(b"usd") }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, scale_info::TypeInfo)]
pub struct AssetId(Vec<u8>);

impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "${}",
            String::from_utf8(self.0.clone()).unwrap_or_else(|_| String::from("UNKNOWN"))
        )
    }
}

impl AssetId {
    pub unsafe fn new_unchecked(inner: &[u8]) -> Self {
        Self(inner.to_vec())
    }

    pub fn new(inner: impl IntoIterator<Item = u8>) -> Option<Self> {
        let inner: Vec<u8> = inner.into_iter().collect();
        if inner.len() as u32 <= MAX_ASSET_ID_LEN {
            Some(Self(inner))
        } else {
            None
        }
    }

    pub fn from_utf8(inner: &str) -> Option<Self> {
        Self::new(inner.bytes())
    }
}

impl MaxEncodedLen for AssetId {
    fn max_encoded_len() -> usize {
        u32::max_encoded_len() + MAX_ASSET_ID_LEN as usize
    }
}

impl TryFrom<Vec<u8>> for AssetId {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Decode, Encode, MaxEncodedLen, PartialEq, Eq, scale_info::TypeInfo)]
pub struct AssetData {
    pub decimals: u8,
}

pub trait AssetGetter {
    type AssetId: Parameter + Member + Ord + MaybeSerializeDeserialize + MaxEncodedLen;
    type AssetData: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
    type AssetError;

    fn get(id: &Self::AssetId) -> Result<Self::AssetData, Self::AssetError>;

    fn get_main() -> Self::AssetId;

    fn get_all() -> Vec<Self::AssetId>;

    fn get_all_in_ord() -> Vec<Self::AssetId> {
        let mut assets = Self::get_all();
        assets.sort_by_key(|a| a.encode());
        assets
    }

    fn check(id: &Self::AssetId) -> Result<(), Self::AssetError> {
        Self::get(id).map(|_| ())
    }
}

pub trait OnAssetCreate<AssetId> {
    fn on_asset_create(asset: &AssetId) -> Option<()>;
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<AssetId> OnAssetCreate<AssetId> for Tuple {
    fn on_asset_create(asset: &AssetId) -> Option<()> {
        for_tuples!(#( Tuple::on_asset_create(asset)?; )*);
        Some(())
    }
}
