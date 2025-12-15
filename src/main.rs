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

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Create { input, output, level, threads, recursive, verbose, exclude, no_progress } => {
            let threads = threads.unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8));

            if recursive || input.is_dir() {
                // Directory compression mode
                let compression_config = glifzip::CompressionConfig::new(level, threads);
                let dir_config = glifzip::DirectoryCompressionConfig::new(compression_config)
                    .with_exclude_patterns(exclude)
                    .with_verbose(verbose)
                    .with_progress(!no_progress);

                if verbose {
                    println!("Compressing directory {} to {} (level={}, threads={})",
                             input.display(), output.display(), level, threads);
                }

                let compressor = glifzip::DirectoryCompressor::new(dir_config)?;
                compressor.compress_directory(&input, &output)
            } else {
                // Single file compression mode
                let config = glifzip::CompressionConfig::new(level, threads);

                if verbose {
                    println!("Compressing {} to {} (level={}, threads={})",
                             input.display(), output.display(), level, threads);
                }

                glifzip::compress_file(&input, &output, &config)
            }
        }

        Commands::Extract { input, output, threads, verbose, no_progress } => {
            let threads = threads.unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8));

            // Try to read the archive to determine if it's a directory archive
            let archive_data = std::fs::read(&input)?;
            let mut cursor = std::io::Cursor::new(&archive_data);

            // Try to read as directory archive first
            if let Ok(_manifest) = glifzip::ArchiveManifest::read(&mut cursor) {
                // Directory archive
                if verbose {
                    println!("Extracting directory archive {} to {} (threads={})",
                             input.display(), output.display(), threads);
                }

                glifzip::DirectoryCompressor::extract_directory(
                    &input,
                    &output,
                    threads,
                    verbose,
                    !no_progress
                )
            } else {
                // Single file archive
                if verbose {
                    println!("Extracting {} to {} (threads={})",
                             input.display(), output.display(), threads);
                }

                glifzip::decompress_file(&input, &output, threads)
            }
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

        Commands::List { input, verbose } => {
            println!("Listing contents of {}...", input.display());

            std::fs::read(&input)
                .and_then(|archive_data| {
                    let mut cursor = std::io::Cursor::new(&archive_data);
                    let manifest = glifzip::ArchiveManifest::read(&mut cursor)?;

                    println!("Archive: {}", input.display());
                    println!("Files: {}", manifest.file_count);
                    println!("Total size: {} bytes", manifest.total_size);
                    println!("Base directory: {}", manifest.base_directory.display());
                    println!("\nContents:");

                    for file_info in manifest.list_files() {
                        println!("  {}", file_info);
                    }

                    if verbose {
                        println!("\nDetailed information:");
                        for entry in &manifest.entries {
                            println!("  {} ({} bytes, mode: {:o})",
                                entry.path.display(),
                                entry.size,
                                entry.mode
                            );
                            if let Some(ref target) = entry.symlink_target {
                                println!("    -> {}", target.display());
                            }
                        }
                    }

                    Ok(())
                })
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
