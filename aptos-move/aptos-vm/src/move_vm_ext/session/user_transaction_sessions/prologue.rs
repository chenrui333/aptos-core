// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    move_vm_ext::{
        session::{
            respawnable_session::RespawnableSession,
            user_transaction_sessions::{epilogue::EpilogueSession, user::UserSession, Context},
        },
        AptosMoveResolver, SessionId,
    },
    transaction_metadata::TransactionMetadata,
    AptosVM,
};
use aptos_gas_algebra::Fee;
use aptos_vm_types::storage::change_set_configs::ChangeSetConfigs;
use derive_more::{Deref, DerefMut};
use move_core_types::vm_status::VMStatus;

#[derive(Deref, DerefMut)]
pub struct PrologueSession<'r, 'l> {
    context: Context<'l>,
    #[deref]
    #[deref_mut]
    session: RespawnableSession<'r, 'l>,
}

impl<'r, 'l> PrologueSession<'r, 'l> {
    pub fn new(
        vm: &'l AptosVM,
        change_set_configs: &'l ChangeSetConfigs,
        txn_meta: &TransactionMetadata,
        resolver: &'r impl AptosMoveResolver,
    ) -> Self {
        let context = Context {
            vm,
            change_set_configs,
            txn_meta,
        };
        let session_id = SessionId::prologue_meta(context.txn_meta);
        let session = RespawnableSession::new_session(context.vm, session_id, resolver);

        Self { context, session }
    }

    ///
    pub fn into_sessions(
        self,
        gas_feature_version: u64,
    ) -> Result<(EpilogueSession<'r, 'l>, UserSession<'r, 'l>), VMStatus> {
        let Self {
            context,
            session: mut prologue_session,
        } = self;

        let epilogue_session_id = SessionId::epilogue_meta(context.txn_meta);

        let (e, u) = if gas_feature_version >= 1 {
            // Create a new session so that the data cache is flushed.
            // This is to ensure we correctly charge for loading certain resources, even if they
            // have been previously cached in the prologue.
            //
            // TODO(Gas): Do this in a better way in the future, perhaps without forcing the data cache to be flushed.
            // By releasing resource group cache, we start with a fresh slate for resource group
            // cost accounting.

            let change_set = prologue_session.take_additional_change_set()?;
            let executor_view = prologue_session.unpack();

            (
                EpilogueSession::new(
                    RespawnableSession::with_view(
                        context.vm,
                        epilogue_session_id,
                        executor_view.with_change_set(change_set.clone()),
                    ),
                    Fee::zero(),
                ),
                UserSession::new(
                    context,
                    RespawnableSession::with_view(
                        context.vm,
                        SessionId::txn_meta(context.txn_meta),
                        executor_view.with_change_set(change_set),
                    ),
                ),
            )
        } else {
            (
                EpilogueSession::new(
                    RespawnableSession::with_view(
                        context.vm,
                        epilogue_session_id,
                        prologue_session
                            .borrow_executor_view()
                            .assert_no_change_set()
                            .clone(),
                    ),
                    Fee::zero(),
                ),
                UserSession::new(context, prologue_session),
            )
        };
        Ok((e, u))
    }
}
