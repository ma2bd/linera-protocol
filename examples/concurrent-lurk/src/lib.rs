// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::SimpleObject;
use linera_sdk::{
    abi::{ContractAbi, ServiceAbi},
    graphql::GraphQLMutationRoot,
    linera_base_types::{ChainId, MessageId, AccountOwner},
    DataBlobHash,
};
use serde::{Deserialize, Serialize};

pub struct ConcurrentLurkAbi;

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    Transition {
        chain_proof: DataBlobHash,
    },
    Start {
        owner: AccountOwner,
        chain_state: DataBlobHash,
    },
}

/// The IDs of a temporary chain for a Lurk process.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, SimpleObject)]
pub struct ProcessId {
    /// The ID of the `OpenChain` message that created the chain.
    pub message_id: MessageId,
    /// The ID of the temporary game chain itself.
    pub chain_id: ChainId,
}

impl ProcessId {
    pub fn new(message_id: MessageId, chain_id: ChainId) -> Self {
        Self {
            message_id,
            chain_id,
        }
    }
}

impl ContractAbi for ConcurrentLurkAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for ConcurrentLurkAbi {
    type Query = async_graphql::Request;
    type QueryResponse = async_graphql::Response;
}
