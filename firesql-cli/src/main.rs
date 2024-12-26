use std::{io::Read, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};
use firesql_core::{FireSQLParser, SQLExecutor as _};
use firestore::{FirestoreDb, FirestoreDbOptions};

mod arguments;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = arguments::Args::try_parse()?;

    let firestore = match args.gcp_service_account_key_file {
        Some(path) => {
            FirestoreDb::with_options_service_account_key_file(
                FirestoreDbOptions::new(args.firebase_project_id),
                path,
            )
            .await?
        }
        None => FirestoreDb::new(args.firebase_project_id).await?,
    };

    let sql = match args.input {
        Some(input_file) => read_from_input_file(&input_file),
        None => read_sql_from_stdin(),
    };

    let sql = sql?;
    let select = FireSQLParser::parse(&sql)?;

    let results = select.execute(&firestore).await?;

    for (index, row) in results.iter().enumerate() {
        println!("{index}: {row:?}");
    }
    Ok(())
}

fn read_sql_from_stdin() -> Result<String> {
    let stdin = std::io::stdin();
    let mut output = String::new();
    loop {
        stdin.read_line(&mut output)?;
        if firesql_core::FireSQLParser::parse(&output).is_ok() {
            return Ok(output);
        }
    }
}

fn read_from_input_file(input_file: &PathBuf) -> Result<String> {
    Ok(std::fs::read_to_string(input_file)?)
}
