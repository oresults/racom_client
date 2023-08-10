use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use clap::Parser;

mod models;
use crate::models::{Punch, PunchesRequest};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// device API token
    #[arg(short, long)]
    token: String,
    #[arg(short, long)]
    file_path: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = File::open(args.file_path.clone())
        .map_err(|e| format!("Failed to open file: {}, {}", args.file_path, e.to_string()))?;

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
