extern crate anyhow;
extern crate futures;
extern crate reqwest;
extern crate structopt;
extern crate tokio;

use std::collections::HashSet;

use anyhow::{Error, Result};
use futures::future::join_all;
use reqwest::Client;
use structopt::StructOpt;

use config::Opt;
use html_parser::HtmlParser;
use ip_checker::IpChecker;
use models::Player;

mod config;
mod html_parser;
mod ip_checker;
mod models;

const CHECKING_INTERVAL_IN_SECONDS: u64 = 5 * 60;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    let opt = Opt::from_args();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
        CHECKING_INTERVAL_IN_SECONDS,
    ));
    let mut checked_players_ip: HashSet<String> = HashSet::new();

    loop {
        interval.tick().await;

        match HtmlParser::fetch_and_parse_html(&client, &opt).await {
            Ok(players) => {
                let players_to_check: Vec<_> = players
                    .into_iter()
                    .filter(|player| !checked_players_ip.contains(&player.ip))
                    .collect();

                let tasks: Vec<_> = players_to_check
                    .into_iter()
                    .map(|player| {
                        checked_players_ip.insert(player.steam_id.clone());
                        process_player(&client, player, &opt.api_key)
                    })
                    .collect();

                if tasks.len() > 0 {
                    println!("--------------------------");
                }

                join_all(tasks).await;
            }
            Err(e) => eprintln!(
                "Error: {}, Failed to fetch and parse HTML or no players are playing",
                e
            ),
        }
    }
    //Ok(())
}

async fn process_player(client: &Client, player: Player, api_key: &str) -> Result<Player, Error> {
    match IpChecker::check_ip(client, &player, api_key).await {
        Ok(response) => {
            let output: String = format!("{:?} - {:?}", player, response);
            if response.proxy {
                println!("\x1b[31m{}\x1b[0m", output);
            } else {
                println!("{}", output);
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(player)
}
