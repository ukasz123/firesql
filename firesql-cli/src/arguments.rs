use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// Simple program to perform SQL queries on Firestore
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Firebase project id.
    #[arg(short, long, value_name = "project id")]
    pub(crate) firebase_project_id: String,

    /// Path to Google Cloud service account key file.
    #[arg(short, long, value_name = "GCP service key")]
    pub(crate) gcp_service_account_key_file: Option<PathBuf>,

    /// Path to file containing SQL select statement to run
    #[arg(short, long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long, value_name = "Database to perform query against")]
    pub(crate) database: Option<String>,

    #[arg(long, short, value_name = "Results output format", value_enum, default_value_t=OutputMode::AnsiTable)]
    pub(crate) output_mode: OutputMode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum OutputMode {
    AnsiTable,
    Json,
}
