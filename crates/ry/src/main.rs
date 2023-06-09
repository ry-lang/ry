#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::mem_forget,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::mismatched_target_os,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    nonstandard_style,
    unused_import_braces,
    unused_qualifications
)]
#![deny(
    clippy::await_holding_lock,
    clippy::if_let_mutex,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::ok_expect,
    clippy::unimplemented,
    clippy::unwrap_used,
    unsafe_code,
    unstable_features,
    unused_results
)]
#![allow(clippy::match_single_binding, clippy::inconsistent_struct_constructor)]

use std::env;

use clap::{Parser, Subcommand};

mod lex;
mod new;
mod parse;
mod parse_manifest;
mod prefix;
mod unique_file;

#[derive(Parser)]
#[command(name = "ry")]
#[command(about = "Ry programming language compiler cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Tokenize Ry source file")]
    Lex {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[command(about = "Parse Ry source file")]
    Parse { filepath: String },
    #[command(about = "Parse Ry manifest file")]
    ParseManifest { filepath: String },
    #[command(about = "Create a new Ry project")]
    New { project_name: String },
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env::var("RY_LOG").unwrap_or_else(|_| "off".to_owned()))
        .without_time()
        .with_ansi(false)
        .init();

    match Cli::parse().command {
        Commands::Lex {
            filepath,
            show_locations,
        } => lex::command(&filepath, show_locations),
        Commands::Parse { filepath } => {
            parse::command(&filepath);
        }
        Commands::ParseManifest { filepath } => {
            parse_manifest::command(&filepath);
        }
        Commands::New { project_name } => {
            new::command(&project_name);
        }
    }
}
