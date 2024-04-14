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

    // check if there are any arguments passed to the program
    // if there are, process them and then exit
    if std::env::args().len() > 1 {
        // remove the first argument which is the name of the program
        let args = std::env::args().skip(1).collect::<Vec<String>>();
        let matches = cli.clone().get_matches_from(args);
        match handle_arg_matches(matches, &mut config).await {
            Ok(_) => {}
            Err(e) => eprintln!("\x1b[93mError:\x1b[0m {}", e),
        };
        // wait for the HTTP server to start
        // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        if config.is_http_running() {
            // wait for user to exit with control-c
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c");
            exit_gracefully(&mut config).await;
        }
        return;
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
