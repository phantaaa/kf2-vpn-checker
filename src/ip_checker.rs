use std::collections::HashMap;

use eyre::{Context, Result};
use maplit::hashmap;
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Deserialize, Debug, Clone)]
pub struct IpQualityApiResponse {
  pub country_code: String,
  pub proxy: bool,
  pub vpn: bool,
}

pub(crate) struct IpChecker {
  client: reqwest::Client,
  api_key: String,
  cache: RwLock<HashMap<String, IpQualityApiResponse>>,
}

impl IpChecker {
  pub async fn get<S: AsRef<str>>(
    &self,
    ip: S,
  ) -> Result<IpQualityApiResponse> {
    const IP_QUALITY_BASE_URL: &str =
      "https://www.ipqualityscore.com/api/json/ip";

    let ip = ip.as_ref();

    // IMPORTANT: this block ensures that the cache lock is dropped before we do any async processing.
    // If we were to hold the lock, no other thread could make use of the ip checker.
    {
      let cache = self.cache.read().await;
      if let Some(data) = cache.get(ip) {
        return Ok(data.clone());
      }
    }

    let params = hashmap! {
        "strictness" => "1",
        "allow_public_access_points" => "true",
        "lighter_penalties" => "true",
        "mobile" => "true",
    };

    let url = format!("{}/{}/{}", IP_QUALITY_BASE_URL, self.api_key, ip);

    let response = self
      .client
      .get(&url)
      .query(&params)
      .send()
      .await
      .context("failed to check ip info")?
      .json::<IpQualityApiResponse>()
      .await
      .context("failed to parse response")?;

    let mut cache = self.cache.write().await;
    if let Some(entry) = cache.get_mut(ip) {
      *entry = response.clone()
    }

    Ok(response)
  }

  pub fn new<S>(client: reqwest::Client, api_key: S) -> Self
  where
    S: Into<String>,
  {
    let api_key = api_key.into();
    Self {
      api_key,
      client,
      cache: RwLock::new(HashMap::new()),
    }
  }
}
