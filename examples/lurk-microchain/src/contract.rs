// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use async_graphql::ComplexObject;
use linera_sdk::{
    abi::WithContractAbi,
    linera_base_types::{
        Amount, ApplicationPermissions, ChainId, ChainOwnership, LurkMicrochainData, Owner,
        TimeoutConfig,
    },
    views::{RootView, View},
    Contract, ContractRuntime, DataBlobHash,
};
use lurk_microchain::{LurkMicrochainAbi, Operation};
use serde::{Deserialize, Serialize};
use state::{LurkMicrochainState, MicrochainId};

pub struct LurkMicrochainContract {
    state: LurkMicrochainState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(LurkMicrochainContract);

impl WithContractAbi for LurkMicrochainContract {
    type Abi = LurkMicrochainAbi;
}

impl Contract for LurkMicrochainContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = LurkMicrochainState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        LurkMicrochainContract { state, runtime }
    }

    async fn instantiate(&mut self, _: ()) {
        log::trace!("Instantiating");
        self.runtime.application_parameters(); // Verifies that these are empty.
    }

    async fn execute_operation(&mut self, operation: Operation) {
        log::trace!("Handling operation {:?}", operation);
        match operation {
            Operation::Transition { chain_proof } => self.execute_transition(chain_proof),
            Operation::Start {
                accounts,
                chain_state,
            } => self.execute_start(accounts, chain_state).await,
        };
    }

    async fn execute_message(&mut self, message: Message) {
        log::trace!("Handling message {:?}", message);
        match message {
            Message::Start {
                accounts: _accounts,
                chain_state,
            } => {
                self.runtime.assert_data_blob_exists(chain_state.clone());
                let chain_state = self.runtime.read_data_blob(chain_state);
                let data = self.runtime.microchain_start(chain_state);
                self.set_data(data);
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl LurkMicrochainContract {
    fn execute_transition(&mut self, chain_proof: DataBlobHash) {
        assert!(self.runtime.chain_id() != self.main_chain_id());
        let data = self
            .runtime
            .microchain_transition(chain_proof, self.get_data());

        self.set_data(data);
    }

    async fn execute_start(&mut self, accounts: [Owner; 2], chain_state: DataBlobHash) {
        assert_eq!(self.runtime.chain_id(), self.main_chain_id());
        let ownership = ChainOwnership::multiple(
            [(accounts[0], 100), (accounts[1], 100)],
            100,
            TimeoutConfig::default(),
        );
        let permissions = ApplicationPermissions::default();
        let (message_id, chain_id) = self
            .runtime
            .open_chain(ownership, permissions, Amount::ZERO);
        for owner in &accounts {
            self.state
                .chains
                .get_mut_or_default(owner)
                .await
                .unwrap()
                .insert(MicrochainId {
                    message_id,
                    chain_id,
                });
        }
        self.runtime.send_message(
            chain_id,
            Message::Start {
                accounts,
                chain_state,
            },
        );
    }

    fn main_chain_id(&mut self) -> ChainId {
        self.runtime.application_creator_chain_id()
    }

    fn get_data(&self) -> LurkMicrochainData {
        LurkMicrochainData {
            chain_proofs: self.state.chain_proofs.get().clone(),
            chain_state: self.state.chain_state.get().clone(),
            zstore_view: self.state.zstore_view.get().clone(),
        }
    }

    fn set_data(&mut self, data: LurkMicrochainData) {
        self.state.chain_proofs.set(data.chain_proofs);
        self.state.chain_state.set(data.chain_state);
        self.state.zstore_view.set(data.zstore_view);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Start {
        accounts: [Owner; 2],
        chain_state: DataBlobHash,
    },
}

/// This implementation is only nonempty in the service.
#[ComplexObject]
impl LurkMicrochainState {}
