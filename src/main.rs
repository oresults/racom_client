use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use clap::Parser;
use pavao::{SmbClient, SmbCredentials, SmbOpenOptions, SmbOptions};

mod models;
use crate::models::{Punch, PunchesRequest};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// device API token
    #[arg(short, long)]
    token: String,
    /// address:port
    #[arg(short, long)]
    address: String,
    #[arg(short, long)]
    file_path: String,
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    password: Option<String>,
    #[arg(short, long)]
    workgroup: Option<String>,
    #[arg(short, long)]
    share: Option<String>,
    /// supply path to local file instead of reading from SMB share
    #[arg(short, long, default_value = "false")]
    debug_local: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client = SmbClient::new(
        SmbCredentials::default()
            .server(format!("smb://{}", args.address))
            .share(args.share.unwrap_or_default())
            .username(args.username)
            .password(args.password.unwrap_or_default())
            .workgroup(args.workgroup.unwrap_or_default()),
        SmbOptions::default()
            .one_share_per_server(true),
    )
        .unwrap();

    let file: Box<dyn Read> = if args.debug_local {
        Box::new(
            File::open(args.file_path.clone())
            .map_err(|e| format!("Failed to open file: {}, {}", args.file_path, e.to_string()))?
        )
    } else {
        Box::new(
            client.open_with(args.file_path.clone(), SmbOpenOptions::default().read(true))
            .map_err(|e| format!("Failed to open file: {}, {}", args.file_path, e.to_string()))?
        )
    };

    let client = reqwest::blocking::Client::new();
    let mut punches_to_send = Vec::new();

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    loop {
        loop {
            buffer.clear();
            let bytes_read = reader.read_line(&mut buffer)?;
            if bytes_read < 24 {
                break;
            }
            match Punch::from_str(&buffer) {
                Ok(punch) => punches_to_send.push(punch),
                Err(e) => {
                    eprintln!("Failed to parse punch: {:?}, error: {:?}", buffer, e);
                    continue;
                }
            }
        }

        if !punches_to_send.is_empty() {
            let payload = PunchesRequest {
                api_token: args.token.clone() ,
                records: punches_to_send.clone()
            };

            let res = client.post("https://api.oresults.eu/punches")
                .json(&payload)
                .send()?;

            if res.status().is_success() {
                println!("Punches sent ({}): [{}]",
                         punches_to_send.len(),
                         punches_to_send.into_iter()
                             .map(|p| format!("{{{}, {}, {}}}", p.code, p.card, p.time))
                             .collect::<Vec<String>>().join(", "));
                punches_to_send = vec![];
            } else {
                eprintln!("Failed to send punches: {:?}, {}", res.status().to_string(), res.text().unwrap_or_default());
                std::thread::sleep(std::time::Duration::from_millis(5000));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
