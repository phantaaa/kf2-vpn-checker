use eyre::{anyhow, bail, Context, Error, Result};
use serde::Serialize;

const TABLE_SELECTOR: &str = "table#players tbody tr";
const CELL_SELECTOR: &str = "td";

#[derive(Serialize, Debug)]
pub(crate) struct Player {
  pub name: String,
  pub ip: String,
  pub steam_id: String,
}

pub(crate) struct WebAdminClient {
  pub client: reqwest::Client,
  pub address: String,
  pub username: String,
  pub password: String,
}

impl WebAdminClient {
  pub async fn players(&self) -> Result<Vec<Player>> {
    let data = self
      .client
      .get(format!("{}/ServerAdmin/current/players", self.address))
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await?
      .text()
      .await?;

    let players = parse(&data)
      .await
      .context("failed to parse players")?;

    Ok(players)
  }
}

async fn parse(data: &str) -> Result<Vec<Player>> {
  let document = scraper::Html::parse_document(data);
  let row_selector = scraper::Selector::parse(TABLE_SELECTOR)
    .map_err(|e| anyhow!("failed to parse row selector: {e}"))?;
  let cell_selector = scraper::Selector::parse(CELL_SELECTOR)
    .map_err(|e| anyhow!("failed to parse cell selector: {e}"))?;

  let players = document
    .select(&row_selector)
    .map(|row| parse_row(&row, &cell_selector))
    .collect();

  players
}

fn parse_row(
  row: &scraper::ElementRef,
  cell_selector: &scraper::Selector,
) -> Result<Player, Error> {
  let columns = row
    .select(cell_selector)
    .map(|td| td.text().collect::<String>())
    .collect::<Vec<_>>();

  if columns.len() < 6 {
    bail!("row doesn't have enough columns");
  }

  Ok(Player {
    name: columns[1].trim().to_string(),
    ip: columns[3].trim().to_string(),
    steam_id: columns[5].trim().to_string(),
  })
}
