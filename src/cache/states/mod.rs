use crate::cache::*;
use crate::fsm::{ResourceState, State, StateHolder, Transition, TransitionTo};
use crate::transitions;

use async_trait::async_trait;
use http::method::Method;
use log::*;
use reqwest::Client;

transitions!(InitState => HeaderState);
transitions!(HeaderState => BodyState*);

impl ResourceState for HttpCacheData {
    type Status = Self;
}

#[async_trait::async_trait]
impl State<HttpCacheData> for InitState {
    async fn next(self: Box<Self>, state: &mut HttpCacheData) -> Transition<HttpCacheData> {
        info!("Transitioning from InitState to ...");

        let mut request = state.request_clone().expect("failed to clone request");
        *request.method_mut() = Method::HEAD;

        let client = Client::new();
        let response = client.execute(request).await.expect("failed to request");

        state.populate(response).await;

        info!("HeaderState");
        Transition::Next(StateHolder {
            state: Box::new(HeaderState),
        })
    }
}

#[async_trait]
impl State<HttpCacheData> for HeaderState {
    async fn next(self: Box<Self>, state: &mut HttpCacheData) -> Transition<HttpCacheData> {
        info!("Transitioning from HeaderState to ...");

        let request = state.request_clone().expect("failed to clone request");
        let client = Client::new();

        let response = client.execute(request).await.expect("failed to request");

        state.populate(response).await;

        info!("Done");
        Transition::Complete(Ok(()))
    }
}
