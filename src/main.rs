use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "glifzip")]
#[command(about = "High-performance compression engine for GlyphOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a GLIF archive from a file or directory
    Create {
        /// Input file or directory to compress
        input: PathBuf,

        /// Output GLIF archive path
        #[arg(short, long)]
        output: PathBuf,

        /// Compression level (1-22, default: 8)
        #[arg(short, long, default_value = "8")]
        level: i32,

        /// Number of threads (default: auto-detect)
        #[arg(short, long)]
        threads: Option<usize>,

        /// Compress directory recursively
        #[arg(short, long)]
        recursive: bool,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Exclude patterns (glob style, can be used multiple times)
        #[arg(short = 'x', long = "exclude")]
        exclude: Vec<String>,

        /// Disable progress bar
        #[arg(long)]
        no_progress: bool,
    },

    /// Extract a GLIF archive
    Extract {
        /// GLIF archive to extract
        input: PathBuf,

        /// Output file or directory path
        #[arg(short, long)]
        output: PathBuf,

        /// Number of threads (default: auto-detect)
        #[arg(short, long)]
        threads: Option<usize>,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Disable progress bar
        #[arg(long)]
        no_progress: bool,
    },

    /// Verify a GLIF archive
    Verify {
        /// GLIF archive to verify
        input: PathBuf,
    },

    /// List contents of a GLIF archive
    List {
        /// GLIF archive to list
        input: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Create { input, output, level, threads } => {
            let threads = threads.unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8));
            let config = glifzip::CompressionConfig::new(level, threads);

            println!("Compressing {} to {} (level={}, threads={})",
                     input.display(), output.display(), level, threads);

            glifzip::compress_file(&input, &output, &config)
        }

        Commands::Extract { input, output, threads } => {
            let threads = threads.unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8));

            println!("Extracting {} to {} (threads={})",
                     input.display(), output.display(), threads);

            glifzip::decompress_file(&input, &output, threads)
        }

        Commands::Verify { input } => {
            println!("Verifying {}...", input.display());

            std::fs::read(&input)
                .and_then(|archive| glifzip::verify_archive(&archive))
                .map(|sidecar| {
                    println!("Archive verified successfully!");
                    println!("  Payload size: {} bytes", sidecar.payload.size);
                    println!("  Archive size: {} bytes", sidecar.archive.size);
                    println!("  Compression ratio: {:.2}%", sidecar.payload.compression_ratio * 100.0);
                    println!("  Compression level: {}", sidecar.archive.compression_level);
                    println!("  Threads used: {}", sidecar.archive.threads);
                })
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
