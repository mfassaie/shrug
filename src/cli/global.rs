use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
}

#[derive(Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}
