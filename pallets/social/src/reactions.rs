use super::*;

impl<T: Trait> Module<T> {

    // FIXME: don't add reaction in storage before the checks in 'create_reaction' are done
    pub fn insert_new_reaction(account: T::AccountId, kind: ReactionKind) -> ReactionId {
        let id = Self::next_reaction_id();
        let reaction: Reaction<T> = Reaction {
            id,
            created: WhoAndWhen::<T>::new(account),
            updated: None,
            kind
        };

        <ReactionById<T>>::insert(id, reaction);
        NextReactionId::mutate(|n| { *n += 1; });

        id
    }
}
