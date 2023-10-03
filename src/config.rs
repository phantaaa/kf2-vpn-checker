use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short, long)]
    pub address: String,
    #[structopt(short, long)]
    pub login: String,
    #[structopt(short, long)]
    pub password: String,
    #[structopt(short, long)]
    pub api_key: String,
}
