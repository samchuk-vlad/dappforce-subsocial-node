use super::*;

use sp_std::collections::btree_set::BTreeSet;
use frame_support::{dispatch::DispatchResult};

impl<T: Trait> Module<T> {

  pub fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
      vector.swap_remove(index);
    }
  }

  pub fn new_whoandwhen(account: T::AccountId) -> WhoAndWhen<T> {
    WhoAndWhen {
      account,
      block: <system::Module<T>>::block_number(),
      time: <pallet_timestamp::Module<T>>::now(),
    }
  }

  pub fn update_space_owners(who: T::AccountId, mut space_owners: SpaceOwners<T>, tx: Transaction<T>) -> DispatchResult {
    let space_id = space_owners.space_id;
    let tx_id = tx.id;

    ensure!(tx.confirmed_by.len() >= space_owners.threshold as usize, Error::<T>::NotEnoughConfirms);
    Self::change_tx_from_pending_to_executed(space_id, tx_id)?;

    space_owners.changes_count = space_owners.changes_count.checked_add(1).ok_or(Error::<T>::OverflowExecutingTx)?;
    if !tx.add_owners.is_empty() || !tx.remove_owners.is_empty() {
      space_owners.owners = Self::transform_new_owners_to_vec(space_owners.owners.clone(), tx.add_owners.clone(), tx.remove_owners.clone());
    }

    if let Some(threshold) = tx.new_threshold {
      space_owners.threshold = threshold;
    }

    for account in &tx.add_owners {
      <SpaceIdsOwnedByAccountId<T>>::mutate(account, |ids| ids.insert(space_id));
    }
    for account in &tx.remove_owners {
      <SpaceIdsOwnedByAccountId<T>>::mutate(account, |ids| ids.remove(&space_id));
    }

    <SpaceOwnersBySpaceById<T>>::insert(space_id, space_owners);
    <TxById<T>>::insert(tx_id, tx);
    Self::deposit_event(RawEvent::SpaceOwnersUpdated(who, space_id, tx_id));

    Ok(())
  }

  pub fn change_tx_from_pending_to_executed(space_id: SpaceId, tx_id: TransactionId) -> DispatchResult {
    ensure!(Self::space_owners_by_space_id(space_id).is_some(), Error::<T>::SpaceOwnersNotFound);
    ensure!(Self::tx_by_id(tx_id).is_some(), Error::<T>::TxNotFound);
    ensure!(!Self::executed_tx_ids_by_space_id(space_id).iter().any(|&x| x == tx_id), Error::<T>::TxAlreadyExecuted);

    PendingTxIdBySpaceId::remove(&space_id);
    PendingTxIds::mutate(|set| set.remove(&tx_id));
    ExecutedTxIdsBySpaceId::mutate(space_id, |ids| ids.push(tx_id));

    Ok(())
  }

  pub fn transform_new_owners_to_vec(current_owners: Vec<T::AccountId>, add_owners: Vec<T::AccountId>, remove_owners: Vec<T::AccountId>) -> Vec<T::AccountId> {
    let mut owners_set: BTreeSet<T::AccountId> = BTreeSet::new();
    let mut new_owners_set: BTreeSet<T::AccountId> = BTreeSet::new();

    // Extract current space owners
    current_owners.iter().for_each(|x| { owners_set.insert(x.clone()); });
    // Extract owners that should be added
    add_owners.iter().for_each(|x| { new_owners_set.insert(x.clone()); });
    // Unite both sets
    owners_set = owners_set.union(&new_owners_set).cloned().collect();
    // Remove accounts that exist in remove_owners from set
    remove_owners.iter().for_each(|x| { owners_set.remove(x); });

    owners_set.iter().cloned().collect()
  }

  pub fn clean_pending_txs(block_number: T::BlockNumber) {
    if (block_number % T::CleanExpiredTxsPeriod::get()).is_zero() {
      for tx_id in Self::pending_tx_ids() {
        if let Some(tx) = Self::tx_by_id(tx_id) {
          if block_number >= tx.expires_at {
            PendingTxIdBySpaceId::remove(&tx.space_id);
            <TxById<T>>::remove(&tx_id);
            PendingTxIds::mutate(|set| set.remove(&tx_id));
          }
        }
      }
    }
  }
}
