// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Conversions from types declared in [`linera-sdk`] to types generated by [`wit-bindgen-guest-rust`].

use super::queryable_system as system;
use linera_base::{
    crypto::CryptoHash,
    identifiers::{ApplicationId, EffectId},
};
use std::task::Poll;

impl From<log::Level> for system::LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Trace => system::LogLevel::Trace,
            log::Level::Debug => system::LogLevel::Debug,
            log::Level::Info => system::LogLevel::Info,
            log::Level::Warn => system::LogLevel::Warn,
            log::Level::Error => system::LogLevel::Error,
        }
    }
}

impl From<CryptoHash> for system::CryptoHash {
    fn from(hash_value: CryptoHash) -> Self {
        let parts = <[u64; 4]>::from(hash_value);

        system::CryptoHash {
            part1: parts[0],
            part2: parts[1],
            part3: parts[2],
            part4: parts[3],
        }
    }
}

impl From<CryptoHash> for super::CryptoHash {
    fn from(hash_value: CryptoHash) -> Self {
        let parts = <[u64; 4]>::from(hash_value);

        super::CryptoHash {
            part1: parts[0],
            part2: parts[1],
            part3: parts[2],
            part4: parts[3],
        }
    }
}

impl From<Poll<Result<Vec<u8>, String>>> for super::PollQuery {
    fn from(poll: Poll<Result<Vec<u8>, String>>) -> Self {
        use super::PollQuery;
        match poll {
            Poll::Pending => PollQuery::Pending,
            Poll::Ready(Ok(response)) => PollQuery::Ready(Ok(response)),
            Poll::Ready(Err(message)) => PollQuery::Ready(Err(message)),
        }
    }
}

impl From<ApplicationId> for system::ApplicationId {
    fn from(application_id: ApplicationId) -> system::ApplicationId {
        system::ApplicationId {
            bytecode_id: application_id.bytecode_id.0.into(),
            creation: application_id.creation.into(),
        }
    }
}

impl From<EffectId> for system::EffectId {
    fn from(effect_id: EffectId) -> Self {
        system::EffectId {
            chain_id: effect_id.chain_id.0.into(),
            height: effect_id.height.0,
            index: effect_id.index,
        }
    }
}
