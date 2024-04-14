mod crypto;
mod db;
mod grpc;

use std::{
    io::{self, Write},
    sync::Arc,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};

#[derive(Parser)]
#[command(version, about, long_about = None, multicall = true)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(about = "Show all balances")]
    Balances,
    #[command(about = "Show all invoices")]
    Invoices,
    #[command(about = "Add funds to a wallet")]
    AddFunds { wallet: String, amount: f32 },
    #[command(about = "Exit the CLI")]
    Exit,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(db::Database::new());
    let addr = "0.0.0.0:50052".parse()?;

    // Launch the gRPC server in the background
    println!("Launching gRPC server on {}", addr);
    let db_clone = db.clone();
    let join = tokio::spawn(async move {
        grpc::serve(addr, db_clone).await.unwrap();
    });

    // Start the CLI Loop
    println!("Orcanet Coin CLI: Type 'help' for a list of commands");
    loop {
        print!("> ");
        // Flush the buffer to ensure the prompt is displayed
        io::stdout().flush().expect("Failed to flush stdout");

        // Read the user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Parse the input
        let args = Args::try_parse_from(input.split_whitespace());
        let args = match args {
            Ok(args) => args,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        // Handle the command
        match args.cmd {
            Commands::Balances => {
                let balances = db.balances.read().await;
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["Wallet", "Balance"]);
                for (wallet, balance) in balances.iter() {
                    table.add_row(vec![Cell::new(wallet), Cell::new(&balance.to_string())]);
                }
                println!("{}", table);
            }
            Commands::Invoices => {
                let invoices = db.invoices.read().await;
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["ID", "Amount", "Wallet", "Paid", "Paid By"]);
                for invoice in invoices.values() {
                    table.add_row(vec![
                        Cell::new(&invoice.id),
                        Cell::new(&invoice.amount.to_string()),
                        Cell::new(&invoice.wallet),
                        Cell::new(&invoice.paid.to_string()),
                        Cell::new(&invoice.paid_by),
                    ]);
                }
                println!("{}", table);
            }
            Commands::AddFunds { wallet, amount } => {
                db.add_funds(&wallet, amount).await;
                println!("Added {} to {}", amount, wallet);
            }
            Commands::Exit => {
                break;
            }
        }
    }

    // Wait for the gRPC server to finish
    join.abort();

    Ok(())
}
