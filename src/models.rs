use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Player {
    pub name: String,
    pub ip: String,
    pub steam_id: String,
}

#[derive(Deserialize, Debug)]
pub struct IpQualityApiResponse {
    pub country_code: String,
    pub proxy: bool,
    pub vpn: bool,
}
