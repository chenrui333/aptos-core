// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{transaction_metadata::TransactionMetadata, AptosVM};
use aptos_vm_types::storage::change_set_configs::ChangeSetConfigs;

pub mod epilogue;
pub mod prologue;
pub mod user;

#[derive(Clone, Copy)]
struct Context<'l> {
    vm: &'l AptosVM,
    change_set_configs: &'l ChangeSetConfigs,
    txn_meta: &'l TransactionMetadata,
}
