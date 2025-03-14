use http::header::USER_AGENT;
use http::header::AUTHORIZATION;
use http::Uri;
use hyper_rustls::HttpsConnectorBuilder;

use hyper_http_proxy::{Proxy, ProxyConnector, Intercept};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

use octocrab::service::middleware::base_uri::BaseUriLayer;
use octocrab::service::middleware::extra_headers::ExtraHeadersLayer;
use octocrab::{AuthState, OctocrabBuilder};
use std::sync::Arc;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let mut proxy_str = match std::env::var("https_proxy") {
        Ok(val) => val,
        Err(_) => String::from(""),
    };
    proxy_str = match std::env::var("HTTPS_PROXY") {
        Ok(val) => val,
        Err(_) => proxy_str,
    };

    let octocrab = match proxy_str.as_str() {
        "" =>
            OctocrabBuilder::new_empty()
                .with_service(Client::builder(TokioExecutor::new()).build(
                    HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .unwrap()
                        .https_only()
                        .enable_http1()
                        .build()))
            .with_layer(&BaseUriLayer::new(Uri::from_static(
                "https://api.github.com",
            )))
            .with_layer(&ExtraHeadersLayer::new(Arc::new(vec![
                (USER_AGENT, "octocrab".parse().unwrap()),
                (AUTHORIZATION, format!("Bearer {}", "<my_token>").parse().unwrap()),
            ])))
            .with_auth(AuthState::None)
            .build()
            .unwrap()
        ,
        _ =>
            OctocrabBuilder::new_empty()
                .with_service(Client::builder(TokioExecutor::new()).build(
                                   ProxyConnector::from_proxy(HttpConnector::new(),
                                          Proxy::new(Intercept::All,
                                                     proxy_str.parse().unwrap()))
                                          .unwrap()))
            .with_layer(&BaseUriLayer::new(Uri::from_static(
                "https://api.github.com",
            )))
            .with_layer(&ExtraHeadersLayer::new(Arc::new(vec![
                (USER_AGENT, "octocrab".parse().unwrap()),
                (AUTHORIZATION, format!("Bearer {}", "<my_token>").parse().unwrap()),
            ])))
            .with_auth(AuthState::None)
            .build()
            .unwrap()
        ,
    };

    let repo = octocrab.repos("rust-lang", "rust").get().await?;

    let repo_metrics = octocrab
        .repos("rust-lang", "rust")
        .get_community_profile_metrics()
        .await?;

    println!(
        "{} has {} stars and {}% health percentage",
        repo.full_name.unwrap(),
        repo.stargazers_count.unwrap_or(0),
        repo_metrics.health_percentage
    );

    Ok(())
}
