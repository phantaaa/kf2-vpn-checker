use anyhow::Error;
use maplit::hashmap;
use reqwest::Client;

use crate::models::{IpQualityApiResponse, Player};

const IP_QUALITY_BASE_URL: &str = "https://www.ipqualityscore.com/api/json/ip";

pub struct IpChecker;

impl IpChecker {
    pub async fn check_ip(
        client: &Client,
        player: &Player,
        api_key: &str,
    ) -> Result<IpQualityApiResponse, Error> {
        let params = hashmap! {
            "strictness" => "1",
            "allow_public_access_points" => "true",
            "lighter_penalties" => "true",
            "mobile" => "true",
        };

        let url = format!("{}/{}/{}", IP_QUALITY_BASE_URL, api_key, player.ip);

        let response = client
            .get(&url)
            .query(&params)
            .send()
            .await?
            .json::<IpQualityApiResponse>()
            .await?;

        Ok(response)
    }
}
