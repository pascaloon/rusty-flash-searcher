use searcher::Searcher;
use clap::{Parser};

/// Searches for regex matches in files
#[derive(Parser)]
#[command(name = "Searcher")]
#[command(version = "1.0")]
#[command(about = "Searches for regex matches in files", long_about = None)]
struct Args {

    /// The regex pattern to filter file names
    file_filter: String,

    /// The regex pattern to search for
    regex: String,

    /// The directory to search (defaults to current directory)
    #[arg(default_value = ".")]
    search_dir: String,

    /// Disable colored output (default: enabled)
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    uncolored: bool,
}

mod searcher;

fn main() {
    let args = Args::parse();

    let searcher = Searcher::new(&args.regex, &args.file_filter, !args.uncolored);
    searcher.search(&args.search_dir);
}