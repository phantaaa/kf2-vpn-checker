use eyre::Result;
use ip_checker::IpChecker;
use reqwest::Client;

use tracing::{error, info};
use webadmin::WebAdminClient;

mod ip_checker;
mod webadmin;

fn app() -> clap::Command {
  const PKG_NAME: &str = env!("CARGO_PKG_NAME");
  const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

  clap::Command::new(PKG_NAME)
    .version(PKG_VERSION)
    .author(clap::crate_authors!("\n"))
    .arg(
      clap::Arg::new("address")
        .required(true)
        .short('a')
        .long("address")
        .num_args(1)
        .value_name("url"),
    )
    .arg(
      clap::Arg::new("username")
        .required(true)
        .short('l')
        .long("login")
        .num_args(1),
    )
    .arg(
      clap::Arg::new("password")
        .required(true)
        .short('p')
        .long("password")
        .num_args(1),
    )
    .arg(
      clap::Arg::new("api-key")
        .required(true)
        .short('k')
        .long("api-key")
        .num_args(1)
        .value_name("key"),
    )
}

const CHECKING_INTERVAL: tokio::time::Duration =
  tokio::time::Duration::from_secs(5 * 60);

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let client = Client::new();
  let args = app().get_matches();

  macro_rules! param_required {
    ($type:ty, $name:expr) => {
      args
        .get_one::<$type>($name)
        .map(ToOwned::to_owned)
        .unwrap()
    };
  }

  let webadmin = WebAdminClient {
    address: param_required!(String, "address"),
    username: param_required!(String, "username"),
    password: param_required!(String, "password"),
    client: client.clone(),
  };
  let checker = IpChecker::new(client, param_required!(String, "api-key"));
  let mut interval = tokio::time::interval(CHECKING_INTERVAL);

  loop {
    interval.tick().await;

    info!("checking for players");

    let tasks = webadmin
      .players()
      .await?
      .into_iter()
      .map(|player| async {
        let result = checker.get(&player.ip).await;
        (player, result)
      })
      .collect::<Vec<_>>();

    for task in tasks {
      let (player, response) = task.await;
      match response {
        Ok(data) => {
          if data.proxy || data.vpn {
            info!(?player, ?data, "ඞ found sussy player ඞ");
          }
        },
        Err(err) => {
          error!(?player, ?err, "failed to check player");
        },
      }
    }
  }
}
