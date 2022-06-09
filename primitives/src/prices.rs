use frame_support::pallet_prelude::*;

pub trait PriceGetter {
    type AssetId: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
    type Balance: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
    type Price: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
    type PriceError;

    fn get(id: &Self::AssetId) -> Result<Self::Price, Self::PriceError>;

    fn base_asset() -> Self::AssetId;

    fn exchange(
        from: &Self::AssetId,
        to: &Self::AssetId,
        amount: Self::Balance,
    ) -> Result<Self::Balance, Self::PriceError>;

    fn to_stable_amount(
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> Result<Self::Balance, Self::PriceError> {
        Self::exchange(asset, &Self::base_asset(), amount)
    }
}
