// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::StorageAdapter,
    errors::unwrap_or_invariant_violation,
    move_vm_ext::{
        session::view_with_change_set::ExecutorViewWithChangeSet, AptosMoveResolver,
        AsExecutorView, AsResourceGroupView, SessionExt, SessionId,
    },
    AptosVM,
};
use aptos_vm_types::{change_set::VMChangeSet, resolver::TResourceGroupView};
use move_core_types::vm_status::VMStatus;

/// FIXME(aldenhu): update documentation
#[ouroboros::self_referencing]
pub struct RespawnableSession<'r, 'l> {
    pub executor_view: ExecutorViewWithChangeSet<'r>,
    #[borrows(executor_view)]
    #[covariant]
    resolver: StorageAdapter<'this, ExecutorViewWithChangeSet<'r>>,
    #[borrows(resolver)]
    #[not_covariant]
    /// This has to be an option because session needs to finish() (which consumes itself) before
    /// RespawnedSession destructs.
    session: Option<SessionExt<'this, 'l>>,
}

impl<'r, 'l> RespawnableSession<'r, 'l> {
    pub fn new_session(
        vm: &'l AptosVM,
        session_id: SessionId,
        base: &'r impl AptosMoveResolver,
    ) -> Self {
        let executor_view = ExecutorViewWithChangeSet::new(
            base.as_executor_view(),
            base.as_resource_group_view(),
            None,
        );

        Self::build(vm, executor_view, session_id)
    }

    pub fn with_view(
        vm: &'l AptosVM,
        session_id: SessionId,
        view: ExecutorViewWithChangeSet<'r>,
    ) -> Self {
        Self::build(vm, view, session_id)
    }

    pub fn clear_cache_and_respawn(
        mut self,
        vm: &'l AptosVM,
        session_id: SessionId,
    ) -> Result<Self, VMStatus> {
        let change_set = self.take_additional_change_set().unwrap();
        let mut executor_view = self.unpack();
        executor_view.set_change_set(change_set.clone())?;
        executor_view.release_group_cache();

        Ok(Self::build(vm, executor_view, session_id))
    }

    fn build(
        vm: &'l AptosVM,
        executor_view: ExecutorViewWithChangeSet<'r>,
        session_id: SessionId,
    ) -> Self {
        RespawnableSessionBuilder {
            executor_view,
            resolver_builder: |executor_view| vm.as_move_resolver(executor_view),
            session_builder: |resolver| Some(vm.new_session(resolver, session_id)),
        }
        .build()
    }

    pub fn execute<T, E: Into<VMStatus>>(
        &mut self,
        fun: impl FnOnce(&mut SessionExt) -> Result<T, E>,
    ) -> Result<T, E> {
        self.with_session_mut(|session| {
            fun(unwrap_or_invariant_violation(
                session.as_mut(),
                "VM respawned session has to be set for execution.",
            )?)
        })
    }

    pub fn take_additional_change_set(&mut self) -> Result<VMChangeSet, VMStatus> {
        self.with_session_mut(|session| {
            unwrap_or_invariant_violation(
                session.take(),
                "VM session cannot be finished more than once.",
            )?
            .finish(self.borrow_change_set_configs())
            .map_err(|e| e.into_vm_status())
        })
    }

    pub fn unpack(self) -> ExecutorViewWithChangeSet<'r> {
        self.into_heads().executor_view
    }
}
