// Copyright (c) Lurk Lab Systems Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Request, Response, Schema};
use concurrent_lurk::Operation;
use linera_sdk::{
    abi::WithServiceAbi, graphql::GraphQLMutationRoot, views::View, Service, ServiceRuntime,
};

use self::state::ConcurrentLurkState;

#[derive(Clone)]
pub struct ConcurrentLurkService {
    runtime: Arc<ServiceRuntime<ConcurrentLurkService>>,
    state: Arc<ConcurrentLurkState>,
}

linera_sdk::service!(ConcurrentLurkService);

impl WithServiceAbi for ConcurrentLurkService {
    type Abi = concurrent_lurk::ConcurrentLurkAbi;
}

impl Service for ConcurrentLurkService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = ConcurrentLurkState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        ConcurrentLurkService {
            runtime: Arc::new(runtime),
            state: Arc::new(state),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            self.state.clone(),
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::{futures_util::FutureExt, Request};
    use linera_sdk::{util::BlockingWait, views::View, Service, ServiceRuntime};
    use serde_json::json;

    use super::*;

    #[test]
    fn query() {
        let runtime = ServiceRuntime::<ConcurrentLurkService>::new();
        let state = ConcurrentLurkState::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to read from mock key value store");

        let service = ConcurrentLurkService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        };

        let response = service
            .handle_query(Request::new("{ clock { increment } }"))
            .now_or_never()
            .expect("Query should not await anything")
            .data
            .into_json()
            .expect("Response should be JSON");

        assert_eq!(response, json!({"clock" : {"increment": 0}}))
    }
}
