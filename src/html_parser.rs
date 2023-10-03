use anyhow::{Error, Result};
use reqwest::Client;
use scraper::{Html, Selector};

use crate::config::Opt;
use crate::models::Player;

const TABLE_SELECTOR: &str = "table#players tbody tr";
const CELL_SELECTOR: &str = "td";

pub struct HtmlParser;

impl HtmlParser {
    pub async fn fetch_and_parse_html(client: &Client, params: &Opt) -> Result<Vec<Player>> {
        let response = client
            .get(format!("{}/ServerAdmin/current/players", params.address))
            .basic_auth(&params.login, Some(&params.password))
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&response);
        let row_selector = Selector::parse(TABLE_SELECTOR)
            .map_err(|_| anyhow::anyhow!("Failed to parse row selector"))?;
        let cell_selector = Selector::parse(CELL_SELECTOR)
            .map_err(|_| anyhow::anyhow!("Failed to parse cell selector"))?;

        let players: Result<Vec<Player>, _> = document
            .select(&row_selector)
            .map(|row| Self::parse_row(&row, &cell_selector))
            .collect::<Result<Vec<_>, _>>();

        players
    }

    fn parse_row(row: &scraper::ElementRef, cell_selector: &Selector) -> Result<Player, Error> {
        let columns: Vec<_> = row
            .select(cell_selector)
            .map(|td| td.text().collect::<String>().trim().to_string())
            .collect();

        if columns.len() < 6 {
            return Err(anyhow::anyhow!("Insufficient columns"));
        }

        Ok(Player {
            name: columns[1].to_string(),
            ip: columns[3].to_string(),
            steam_id: columns[5].to_string(),
        })
    }
}
