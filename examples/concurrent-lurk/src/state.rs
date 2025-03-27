// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::SimpleObject;
use concurrent_lurk::ProcessId;
use linera_sdk::{
    linera_base_types::AccountOwner,
    views::{linera_views, QueueView, RegisterView, RootView, ViewStorageContext},
};

/// The application state.
#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct ConcurrentLurkState {
    /// State of the concurrent lurk application.
    pub dummy: RegisterView<bool>,
    /// Owner of this Lurk process.
    pub owner: RegisterView<Option<AccountOwner>>,
    /// All the proofs currently on chain.
    pub chain_proofs: RegisterView<Vec<u8>>,
    /// The program state.
    pub chain_state: RegisterView<Vec<u8>>,
    /// The zstore state.
    pub zstore_view: RegisterView<Vec<u8>>,

    /// A queue of application level messages as serialized z-ptrs. They are of the form: `(:send to message)`.
    pub message_queue: QueueView<Vec<u8>>,

    /// All children of this Lurk process, in the order they were spawned.
    pub children: RegisterView<Vec<ProcessId>>,
    /// This is the most recently spawned process.
    /// We need to track it because after spawning, the user needs to transition on this PID.
    pub ready: RegisterView<Option<ProcessId>>,
}
