#![allow(unused)]
use ioevent::rpc::*;

use crate::{OneBotGenericId, api_client::OneBotApiClient, config::OneBotConfig};

#[derive(Clone)]
pub struct OneBotAdapterState {
    pub api_client: OneBotApiClient,
    pub pdw: DefaultProcedureWright,
    pub base_generic_id: OneBotGenericId,
}
impl ProcedureCallWright for OneBotAdapterState {
    fn next_echo(&self) -> impl Future<Output = u64> + Send + Sync {
        self.pdw.next_echo()
    }
}
impl OneBotAdapterState {
    pub async fn new() -> Self {
        let config = OneBotConfig::load().unwrap();
        let generic_id = OneBotGenericId::from_config(&config);
        let ws_api = crate::join_url(&config.ws_url, "/api");
        Self {
            api_client: OneBotApiClient::new(&ws_api).await.unwrap(),
            pdw: DefaultProcedureWright::default(),
            base_generic_id: generic_id,
        }
    }
}
