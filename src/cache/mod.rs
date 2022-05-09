mod states;
use states::*;

use crate::fsm::{State, Transition};
use log::*;

use async_recursion::async_recursion;
use http::{HeaderMap, StatusCode, Version};
use reqwest::Request;

#[derive(Debug)]
/// A struct that holds the data of a request.
pub struct HttpCacheData {
    request: Request,
    pub version: Option<Version>,
    pub headers: Option<HeaderMap>,
    pub status_code: Option<StatusCode>,
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
    pub body: Option<String>,
}

impl HttpCacheData {
    /// Creates a new cache.
    pub fn new(request: Request) -> Self {
        Self {
            request,
            version: None,
            headers: None,
            status_code: None,
            content_length: None,
            content_type: None,
            body: None,
        }
    }

    /// Try to get a owned Request.
    pub fn request_clone(&self) -> Option<Request> {
        self.request.try_clone()
    }

    /// Gets the request.
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Populates the cache with the content of a HTTP response.
    async fn populate(&mut self, response: reqwest::Response) {
        self.status_code = Some(response.status());
        self.content_length = response.content_length();
        self.headers = Some(response.headers().clone());
        self.version = Some(response.version());

        if let Some(header) = self.headers.as_ref() {
            self.content_type = header
                .get(http::header::CONTENT_TYPE)
                .map(|s| s.to_str().unwrap().to_string());

            self.content_length = header
                .get(http::header::CONTENT_LENGTH)
                .map(|s| s.to_str().unwrap().parse::<u64>().unwrap());
        };

        if let Ok(body) = response.text().await {
            if !body.is_empty() {
                self.body = Some(body)
            }
        }
    }
}

/// A macro used to define getters for the fields of the cache.
macro_rules! fsm_getter {
    ($(#[$meta:meta])* $name:ident, $type:ty) => {
        $(#[$meta])*
        #[async_recursion]
        pub async fn $name(&mut self) -> Option<$type> {
            if self.fsm_is_locked() {
                self.data.$name.clone()
            } else {
                match &self.data.$name {
                    Some(value) => {
                        Some(value.clone())
                    }
                    None => {
                        self.next().await;
                        self.$name().await
                    }
                }
            }
        }
    };
}

/// A state machine that can be used to cache HTTP responses and to fetch them later.
pub struct HttpCache {
    fsm: Box<dyn State<HttpCacheData>>,
    fsm_locked: bool,
    interupt_conditions: Vec<fn(&HttpCacheData) -> bool>,
    data: HttpCacheData,
}

impl HttpCache {
    /// Creates a new HTTP cache.
    pub fn new(request: Request) -> Self {
        Self {
            fsm: Box::new(InitState),
            fsm_locked: false,
            interupt_conditions: Vec::new(),
            data: HttpCacheData::new(request),
        }
    }

    /// Adds an interupt condition. The condition is a function that returns true if the caching process should be interrupted.
    /// The FSM stops if any of the conditions is met.
    pub fn add_interupt_condition(&mut self, f: fn(&HttpCacheData) -> bool) {
        self.interupt_conditions.push(f);
    }

    /// Locks the state machine. The FSM is locked when the FSM has finished its work or when an interupt condition is met.
    fn lock_fsm(&mut self) {
        self.fsm_locked = true;
    }

    /// Checks if the FSM is locked.
    pub fn fsm_is_locked(&self) -> bool {
        self.fsm_locked
    }

    /// Transitions the FSM to the next state.
    async fn next(&mut self) {
        if self.fsm_is_locked() {
            return;
        }

        match self.fsm.clone().next(&mut self.data).await {
            Transition::Next(state) => {
                self.fsm = state.state;

                if self.interupt_conditions.iter().any(|f| f(&self.data)) {
                    info!("Interupted by condition");
                    self.lock_fsm();
                }
            }
            Transition::Complete(_t) => {
                self.lock_fsm();
            }
        };
    }

    fsm_getter!(
        /// Gets the status code of the response.
        status_code,
        StatusCode
    );

    fsm_getter!(
        /// Gets the headers of the response.
        headers,
        http::HeaderMap
    );

    fsm_getter!(
        /// Gets the body of the response.
        body,
        String
    );

    fsm_getter!(
        /// Gets the content length of the response.
        content_length,
        u64
    );

    fsm_getter!(
        /// Gets the content type of the response.
        content_type,
        String
    );

    fsm_getter!(
        /// Gets the HTTP version of the response.
        version,
        http::Version
    );
}
