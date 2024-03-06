// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::session::{
    respawnable_session::RespawnableSession, user_transaction_sessions::Context,
};
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct UserSession<'r, 'l> {
    pub context: Context<'l>,
    /// This carries the prologue change set.
    #[deref]
    #[deref_mut]
    pub session: RespawnableSession<'r, 'l>,
}

impl<'r, 'l> UserSession<'r, 'l> {
    pub fn new(context: Context<'l>, session: RespawnableSession<'r, 'l>) -> Self {
        Self { context, session }
    }
}
