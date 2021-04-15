use structopt::StructOpt;
use reqwest;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use anyhow::{bail, Result};

/// Download smart contract source code from etherscan.io given its address
#[derive(StructOpt)]
#[structopt(name = "fetch-contract")]
struct Cli {
    /// Smart contract address
    address: String,
    /// Etherscan API key
    #[structopt(short = "k", long = "apikey", env = "ETHERSCAN_APIKEY")]
    apikey: String
}

#[derive(Serialize, Deserialize)]
struct ApiResp {
    status: String,
    result: Vec<SourceCode>
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct SourceCode {
    #[serde(with = "serde_with::json::nested")]
    SourceCode: HashMap<String, Content>
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    content: String
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let url = format!(
        "https://api.etherscan.io/api?module=contract&action=getsourcecode&address={}&apikey={}",
        &args.address,
        &args.apikey
    );
    let data = reqwest::blocking::get(&url)?.text()?;
    let v: ApiResp = serde_json::from_str(&data)?;
    if v.status != "1" {
        bail!("An error occurred while fetching contract");
    }
    for (filename, code) in &v.result[0].SourceCode {
        println!("Writing {}", filename);
        let path = Path::new(filename);
        let display = path.display();
        match File::create(&path) {
            Err(why) => bail!("couldn't open {}: {}", display, why),
            Ok(mut file) => {
                file.write_all(&code.content.as_ref())?;
            },
        };
    }
    Ok(())
}
