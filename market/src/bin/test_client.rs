use std::io::{stdin, stdout, Write};

use lib_proto::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // port to connect to
    #[arg(short, long, default_value = "50051")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scan = stdin();

    let args = Args::parse();

    let mut client = MarketClient::connect(format!("http://127.0.0.1:{}", args.port))
        .await
        .unwrap();

    let mut user = String::new();
    print!("Enter a username: ");
    let _ = stdout().flush();
    scan.read_line(&mut user).unwrap();
    let user = user.trim();

    print!("Enter a price: ");
    let _ = stdout().flush();
    let mut price = String::new();
    scan.read_line(&mut price).unwrap();
    let price: u32 = price.trim().parse().unwrap();

    let user = User {
        name: user.to_owned(),
        id: "1".to_owned(),
        port: 416320,
        ip: "localhost".to_owned(),
        price: price.into(),
    };
    println!();
    println!("Enter 'help' to see the available commands");
    loop {
        let mut s = String::new();
        print!("> ");

        let _ = stdout().flush();
        scan.read_line(&mut s).unwrap();

        let args = s.split_whitespace().collect::<Vec<_>>();

        if args.is_empty() {
            continue;
        }

        let cmd = args.first().unwrap_or(&"");
        let file_hash = args.get(1).unwrap_or(&"");

        match *cmd {
            "register" => {
                if file_hash.is_empty() {
                    println!("Error: File hash was empty");
                    continue;
                }
                register_file(&mut client, file_hash, &user).await?;
                println!("File successfully registered");
            }
            "check" => {
                if file_hash.is_empty() {
                    println!("Error: File hash was empty");
                    continue;
                }
                let response = check_holders(&mut client, file_hash).await?;
                println!("Holders: {:?}", response.holders);
            }
            "help" => {
                println!("Commands:");
                println!("register <file_hash> - register a file");
                println!("check <file_hash> - check holders of a file");
                println!("help - show this message");
                println!("exit - exit the program");
            }
            "exit" => {
                break;
            }
            _ => {
                println!("Unknown command: {}", cmd);
            }
        }
    }

    Ok(())
}

async fn register_file(
    client: &mut MarketClient<tonic::transport::Channel>,
    file_hash: &str,
    user: &User,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(RegisterFileRequest {
        file_hash: file_hash.to_owned(),
        user: Some(user.clone()),
    });

    client
        .register_file(request)
        .await
        .map(|response| response.into_inner())
}

async fn check_holders(
    client: &mut MarketClient<tonic::transport::Channel>,
    file_hash: &str,
) -> Result<HoldersResponse, tonic::Status> {
    let request = tonic::Request::new(CheckHoldersRequest {
        file_hash: file_hash.to_owned(),
    });

    let response = client.check_holders(request).await?;

    Ok(response.into_inner())
}
