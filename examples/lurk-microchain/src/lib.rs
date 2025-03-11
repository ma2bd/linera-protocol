// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

use async_graphql::{Request, Response};
use linera_sdk::{
    abi::{ContractAbi, ServiceAbi},
    graphql::GraphQLMutationRoot,
    linera_base_types::Owner,
    DataBlobHash,
};
use serde::{Deserialize, Serialize};

pub struct LurkMicrochainAbi;

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    Transition {
        chain_proof: DataBlobHash,
    },
    Start {
        accounts: [Owner; 2],
        chain_state: DataBlobHash,
    },
}

impl ContractAbi for LurkMicrochainAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for LurkMicrochainAbi {
    type Query = Request;
    type QueryResponse = Response;
}
