use crate::consumer;
use crate::producer;
use crate::store;

use anyhow::{anyhow, Result};
use clap::value_parser;
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
                            arg!(<WALLET> "The wallet server to connect to")
                                .required(false)
                                .short('w'),
                        )
                        .arg(
                            arg!(<IP> "The public IP address to announce")
                                .required(false)
                                .short('i'),
                        ),
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
                    Command::new("request")
                        .about("Requests an invoice from a producer")
                        .arg(arg!(<FILE_HASH> "The hash of the file to request").required(true))
                        .arg(arg!(<PRODUCER> "The producer to request from").required(true)),
                )
                .subcommand(
                    Command::new("get")
                        .about("Downloads a file from a producer")
                        .arg(arg!(<FILE_HASH> "The hash of the file to download").required(true))
                        .arg(arg!(<PRODUCER> "The producer to download from").required(true))
                        .arg(arg!(<CHUNK_NUM> "The chunk number to download").required(false))
                        .arg(arg!(<CONTINUE> "Continue downloading a file").required(false)),
                )
                .subcommand(
                    Command::new("auto")
                        .about("Downloads a file from a producer, paying as you go")
                        .arg(arg!(<FILE_HASH> "The hash of the file to download").required(true))
                        .arg(arg!(<PRODUCER> "The producer to download from").required(true)),
                )
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
        .subcommand(
            Command::new("wallet")
                .about("Wallet commands")
                .arg_required_else_help(true)
                .subcommand_required(true)
                .subcommand(
                    Command::new("set")
                        .about("Set the wallet server to connect to")
                        .arg(arg!(<SERVER> "The wallet server to connect to").required(true)),
                )
                .subcommand(Command::new("address").about("Get the address of the wallet"))
                .subcommand(Command::new("balance").about("Get the balance of the wallet"))
                .subcommand(
                    Command::new("invoice").about("Create an invoice").arg(
                        arg!(<AMOUNT> "The amount to invoice")
                            .required(true)
                            .value_parser(value_parser!(f32)),
                    ),
                )
                .subcommand(
                    Command::new("pay")
                        .about("Pay an invoice")
                        .arg(arg!(<INVOICE> "The invoice to pay").required(true))
                        .arg(
                            arg!(<AMOUNT> "The amount to pay")
                                .required(false)
                                .value_parser(value_parser!(f32)),
                        ),
                )
                .subcommand(
                    Command::new("check")
                        .about("Check the status of an invoice")
                        .arg(arg!(<INVOICE> "The invoice to check").required(true)),
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
                    let prices = config.get_prices();
                    // register files with the market service
                    let port = match register_matches.get_one::<String>("PORT") {
                        Some(port) => port.clone(),
                        None => config.get_port(),
                    };
                    if let Some(wallet) = register_matches.get_one::<String>("WALLET") {
                        config.set_wallet_server(wallet.clone());
                    }
                    let market_client = match register_matches.get_one::<String>("MARKET") {
                        Some(market) => config.set_market_client(market.to_owned()).await?,
                        None => config.get_market_client().await?,
                    };
                    let ip = match register_matches.get_one::<String>("IP") {
                        Some(ip) => Some(ip.clone()),
                        None => None,
                    };
                    producer::register_files(prices, market_client, port.clone(), ip).await?;
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
                Some(("ls", ls_matches)) => {
                    let file_hash = match ls_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    let market_client = config.get_market_client().await?;
                    consumer::list_producers(file_hash, market_client).await?;
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
                    consumer::get_file(
                        producer.clone(),
                        file_hash,
                        token,
                        chunk_num,
                        continue_download,
                    )
                    .await?;
                    Ok(())
                }
                Some(("auto", auto_matches)) => {
                    let file_hash = match auto_matches.get_one::<String>("FILE_HASH") {
                        Some(file_hash) => file_hash.clone(),
                        None => Err(anyhow!("No file hash provided"))?,
                    };
                    let producer = match auto_matches.get_one::<String>("PRODUCER") {
                        Some(producer) => producer.clone(),
                        None => Err(anyhow!("No producer provided"))?,
                    };
                    let wallet = config.get_wallet().await?;
                    let token = consumer::get_file_auto(producer.clone(), file_hash, wallet).await?;
                    config.set_token(producer, token);
                    Ok(())
                }
                _ => Err(anyhow!("Invalid subcommand")),
            }
        }
        Some(("wallet", wallet_matches)) => match wallet_matches.subcommand() {
            Some(("set", connect_matches)) => {
                if let Some(server) = connect_matches.get_one::<String>("SERVER") {
                    config.set_wallet_server(server.clone());
                }
                config.connect_wallet().await?;
                Ok(())
            }
            Some(("address", _)) => {
                let wallet = config.get_wallet().await?;
                let wallet = wallet.read().await;
                println!("Wallet address: {}", wallet.address);
                Ok(())
            }
            Some(("balance", _)) => {
                let wallet = config.get_wallet().await?;
                let mut wallet = wallet.write().await;
                let balance = wallet.get_balance().await?;
                println!("Wallet balance: {}", balance);
                Ok(())
            }
            Some(("invoice", invoice_matches)) => {
                let amount = match invoice_matches.get_one::<f32>("AMOUNT") {
                    Some(amount) => *amount,
                    None => Err(anyhow!("No amount provided"))?,
                };
                let wallet = config.get_wallet().await?;
                let mut wallet = wallet.write().await;
                let invoice = wallet.create_invoice(amount).await?;
                println!("Invoice: {}", invoice);
                Ok(())
            }
            Some(("pay", pay_matches)) => {
                let invoice = match pay_matches.get_one::<String>("INVOICE") {
                    Some(invoice) => invoice.clone(),
                    None => Err(anyhow!("No invoice provided"))?,
                };
                let amount = pay_matches.get_one::<f32>("AMOUNT").map(|a| *a);
                let wallet = config.get_wallet().await?;
                let mut wallet = wallet.write().await;
                wallet.pay_invoice(invoice, amount).await?;
                println!("Invoice paid");
                Ok(())
            }
            Some(("check", check_matches)) => {
                let invoice = match check_matches.get_one::<String>("INVOICE") {
                    Some(invoice) => invoice.clone(),
                    None => Err(anyhow!("No invoice provided"))?,
                };
                let wallet = config.get_wallet().await?;
                let mut wallet = wallet.write().await;
                let status = wallet.check_invoice(invoice).await?;
                println!(
                    "Invoice Amount: {}, Amount Paid: {}, Fully Paid: {}",
                    status.amount, status.amount_paid, status.paid
                );
                Ok(())
            }
            _ => Err(anyhow!("Invalid subcommand")),
        },
        Some(("market", market_matches)) => match market_matches.subcommand() {
            Some(("set", set_matches)) => {
                let market = match set_matches.get_one::<String>("MARKET") {
                    Some(market) => market.clone(),
                    None => Err(anyhow!("No market provided"))?,
                };
                config.set_market_client(market).await?;
                Ok(())
            }
            _ => Err(anyhow!("Invalid subcommand")),
        },
        Some(("exit", _)) => Ok(()),
        _ => Err(anyhow!("Invalid subcommand")),
    }
}
