use crate::consumer;
use crate::producer;
use crate::store;

use anyhow::{anyhow, Result};
use clap::{arg, Command};
use store::Configurations;

#[cfg(test)]
mod tests;

pub fn cli() -> Command {
    Command::new("peernode")
        .about("Orcanet Peernode CLI")
        .no_binary_name(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("producer")
                .about("Producer node commands")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(
                    Command::new("register")
                        .about("Registers with all known market servers")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(false))
                        .arg(
                            arg!(<MARKET> "The market to connect to")
                                .required(false)
                                .short('m'),
                        )
                        .arg(
                            arg!(<IP> "The public IP address to announce")
                                .required(false)
                                .short('i'),
                        )
                )
                .subcommand(
                    Command::new("add")
                        .about("Adds a dir/file to be registered with the market server")
                        .arg(
                            arg!(<FILE_NAME> "The file or directory name to register")
                                .required(true),
                        )
                        .arg(arg!(<PRICE> "The price of the file").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("rm")
                        .about("Removes a file from the market server")
                        .arg(arg!(<FILE_NAME> "The file to remove").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("restart")
                        .about("Restarts the HTTP server")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(false)),
                )
                .subcommand(Command::new("kill").about("Kills the HTTP server"))
                .subcommand(
                    Command::new("port")
                        .about("Sets the port for the HTTP server")
                        .arg(arg!(<PORT> "The port to run the HTTP server on").required(true)),
                )
                .subcommand(
                    Command::new("market")
                        .about("Sets the market")
                        .arg(arg!(<MARKET> "The market").required(true)),
                )
                .subcommand(
                    Command::new("ls").about("Lists all files registered with the market server"),
                ),
        )
        .subcommand(
            Command::new("consumer")
                .about("Consumer node commands")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(
                    Command::new("consumer")
                        .about("Consumer node commands")
                        // .ignore_errors(true)
                        .arg_required_else_help(true)
                        .subcommand(
                            Command::new("send")
                                .about("transfer funds to another user")
                                .arg(arg!(<AMOUNT> "The amount to transfer").required(true))
                                .arg(arg!(<RECIPIENT> "The recipient of the funds").required(true))
                                .arg_required_else_help(true),
                        ),
                )
                .subcommand(
                    Command::new("upload")
                        .about("Uploads a file to a producer")
                        .arg(arg!(<FILE_NAME> "The file to upload").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("ls")
                        .about("Lists all producers with a file")
                        .arg(arg!(<FILE_HASH> "The hash of the file to list").required(true))
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("get")
                        .about("Downloads a file from a producer")
                        .arg(arg!(<FILE_HASH> "The hash of the file to download").required(true))
                        .arg(arg!(<PRODUCER> "The producer to download from").required(true))
                        .arg(arg!(<CHUNK_NUM> "The chunk number to download").required(false))
                        .arg(arg!(<CONTINUE> "Continue downloading a file").required(false)),
                ),
        )
        .subcommand(
            Command::new("market")
                .about("Market node commands")
                .subcommand_required(true)
                // .ignore_errors(true)
                .subcommand(
                    Command::new("set")
                        .about("Sets the market to connect to")
                        .arg(arg!(<MARKET> "The market to connect to").required(true)),
                ),
        )
        .subcommand(Command::new("exit").about("Exits the CLI"))
}

pub async fn handle_arg_matches(
    matches: clap::ArgMatches,
    config: &mut Configurations,
) -> Result<()> {
    match matches.subcommand() {
        Some(("producer", producer_matches)) => {
            match producer_matches.subcommand() {
                Some(("register", register_matches)) => {
                    // register files with the market service
                    let port = match register_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => config.get_port(),
                    };
                    let market = match register_matches.get_one::<String>("MARKET") {
                        Some(market) => config.set_market(market.to_owned()),
                        None => config.get_market(),
                    };
                    let ip = register_matches.get_one::<String>("IP");
                    producer::register_files(config.get_prices(), market, port.clone(), ip).await?;
                    config.start_http_client(port).await;
                    Ok(())
                }
                Some(("restart", restart_matches)) => {
                    // restart the HTTP server
                    let port = match restart_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => config.get_port(),
                    };
                    config.start_http_client(port).await;
                    Ok(())
                }
                Some(("kill", _)) => {
                    // kill the HTTP server
                    config.stop_http_client().await;
                    Ok(())
                }
                Some(("add", add_matches)) => {
                    let file_name = match add_matches
                        .get_one::<String>("FILE_NAME")
                        .map(|s| s.as_str())
                    {
                        Some(file_name) => file_name,
                        _ => Err(anyhow!("Invalid file name"))?,
                    };
                    let price = match add_matches.get_one::<String>("PRICE") {
                        Some(price) => price,
                        _ => Err(anyhow!("Invalid price"))?,
                    };
                    // get i64 price
                    let price = match price.parse::<i64>() {
                        Ok(price) => price,
                        Err(_) => {
                            // eprintln!("Invalid price");
                            return Err(anyhow!("Invalid price"));
                        }
                    };
                    config.add_file_path(file_name.to_string(), price);
                    // print
                    println!("File {} has been registered at price {}", file_name, price);
                    Ok(())
                }
                Some(("rm", rm_matches)) => {
                    let file_name = match rm_matches
                        .get_one::<String>("FILE_NAME")
                        .map(|s| s.as_str())
                    {
                        Some(file_name) => file_name,
                        _ => Err(anyhow!("Invalid file name"))?,
                    };
                    config.remove_file(file_name.to_string());
                    Ok(())
                }
                Some(("ls", _)) => {
                    let files = config.get_files();
                    let prices = config.get_prices();

                    for (hash, path) in files {
                        println!(
                            "File: {}, Price: {}",
                            path.to_string_lossy(),
                            *prices.get(&hash).unwrap_or(&0)
                        );
                    }
                    Ok(())
                }
                Some(("port", port_matches)) => {
                    let port = match port_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => Err(anyhow!("No port provided"))?,
                    };
                    config.set_port(port);
                    Ok(())
                }
                //  handle invalid subcommand
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("consumer", consumer_matches)) => {
            match consumer_matches.subcommand() {
                Some(("upload", _upload_matches)) => {
                    // Add your implementation for the upload subcommand here
                    Ok(())
                }
                Some(("send", _send_matches)) => {
                    // Add your implementation for the send subcommand here
                    Ok(())
                }
                Some(("ls", ls_matches)) => {
                    let file_hash = match ls_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    consumer::list_producers(file_hash, config.get_market()).await?;
                    Ok(())
                }
                Some(("get", get_matches)) => {
                    let file_hash = match get_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    let producer = match get_matches.get_one::<String>("PRODUCER") {
                        Some(producer) => producer.clone(),
                        None => Err(anyhow!("No producer provided"))?,
                    };
                    let chunk_num = match get_matches.get_one::<u64>("CHUNK_NUM") {
                        Some(chunk_num) => *chunk_num,
                        None => 0,
                    };
                    let continue_download = match get_matches.get_one::<bool>("CONTINUE") {
                        Some(continue_download) => *continue_download,
                        None => true,
                    };
                    let token = config.get_token(producer.clone());
                    let ret_token = match consumer::get_file(
                        producer.clone(),
                        file_hash,
                        token,
                        chunk_num,
                        continue_download,
                    )
                    .await
                    {
                        Ok(token) => token,
                        Err(e) => {
                            match e.to_string().as_str() {
                                "Request failed with status code: 404" => {
                                    println!("Consumer: File downloaded successfully");
                                }
                                _ => {
                                    eprintln!("Failed to download chunk {}: {}", chunk_num, e);
                                }
                            };
                            return Ok(());
                        }
                    };
                    config.set_token(producer, ret_token);
                    Ok(())
                }
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("market", market_matches)) => match market_matches.subcommand() {
            Some(("set", set_matches)) => {
                let market = match set_matches.get_one::<String>("MARKET") {
                    Some(market) => market.clone(),
                    None => Err(anyhow!("No market provided"))?,
                };
                config.set_market(market);
                Ok(())
            }
            _ => Err(anyhow!("Invalid subcommand")),
        },
        Some(("exit", _)) => Ok(()),
        _ => Err(anyhow!("Invalid subcommand")),
    }
}
