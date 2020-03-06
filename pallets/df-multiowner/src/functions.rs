use super::*;

use frame_support::{dispatch::DispatchResult};

impl<T: Trait> Module<T> {

  pub fn vec_remove_on<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
      vector.swap_remove(index);
    }
  }

  pub fn new_updated_at() -> UpdatedAt<T> {
    UpdatedAt {
      block: <system::Module<T>>::block_number(),
      time: <pallet_timestamp::Module<T>>::now(),
    }
  }

  pub fn update_space_owners(executer: T::AccountId, mut space: SpaceOwners<T>, tx: Transaction<T>) -> DispatchResult {
    let space_id = space.space_id;
    let tx_id = tx.id;

    ensure!(tx.confirmed_by.len() >= space.threshold as usize, Error::<T>::NotEnoughConfirms);

    // TODO set new space owners here!

    // TODO add or remove space from list of spaces by account that is added or removed
    // <SpaceIdsOwnedByAccountId<T>>::mutate(owner.clone(), |ids| ids.push(space_id.clone())); // or add or remove space id

    space.pending_tx_count = space.pending_tx_count.checked_sub(1).ok_or(Error::<T>::UnderflowExecutingTx)?;
    space.executed_tx_count = space.executed_tx_count.checked_add(1).ok_or(Error::<T>::OverflowExecutingTx)?;

    Self::change_tx_from_pending_to_executed(space_id, tx_id)?;

    <SpaceOwnersBySpaceById<T>>::insert(space_id, space);
    <TxById<T>>::insert(tx_id, tx);
    Self::deposit_event(RawEvent::SpaceOwnersUpdated(executer, space_id, tx_id));

    Ok(())
  }

  pub fn change_tx_from_pending_to_executed(space_id: SpaceId, tx_id: TransactionId) -> DispatchResult {
    ensure!(Self::space_by_id(space_id).is_some(), Error::<T>::SpaceNotFound);
    ensure!(Self::tx_by_id(tx_id).is_some(), Error::<T>::TxNotFound);
    ensure!(!Self::executed_tx_ids_by_space_id(space_id).iter().any(|&x| x == tx_id), Error::<T>::TxAlreadyExecuted);

    PendingTxIdsBySpaceId::mutate(space_id, |txs| Self::vec_remove_on(txs, tx_id));
    ExecutedTxIdsBySpaceId::mutate(space_id, |ids| ids.push(tx_id));

    Ok(())
  }
}
