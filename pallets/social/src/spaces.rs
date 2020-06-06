use super::*;

use frame_support::dispatch::DispatchResult;

impl<T: Trait> Space<T> {

    pub fn new(
        id: SpaceId,
        created_by: T::AccountId,
        ipfs_hash: Vec<u8>,
        handle: Option<Vec<u8>>
    ) -> Self {
        Space {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            hidden: false,
            owner: created_by,
            handle,
            ipfs_hash,
            posts_count: 0,
            followers_count: 0,
            edit_history: Vec::new(),
            score: 0,
            permissions: None
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn increment_posts_count(&mut self) -> DispatchResult {
        self.posts_count = self.posts_count.checked_add(1).ok_or(Error::<T>::OverflowAddingPostOnSpace)?;
        Ok(())
    }

    pub fn ensure_space_owner(&self, who: T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(&who), Error::<T>::NotASpaceOwner);
        Ok(())
    }

    pub fn ensure_space_exists(space_id: SpaceId) -> DispatchResult {
        ensure!(<SpaceById<T>>::exists(space_id), Error::<T>::SpaceNotFound);
        Ok(())
    }
}

impl<T: Trait> Module<T> {

    pub fn get_or_new_social_account(account: T::AccountId) -> SocialAccount<T> {
        if let Some(social_account) = Self::social_account_by_id(account) {
            social_account
        } else {
            SocialAccount {
                followers_count: 0,
                following_accounts_count: 0,
                following_spaces_count: 0,
                reputation: 1,
                profile: None
            }
        }
    }

    // TODO replace with code from Sefl::lowercase_and_validate_a_handle()
    pub fn is_username_valid(username: Vec<u8>) -> DispatchResult {
        ensure!(Self::account_by_profile_username(username.clone()).is_none(), Error::<T>::UsernameIsBusy);
        ensure!(username.len() >= T::MinUsernameLen::get() as usize, Error::<T>::UsernameIsTooShort);
        ensure!(username.len() <= T::MaxUsernameLen::get() as usize, Error::<T>::UsernameIsTooLong);
        ensure!(username.iter().all(|&x| Self::is_valid_handle_char(x)), Error::<T>::UsernameIsNotAlphanumeric);

        Ok(())
    }

    /// An example of a valid handle: `good_handle`.
    fn is_valid_handle_char(c: u8) -> bool {
        match c {
            b'0'..=b'9' | b'a'..=b'z' | b'_' => true,
            _ => false,
        }
    }

    pub fn lowercase_and_validate_a_handle(mut handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        handle = handle.to_ascii_lowercase();

        ensure!(Self::space_id_by_handle(handle.clone()).is_none(), Error::<T>::HandleIsNotUnique);

        ensure!(handle.len() >= T::MinHandleLen::get() as usize, Error::<T>::HandleIsTooShort);
        ensure!(handle.len() <= T::MaxHandleLen::get() as usize, Error::<T>::HandleIsTooLong);

        ensure!(handle.iter().all(|&x| Self::is_valid_handle_char(x)), Error::<T>::HandleContainsInvalidChars);

        Ok(handle)
    }
}
