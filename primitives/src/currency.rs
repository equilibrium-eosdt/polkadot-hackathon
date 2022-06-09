use frame_support::pallet_prelude::*;

pub trait Currency {
    type AccountId: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
    type Balance: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen + Default + Copy;
    type AssetId: Parameter + Member + MaybeSerializeDeserialize + MaxEncodedLen;
    type CurrencyError;

    fn mint(
        who: &Self::AccountId,
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> Result<(), Self::CurrencyError>;

    fn burn(
        who: &Self::AccountId,
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> Result<(), Self::CurrencyError>;

    fn transfer(
        from: &Self::AccountId,
        to: &Self::AccountId,
        asset: &Self::AssetId,
        amount: Self::Balance,
    ) -> Result<(), Self::CurrencyError>;

    fn balance(who: &Self::AccountId, asset: &Self::AssetId) -> Self::Balance;

    fn total_in_stable(who: &Self::AccountId) -> Self::Balance;

    type BalancesIter: Iterator<Item = (Self::AccountId, Self::AssetId, Self::Balance)>;

    fn iter_balances() -> Self::BalancesIter;
}
