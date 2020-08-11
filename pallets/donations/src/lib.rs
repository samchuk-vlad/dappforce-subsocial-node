#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
// use sp_runtime::traits::{Saturating, SaturatedConversion};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::{DispatchResult, DispatchError},
    traits::{Currency, ExistenceRequirement}
};
use frame_system::{self as system, ensure_signed};

use pallet_permissions::SpacePermission;
use pallet_posts::{Module as Posts, PostId};
use pallet_spaces::{Module as Spaces};
use pallet_utils::{Content, WhoAndWhen, SpaceId};

type BalanceOf<T> = <<T as pallet_utils::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub type DonationId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum DonationRecipient<AccountId> {
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Donation<T: Trait> {
    pub id: DonationId,
    pub created: WhoAndWhen<T>,
    pub recipient: DonationRecipient<T::AccountId>,
    pub donation_wallet: T::AccountId,
    pub amount: BalanceOf<T>,
    pub comment_id: Option<PostId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct DonationSettings<T: Trait> {
    pub donations_enabled: bool,
    pub min_amount: Option<BalanceOf<T>>,
    pub max_amount: Option<BalanceOf<T>>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct DonationSettingsUpdate<T: Trait> {
    pub donations_enabled: Option<bool>,
    pub min_amount: Option<Option<BalanceOf<T>>>,
    pub max_amount: Option<Option<BalanceOf<T>>>,
}

pub trait Trait: system::Trait
    + pallet_posts::Trait
    + pallet_spaces::Trait
    + pallet_utils::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as DonationsModule {

        pub NextDonationId get(fn next_donation_id):
            DonationId = 1;

        pub DonationById get(fn donation_by_id):
            map hasher(twox_64_concat) DonationId => Option<Donation<T>>;

        pub DonationIdsByBacker get(fn donations_by_backer):
            map hasher(blake2_128_concat) T::AccountId => Vec<DonationId>;

        pub DonationIdsByRecipient get(fn donation_ids_by_recipient):
            map hasher(blake2_128_concat) DonationRecipient<T::AccountId> => Vec<DonationId>;

        pub DonationWalletByRecipient get(fn donation_wallet_by_recipient):
            map hasher(blake2_128_concat) DonationRecipient<T::AccountId> => Option<T::AccountId>;

        pub DonationSettingsBySpace get(fn donation_settings_by_space):
            map hasher(twox_64_concat) SpaceId => Option<DonationSettings<T>>;
    }
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as system::Trait>::AccountId,
        DonationRecipient = DonationRecipient<<T as system::Trait>::AccountId>,
        BalanceOf = BalanceOf<T>
    {
        Donated(
            // Backer - from whom if was donated.
            AccountId,
            // To which recipient it was donated.
            DonationRecipient,
            // Amount of donated tokens.
            BalanceOf
        ),
        DonationWalletSet(
            // Origin - who set a new wallet.
            AccountId,
            // For which recipient a new wallet was set.
            DonationRecipient,
            // An address of a new wallet.
            AccountId
        ),
        DonationWalletRemoved(
            // Origin - who removed a donation wallet.
            AccountId,
            // From which recipient a wallet was removed.
            DonationRecipient
        ),
        DonationSettingsUpdated(
            // Origin - who updated the donation settings.
            AccountId,
            // For what space the donation settings have been updated.
            SpaceId
        ),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Thrown if an origin is not allowed to change a donation wallet,
        /// because their are not an owner of this repicient (e.g. space or post owner).
        NotRecipientManager,
        /// Nothing to update in the donation settings.
        NoUpdatesForDonationSettings,
    }
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    type Error = Error<T>;

    fn deposit_event() = default;

    #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
    pub fn donate(
        origin,
        recipient: DonationRecipient<T::AccountId>,
        amount: BalanceOf<T>,
        comment: Content
    ) -> DispatchResult {
        let backer = ensure_signed(origin)?;

        let donation_wallet = Self::get_recipient_wallet(recipient.clone())?;
        let donation_id = Self::next_donation_id();

        // TODO check that donations are enabled per this recipient's space.

        // TODO check donated amount against min / max settings if defined.

        // TODO create a comment under the post or a new post in DonationSpace

        let donation = Donation {
            id: donation_id,
            created: WhoAndWhen::<T>::new(backer.clone()),
            recipient: recipient.clone(),
            donation_wallet: donation_wallet.clone(),
            amount,
            comment_id: None // TODO put id of created comment
        };

        // Transfer tokens from the backer to the recipient...
        T::Currency::transfer(&backer, &donation_wallet, amount, ExistenceRequirement::KeepAlive)?;

        DonationById::<T>::insert(donation_id, donation);
        DonationIdsByBacker::<T>::mutate(backer.clone(), |ids| ids.push(donation_id));
        DonationIdsByRecipient::<T>::mutate(recipient.clone(), |ids| ids.push(donation_id));
        NextDonationId::mutate(|n| { *n += 1; });

        Self::deposit_event(RawEvent::Donated(backer, recipient, amount));
        Ok(())
    }

    #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
    pub fn update_donation_wallet(
        origin,
        recipient: DonationRecipient<T::AccountId>,
        maybe_wallet: Option<T::AccountId>
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::ensure_recipient_manager(who, recipient)?;

        // TODO continue...

        // TODO maybe split into two tx:
        // - set_donation_wallet
        // - remove_donation_wallet

        Ok(())
    }

    #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
    pub fn update_settings(
        origin,
        space_id: SpaceId,
        update: DonationSettingsUpdate<T>
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        let has_updates =
            update.donations_enabled.is_some() ||
            update.min_amount.is_some() ||
            update.max_amount.is_some();

        ensure!(has_updates, Error::<T>::NoUpdatesForDonationSettings);

        let space = Spaces::<T>::require_space(space_id)?;

        Spaces::<T>::ensure_account_has_space_permission(
            who.clone(),
            &space,
            SpacePermission::UpdateSpaceSettings,
            Error::<T>::NoUpdatesForDonationSettings.into(),
        )?;

        // `true` if there is at least one updated field.
        let mut should_update = false;

        let settings = Self::donation_settings_by_space(space_id)
            .unwrap_or_else(DonationSettings::default);

        if let Some(donations_enabled) = update.donations_enabled {
            if donations_enabled != settings.donations_enabled {
                settings.donations_enabled = donations_enabled;
                should_update = true;
            }
        }

        if let Some(min_amount) = update.min_amount {
            if min_amount != settings.min_amount {
                settings.min_amount = min_amount;
                should_update = true;
            }
        }

        if let Some(max_amount) = update.max_amount {
            if max_amount != settings.max_amount {
                settings.max_amount = max_amount;
                should_update = true;
            }
        }

        if should_update {
            DonationSettingsBySpace::insert(space_id, settings);
            Self::deposit_event(RawEvent::DonationSettingsUpdated(who, space_id));
        }
        Ok(())
    }
  }
}

impl<T: Trait> Default for DonationSettings<T> {
    fn default() -> Self {
        DonationSettings {
            donations_enabled: true,
            min_amount: None,
            max_amount: None,
        }
    }
}

impl<T: Trait> Module<T> {

    /// Returns an account that should be used as a donation wallet for this recipient.
    pub fn get_recipient_wallet(
        recipient: DonationRecipient<T::AccountId>
    ) -> Result<T::AccountId, DispatchError> {
        let wallet = DonationWalletByRecipient::<T>::get(recipient.clone());
        if let Some(account) = wallet {
            return Ok(account)
        }

        // If a donation wallet is not defined for this recipient:
        match recipient {
            DonationRecipient::Account(account) => {
                Ok(account)
            }
            DonationRecipient::Space(space_id) => {
                let space = Spaces::<T>::require_space(space_id)?;
                let owner = DonationRecipient::Account(space.owner);
                Self::get_recipient_wallet(owner)
            },
            DonationRecipient::Post(post_id) => {
                let post = Posts::<T>::require_post(post_id)?;
                let owner = DonationRecipient::Account(post.owner);
                Self::get_recipient_wallet(owner)
            },
        }
    }

    /// Checks if `maybe_owner` can manage / is owner of a `recipient`.
    pub fn ensure_recipient_manager(
        maybe_owner: T::AccountId,
        recipient: DonationRecipient<T::AccountId>,
    ) -> DispatchResult {
        let is_owner = match recipient {
            DonationRecipient::Account(account) => {
                account == maybe_owner
            },
            DonationRecipient::Space(space_id) => {
                let space = Spaces::<T>::require_space(space_id)?;
                space.is_owner(&maybe_owner)
            },
            DonationRecipient::Post(post_id) => {
                let post = Posts::<T>::require_post(post_id)?;
                post.is_owner(&maybe_owner)
            },
        };
        ensure!(is_owner, Error::<T>::NotRecipientManager);
        Ok(())
    }
}
