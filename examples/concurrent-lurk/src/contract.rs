// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use async_graphql::ComplexObject;
use concurrent_lurk::{ConcurrentLurkAbi, Operation, ProcessId};
use linera_sdk::{
    abi::WithContractAbi,
    linera_base_types::{
        Amount, ApplicationPermissions, ChainOwnership, LurkMicrochainData, AccountOwner, PostprocessData,
        PreprocessData,
    },
    views::{RootView, View},
    Contract, ContractRuntime, DataBlobHash,
};
use serde::{Deserialize, Serialize};
use state::ConcurrentLurkState;

pub struct ConcurrentLurkContract {
    state: ConcurrentLurkState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(ConcurrentLurkContract);

impl WithContractAbi for ConcurrentLurkContract {
    type Abi = ConcurrentLurkAbi;
}

impl Contract for ConcurrentLurkContract {
    type Message = Message;
    type InstantiationArgument = ();
    type Parameters = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = ConcurrentLurkState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state.");
        ConcurrentLurkContract { state, runtime }
    }

    async fn instantiate(&mut self, _: ()) {
        log::trace!("Instantiating...");
        self.runtime.application_parameters(); // Verifies that these are empty.
    }

    async fn execute_operation(&mut self, operation: Operation) {
        match operation {
            Operation::Transition { chain_proof } => self.execute_transition(chain_proof).await,
            Operation::Start { owner, chain_state } => self.execute_start(owner, chain_state).await,
        };
    }

    async fn execute_message(&mut self, message: Message) {
        match message {
            Message::Start => {
                log::info!("Starting this application...");
                self.state.dummy.set(true);
            }
            Message::Message(message) => {
                log::info!("Handling message.");
                self.state.message_queue.push_back(message)
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state.");
    }
}

impl ConcurrentLurkContract {
    async fn execute_start(&mut self, owner: AccountOwner, chain_state: DataBlobHash) {
        self.state.owner.set(Some(owner));

        log::info!(">>> START assert_data_blob_exists");
        self.runtime.assert_data_blob_exists(chain_state.clone());
        let chain_state = self.runtime.read_data_blob(chain_state);
        log::info!(">>> END assert_data_blob_exists");

        log::info!(">>> START microchain_start");
        let new_data = self.runtime.microchain_start(chain_state);
        self.set_data(new_data);
        log::info!(">>> END microchain_start");

        log::info!(">>> START postprocess_microchain_transition");
        let postprocess_data = self
            .runtime
            .postprocess_microchain_transition(self.get_data());
        self.postprocess_microchain_transition(postprocess_data);
        log::info!(">>> END postprocess_microchain_transition");
    }

    async fn execute_transition(&mut self, chain_proof: DataBlobHash) {
        let data = self.get_data();

        log::info!(">>> START preprocess_microchain_transition");
        let preprocess_data = self
            .runtime
            .preprocess_microchain_transition(chain_proof, data.clone());
        self.preprocess_microchain_transition(preprocess_data).await;
        log::info!(">>> END preprocess_microchain_transition");

        log::info!(">>> START microchain_transition");
        let new_data = self.runtime.microchain_transition(chain_proof, data);
        self.set_data(new_data);
        log::info!(">>> END microchain_transition");

        log::info!(">>> START postprocess_microchain_transition");
        let postprocess_data = self
            .runtime
            .postprocess_microchain_transition(self.get_data());
        self.postprocess_microchain_transition(postprocess_data);
        log::info!(">>> END postprocess_microchain_transition");
    }

    async fn preprocess_microchain_transition(&mut self, preprocess_data: PreprocessData) {
        match preprocess_data {
            PreprocessData::Spawn { pid } => {
                let ready = self.state.ready.get().clone().unwrap();
                assert_eq!(ready.chain_id, pid, "Incorrect spawn PID.");
                self.state.ready.set(None);
                self.state.children.get_mut().push(ready);
            }
            PreprocessData::Receive { message } => {
                let expected = self.state.message_queue.front().await.unwrap().unwrap();
                assert_eq!(message, expected, "Got a different message than expected.");
                self.state.message_queue.delete_front();
            }
            // If `:send` or just a normal result, do nothing.
            PreprocessData::Send => (),
            PreprocessData::None => (),
        }
    }

    fn postprocess_microchain_transition(&mut self, postprocess_data: PostprocessData) {
        match postprocess_data {
            PostprocessData::Spawn { .. } => {
                let owner = self.state.owner.get().unwrap();
                let ownership = ChainOwnership::single_super(owner);
                let permissions = ApplicationPermissions::default();
                let (message_id, chain_id) =
                    self.runtime
                        .open_chain(ownership, permissions, Amount::ZERO);
                
                assert!(self.state.ready.get().is_none());
                self.state
                    .ready
                    .set(Some(ProcessId::new(message_id, chain_id)));
                
                self.runtime.send_message(chain_id, Message::Start);
            }
            PostprocessData::Send { other_pid, message } => {
                self.runtime
                    .send_message(other_pid, Message::Message(message));
            }
            // If `:receive` or just a normal result, do nothing.
            PostprocessData::Receive => (),
            PostprocessData::None => (),
        }
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
    Start,
    Message(Vec<u8>),
}

/// This implementation is only nonempty in the service.
#[ComplexObject]
impl ConcurrentLurkState {}
