// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use async_graphql::SimpleObject;
use linera_sdk::{
    linera_base_types::{ChainId, MessageId, Owner},
    views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext},
};
use serde::{Deserialize, Serialize};

/// The IDs of a temporary chain for a Lurk Microchain.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, SimpleObject)]
pub struct MicrochainId {
    /// The ID of the `OpenChain` message that created the chain.
    pub message_id: MessageId,
    /// The ID of the temporary game chain itself.
    pub chain_id: ChainId,
}

/// The application state.
#[derive(RootView, SimpleObject)]
#[view(context = "ViewStorageContext")]
pub struct LurkMicrochainState {
    /// The `Owner`s that can interact with this Lurk Microchain.
    pub owners: RegisterView<Option<[Owner; 2]>>,
    /// Temporary chains for individual Lurk Microchains.
    pub chains: MapView<Owner, BTreeSet<MicrochainId>>,
    /// All the proofs currently on chain.
    pub chain_proofs: RegisterView<Vec<u8>>,
    /// The program state.
    pub chain_state: RegisterView<Vec<u8>>,
    /// The zstore state.
    pub zstore_view: RegisterView<Vec<u8>>,
}
