mod cache;
mod fsm;

pub use cache::*;

pub use http::method::Method;
pub use reqwest::Request;

use env_logger::Builder;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::new().filter_level(LevelFilter::Info).init();

    let request = Request::new(Method::GET, "https://theses.fr/".try_into()?);
    let mut cache = HttpCache::new(request);

    cache.add_interupt_condition(|cache| {
        cache
            .status_code
            .map(|status_code| !status_code.is_success())
            .unwrap_or(false)
    });

    cache.add_interupt_condition(|cache| {
        cache
            .content_type
            .as_ref()
            .map(|content_type| content_type == "application/pdf")
            .unwrap_or(false)
    });

    cache.add_interupt_condition(|cache| cache.request().method() == Method::HEAD);

    println!("{:?}", cache.version().await);
    println!("{:?}", cache.body().await);


    Ok(())
}
