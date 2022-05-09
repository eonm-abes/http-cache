mod cache;
mod fsm;

pub use cache::*;

pub use http::method::Method;
pub use reqwest::Request;

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_http_cache() {
        let request = Request::new(Method::GET, "https://example.org/".try_into().unwrap());
        let mut cache = HttpCache::new(request);

        assert!(cache.version().await.is_some());
        assert!(!cache.fsm_is_locked());
        assert!(cache.body().await.is_some());
        assert!(cache.fsm_is_locked());
    }

    #[tokio::test]
    async fn test_http_cache_interupt_conditions_met() {
        let request = Request::new(Method::HEAD, "https://example.org/".try_into().unwrap());
        let mut cache = HttpCache::new(request);

        cache.add_interupt_condition(|cache| {
            cache
                .request().method() == Method::HEAD
        });

        assert!(cache.body().await.is_none());
        assert!(cache.fsm_is_locked());
    }

    #[tokio::test]
    async fn test_http_cache_interupt_conditions_not_met() {
        let request = Request::new(Method::GET, "https://example.org/".try_into().unwrap());
        let mut cache = HttpCache::new(request);

        cache.add_interupt_condition(|cache| {
            cache
                .request().method() == Method::HEAD
        });

        assert!(cache.body().await.is_some());
        assert!(cache.fsm_is_locked());
    }
}