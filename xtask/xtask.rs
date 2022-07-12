use std::path::Path;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Icons,
    CopyIcons,
    Clean,
}

fn main() {
    let args = Args::parse();

    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&root).parent().unwrap();
    match args.command {
        Commands::Icons => {
            icons::ICONS.iter().for_each(|(_, path)| {
                println!("{}", path);
            });
        }
        Commands::CopyIcons => {
            let src_root = root.join("icons/icons/ffffff/transparent/1x1");
            let dst_root = root.join("public/icons");
            std::fs::remove_dir_all(&dst_root).unwrap_or_default();
            std::fs::create_dir_all(&dst_root).unwrap();
            icons::ICONS.iter().for_each(|(name, path)| {
                let src_path = src_root.join(path);
                let dst_path = dst_root.join(format!("{}.svg", name));

                std::fs::copy(src_path, dst_path).unwrap();
            });
        }
        Commands::Clean => {
            std::fs::remove_dir_all(root.join("public/build")).unwrap_or_default();
            std::fs::remove_dir_all(root.join("public/icons")).unwrap_or_default();
        }
    }
}
