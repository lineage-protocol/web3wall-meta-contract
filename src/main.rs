#![allow(improper_ctypes)]

mod data;
mod defaults;
mod types;

use data::OpenSeaAttributes;
use ethabi::{decode, ParamType};
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::WasmLoggerBuilder;
use types::MetaContract;
use types::Metadata;
use types::Transaction;
use types::{FinalMetadata, MetaContractResult};

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

#[marine]
pub fn on_execute(
    contract: MetaContract,
    metadatas: Vec<Metadata>,
    transaction: Transaction,
) -> MetaContractResult {
    let mut finals: Vec<FinalMetadata> = vec![];

    let data: serde_json::Result<serde_json::Value> = serde_json::from_str(&transaction.data);

    match data {
        Ok(json_data) => {
            if json_data.is_object() {
                let image: Option<&str> = json_data["image"].as_str();
                let text: Option<&str> = json_data["text"].as_str();

                if image.is_none() && text.is_none() {
                    return MetaContractResult {
                        result: false,
                        metadatas: Vec::new(),
                        error_string: "No data inputted".to_string(),
                    };
                }

                if let Some(image) = image {
                    if !is_nft_storage_link(image) {
                        return MetaContractResult {
                            result: false,
                            metadatas: Vec::new(),
                            error_string: "Invalid image link is been used".to_string(),
                        };
                    }
                }

                if let Some(text) = text {
                    if is_profane(&text) {
                        // Text is profane, handle accordingly
                        return MetaContractResult {
                            result: false,
                            metadatas: Vec::new(),
                            error_string: "Profanity found in the text.".to_string(),
                        };
                    }
                }

                let text = json_data["text"].as_str().unwrap();
                if is_profane(text) {
                    // Text is profane, handle accordingly
                    return MetaContractResult {
                        result: false,
                        metadatas: Vec::new(),
                        error_string: "Profanity found in the text.".to_string(),
                    };
                }
            } else {
                // JSON schema is not valid
                return MetaContractResult {
                    result: false,
                    metadatas: Vec::new(),
                    error_string: "Data does not follow the required JSON schema.".to_string(),
                };
            }
        }
        Err(_) => {
            return MetaContractResult {
                result: false,
                metadatas: Vec::new(),
                error_string: "Data is not a valid format.".to_string(),
            };
        }
    }

    finals.push(FinalMetadata {
        public_key: transaction.public_key.clone(),
        alias: "".to_string(),
        content: transaction.data,
        loose: 1,
        version: transaction.version,
    });

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}

#[marine]
pub fn on_clone() -> bool {
    return true;
}

#[marine]
pub fn on_mint(
    contract: MetaContract,
    data_key: String,
    token_id: String,
    data: String,
) -> MetaContractResult {
    let mut error: Option<String> = None;
    let mut finals: Vec<FinalMetadata> = vec![];

    // extract out data
    if data.len() > 0 {
        let data_bytes = &hex::decode(&data);

        match data_bytes {
            Ok(decoded) => {
                let param_types = vec![ParamType::String, ParamType::String, ParamType::String];

                let results = decode(&param_types, decoded);

                match results {
                    Ok(result) => {
                        if result.len() == 3 {
                            finals.push(FinalMetadata {
                                public_key: contract.public_key.clone(),
                                alias: "name".to_string(),
                                content: result[0].clone().to_string(),
                                loose: 1,
                                version: "".to_string(),
                            });

                            finals.push(FinalMetadata {
                                public_key: contract.public_key.clone(),
                                alias: "image".to_string(),
                                content: result[1].clone().to_string(),
                                loose: 1,
                                version: "".to_string(),
                            });

                            finals.push(FinalMetadata {
                                public_key: contract.public_key.clone(),
                                alias: "body".to_string(),
                                content: result[2].clone().to_string(),
                                loose: 1,
                                version: "".to_string(),
                            });
                        }
                    }
                    Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
                }
            }
            Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
        }
    }

    if !error.is_none() {
        return MetaContractResult {
            result: false,
            metadatas: Vec::new(),
            error_string: error.unwrap().to_string(),
        };
    }

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "description".to_string(),
        content: "A subject in w3wall decentralize forum"
            .to_string(),
        loose: 1,
        version: "".to_string(),
    });

    // adds attributes
    let attr = vec![
        OpenSeaAttributes {
            trait_type: "origin".to_string(),
            value: "w3wall".to_string()
          },
          OpenSeaAttributes {
            trait_type: "type".to_string(),
            value: "topic".to_string()
          },
    ];

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "attributes".to_string(),
        content: serde_json::to_string(&attr).unwrap(),
        loose: 1,
        version: "".to_string(),
    });

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}

/**
 * For now leaving it empty. Freedom of speech
 */
fn is_profane(text: &str) -> bool {
    let profane_words = vec!["", ""];
    profane_words.iter().any(|&word| {
      if word != "" {
        return text.contains(word)
      }
      false
    })
}

fn is_nft_storage_link(link: &str) -> bool {
    link == "" || link.starts_with("https://nftstorage.link/ipfs/")
}
