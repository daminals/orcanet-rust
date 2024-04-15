pub mod cli;
pub mod consumer;
pub mod grpc;
pub mod producer;
pub mod store;

use std::io::{self, Write};
use store::Configurations;

use cli::{cli, handle_arg_matches};
async fn exit_gracefully(config: &mut Configurations) {
    if config.is_http_running() {
        // stop the current http client
        config.stop_http_client().await;
    }
}

#[tokio::main]
async fn main() {
    let cli = cli();
    // Load the configuration
    let mut config = store::Configurations::new().await;

    match args.producer {
        true => {
            producer::run(
                &args.bootstrap_peers,
                args.private_key,
                args.listen_address,
                args.ip,
                args.port,
            )
            .await?
        }
        false => match args.file_hash {
            Some(file_hash) => {
                consumer::run(
                    &args.bootstrap_peers,
                    args.private_key,
                    None,
                    file_hash,
                )
                .await?
            }
            None => return Err(anyhow!("No file hash provided")),
        },
    }

    println!("Orcanet Peernode CLI: Type 'help' for a list of commands");
    loop {
        // Show the command prompt
        print!("> ");
        // Print command prompt and get command
        io::stdout().flush().expect("Couldn't flush stdout");
        // take in user input, process it with cli, and then execute the command
        // if the user wants to exit, break out of the loop

        // take in user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let matches = cli
            .clone()
            .try_get_matches_from(input.split_whitespace().collect::<Vec<&str>>());
        let matches = match matches {
            Ok(matches) => matches,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        match handle_arg_matches(matches, &mut config).await {
            Ok(_) => {}
            Err(e) => eprintln!("\x1b[93mError:\x1b[0m {}", e),
        };
    }
}
