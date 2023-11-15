use std::sync::Arc;

use candid::utils::ArgumentEncoder;
use candid::{CandidType, Decode, Principal};
use ic_exports::ic_kit::RejectionCode;
use ic_exports::pocket_ic;
use pocket_ic::{PocketIc, WasmResult};
use serde::de::DeserializeOwned;

use crate::{CanisterClient, CanisterClientError, CanisterClientResult};

/// A client for interacting with a canister inside dfinity's PocketIc test framework.
#[derive(Clone)]
pub struct PocketIcClient {
    client: Arc<PocketIc>,
    pub canister: Principal,
    pub caller: Principal,
}

impl PocketIcClient {
    /// Creates a new instance of a PocketIcClient.
    /// The new instance is independent and have no access to canisters of other instances.
    pub fn new(canister: Principal, caller: Principal) -> Self {
        Self {
            client: Arc::new(pocket_ic::init_pocket_ic()),
            canister,
            caller,
        }
    }

    /// Crates new instance of PocketIcClient from an existing client instance.
    pub fn from_client(client: Arc<PocketIc>, canister: Principal, caller: Principal) -> Self {
        Self {
            client,
            canister,
            caller,
        }
    }

    /// Returns the PocketIC client for the canister.
    pub fn client(&self) -> &Arc<PocketIc> {
        &self.client
    }

    /// Performs a blocking action with PocketIC client and awaits the result.
    ///
    /// Arguments of the closure `f`:
    /// 1) `client` - The PocketIC client.
    /// 2) `canister` - The canister principal.
    /// 3) `caller` - The caller principal.
    pub async fn with_client<F, R>(&self, f: F) -> R
    where
        F: Send + FnOnce(Arc<PocketIc>, Principal, Principal) -> R + 'static,
        R: Send + 'static,
    {
        let client = self.client.clone();
        let cansiter = self.canister;
        let caller = self.caller;

        tokio::task::spawn_blocking(move || f(client, cansiter, caller))
            .await
            .unwrap()
    }

    /// Performs update call with the given arguments.
    pub async fn update<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType,
    {
        let args = candid::encode_args(args)?;
        let method = String::from(method);

        let call_result = self
            .with_client(move |client, canister, caller| {
                client.update_call(canister, caller, &method, args)
            })
            .await?;

        let reply = match call_result {
            WasmResult::Reply(reply) => reply,
            WasmResult::Reject(e) => return Err(reject_error(e)),
        };

        let decoded = Decode!(&reply, R)?;
        Ok(decoded)
    }

    /// Performs query call with the given arguments.
    pub async fn query<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType,
    {
        let args = candid::encode_args(args)?;
        let method = String::from(method);

        let call_result = self
            .with_client(move |env, canister, caller| {
                env.query_call(canister, caller, &method, args)
            })
            .await?;

        let reply = match call_result {
            WasmResult::Reply(reply) => reply,
            WasmResult::Reject(e) => return Err(reject_error(e)),
        };

        let decoded = Decode!(&reply, R)?;
        Ok(decoded)
    }
}

fn reject_error(e: String) -> CanisterClientError {
    CanisterClientError::CanisterError((RejectionCode::CanisterError, e))
}

#[async_trait::async_trait]
impl CanisterClient for PocketIcClient {
    async fn update<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType,
    {
        PocketIcClient::update(self, method, args).await
    }

    async fn query<T, R>(&self, method: &str, args: T) -> CanisterClientResult<R>
    where
        T: ArgumentEncoder + Send + Sync,
        R: DeserializeOwned + CandidType,
    {
        PocketIcClient::query(self, method, args).await
    }
}
