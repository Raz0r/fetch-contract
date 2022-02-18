use structopt::StructOpt;
use reqwest;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use anyhow::{bail, Result};
use structopt::clap::arg_enum;

/// Download smart contract source code from etherscan.io given its address
#[derive(StructOpt)]
#[structopt(name = "fetch-contract")]
struct Cli {
    /// Smart contract address
    address: String,
    /// Etherscan API key
    #[structopt(short = "k", long = "apikey", env = "ETHERSCAN_APIKEY")]
    apikey: String,
    /// Chain
    #[structopt(short = "c", long = "chain")]
    chain: Chain
}

arg_enum! {
    enum Chain {
        Ethereum,
        BSC
    }
}

#[derive(Serialize, Deserialize)]
struct ApiResp {
    status: String,
    result: Vec<SourceCode>
}

#[derive(Serialize, Deserialize)]
struct ApiRespVariant {
    status: String,
    result: Vec<SourceCodeVariant>
}

#[derive(Serialize, Deserialize, Debug)]
struct Wrap {
    language: String,
    sources: HashMap<String, Content>
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct SourceCode {
    SourceCode: String
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct SourceCodeVariant {
    #[serde(with = "serde_with::json::nested")]
    SourceCode: HashMap<String, Content>
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    content: String
}

struct Contract {
    filename: String,
    contents: String,
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let url = match args.chain {
        Chain::Ethereum => format!(
            "https://api.etherscan.io/api?module=contract&action=getsourcecode&address={}&apikey={}",
            &args.address,
            &args.apikey
        ),
        Chain::BSC => format!(
            "https://api.bscscan.com/api?module=contract&action=getsourcecode&address={}&apikey={}",
            &args.address,
            &args.apikey
        )
    };
    let mut contracts: Vec<Contract> = vec!();

    match reqwest::blocking::get(&url) {
        Ok(response) => {
            match response.text() {
                Ok(text) => {
                    match serde_json::from_str::<ApiRespVariant>(&text) {
                        Ok(data) => {
                            for (filename, code) in &data.result[0].SourceCode {
                                contracts.push(Contract {
                                    filename: filename.to_string(),
                                    contents: code.content.to_string()
                                })
                            }
                        },
                        Err(_) => {
                            match serde_json::from_str::<ApiResp>(&text) {
                                Ok(data) => {
                                    if data.status == "1" {
                                        let code = &data.result[0].SourceCode;
                                        if code.len() == 0 {
                                            bail!("contract is not verified")
                                        } else {
                                            match serde_json::from_str(&code[1..code.len() - 1]) {
                                                Ok(res) => {
                                                    let sources: Wrap = res;
                                                    for (filename, code) in sources.sources {
                                                        contracts.push(Contract {
                                                            filename: filename.to_string(),
                                                            contents: code.content.to_string()
                                                        })
                                                    }
                                                },
                                                Err(e) => bail!("single file")
                                            }
                                        }
                                    }
                                },
                                Err(e) => bail!("cannot parse EtherScan response")
                            }
                        }
                    }
                },
                Err(e) => bail!("cannot download EtherScan response")
            }
        },
        Err(e) => bail!("EtherScan is down")
    }

    for contract in contracts {
        println!("Writing {}", &contract.filename);
        let pth = PathBuf::from( &contract.filename);
        let dir = pth.parent().unwrap();
        fs::create_dir_all(&dir)?;
        let path = Path::new(&contract.filename);
        let display = path.display();
        match File::create(&path) {
            Err(why) => bail!("couldn't open {}: {}", display, why),
            Ok(mut file) => {
                file.write_all(&contract.contents.as_ref())?;
            },
        };
    }
    Ok(())
}
