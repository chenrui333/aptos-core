// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::session::respawned_session::RespawnedSession;
use aptos_gas_algebra::Fee;
use aptos_vm_types::{change_set::VMChangeSet, storage::change_set_configs::ChangeSetConfigs};
use move_core_types::vm_status::{err_msg, StatusCode, VMStatus};
use std::ops::{Deref, DerefMut};

pub struct EpilogueSession<'r, 'l> {
    session: RespawnedSession<'r, 'l>,
    storage_refund: Fee,
}

impl<'r, 'l> Deref for EpilogueSession<'r, 'l> {
    type Target = RespawnedSession<'r, 'l>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl<'r, 'l> DerefMut for EpilogueSession<'r, 'l> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}

impl<'r, 'l> EpilogueSession<'r, 'l> {
    pub fn new(session: RespawnedSession<'r, 'l>, storage_refund: Fee) -> Self {
        Self {
            session,
            storage_refund,
        }
    }

    pub fn finish(self, change_set_configs: &ChangeSetConfigs) -> Result<VMChangeSet, VMStatus> {
        let mut session = self.session;
        let additional_change_set = session.take_additional_change_set(change_set_configs)?;

        if additional_change_set.has_creation() {
            // In the epilogue there shouldn't be new slots created, otherwise there's a potential
            // vulnerability like this:
            // 1. slot created by the user
            // 2. another user transaction deletes the slot and claims the refund
            // 3. in the epilogue the same slot gets recreated, and the final write set will have
            //    a ModifyWithMetadata carrying the original metadata
            // 4. user keeps doing the same and repeatedly claim refund out of the slot.
            return Err(VMStatus::error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                err_msg("Unexpected storage allocation after respawning session."),
            ));
        }

        let (_vm, executor_view) = session.unpack();
        let mut change_set = executor_view.change_set;
        change_set
            .squash_additional_change_set(additional_change_set, change_set_configs)
            .map_err(|_err| {
                VMStatus::error(
                    StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                    err_msg("Failed to squash VMChangeSet"),
                )
            })?;

        Ok(change_set)
    }

    pub fn get_storage_fee_refund(&self) -> Fee {
        self.storage_refund
    }
}
