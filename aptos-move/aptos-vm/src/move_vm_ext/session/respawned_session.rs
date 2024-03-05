// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::StorageAdapter,
    errors::unwrap_or_invariant_violation,
    move_vm_ext::{
        session::view_with_change_set::ExecutorViewWithChangeSet, AptosMoveResolver, SessionExt,
        SessionId,
    },
    AptosVM,
};
use aptos_vm_types::{change_set::VMChangeSet, storage::change_set_configs::ChangeSetConfigs};
use move_core_types::vm_status::VMStatus;

/// FIXME(aldenhu): update documentation
/// We finish the session after the user transaction is done running to get the change set and
/// charge gas and storage fee based on it before running storage refunds and the transaction
/// epilogue. The latter needs to see the state view as if the change set is applied on top of
/// the base state view, and this struct implements that.
#[ouroboros::self_referencing]
pub struct RespawnedSession<'r, 'l> {
    vm: &'l AptosVM,
    executor_view: ExecutorViewWithChangeSet<'r>,
    #[borrows(executor_view)]
    #[covariant]
    resolver: StorageAdapter<'this, ExecutorViewWithChangeSet<'r>>,
    #[borrows(resolver)]
    #[not_covariant]
    pub session: Option<SessionExt<'this, 'l>>,
}

impl<'r, 'l> RespawnedSession<'r, 'l> {
    pub fn spawn(
        vm: &'l AptosVM,
        session_id: SessionId,
        base: &'r impl AptosMoveResolver,
        previous_session_change_set: VMChangeSet,
    ) -> Result<Self, VMStatus> {
        let executor_view = ExecutorViewWithChangeSet::new(
            base.as_executor_view(),
            base.as_resource_group_view(),
            previous_session_change_set,
        );

        Ok(RespawnedSessionBuilder {
            vm,
            executor_view,
            resolver_builder: |executor_view| vm.as_move_resolver(executor_view),
            session_builder: |resolver| Some(vm.new_session(resolver, session_id)),
        }
        .build())
    }

    pub fn execute<T>(
        &mut self,
        fun: impl FnOnce(&mut SessionExt) -> Result<T, VMStatus>,
    ) -> Result<T, VMStatus> {
        self.with_session_mut(|session| {
            fun(unwrap_or_invariant_violation(
                session.as_mut(),
                "VM respawned session has to be set for execution.",
            )?)
        })
    }

    pub fn take_additional_change_set(
        &mut self,
        change_set_configs: &ChangeSetConfigs,
    ) -> Result<VMChangeSet, VMStatus> {
        self.with_session_mut(|session| {
            unwrap_or_invariant_violation(
                session.take(),
                "VM session cannot be finished more than once.",
            )?
            .finish(change_set_configs)
            .map_err(|e| e.into_vm_status())
        })
    }

    pub fn unpack(self) -> (&'l AptosVM, ExecutorViewWithChangeSet<'r>) {
        let heads = self.into_heads();
        (heads.vm, heads.executor_view)
    }
}
