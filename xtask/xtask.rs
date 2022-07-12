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
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Icons => {
            icons::ICONS.iter().for_each(|(_, path)| {
                println!("{}", path);
            });
        }
        Commands::CopyIcons => {
            let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let src_root = Path::new(&root)
                .parent()
                .unwrap()
                .join("icons/icons/ffffff/transparent/1x1");
            let dst_root = Path::new(&root).parent().unwrap().join("public/icons");
            std::fs::remove_dir_all(&dst_root).unwrap_or_default();
            std::fs::create_dir_all(&dst_root).unwrap();
            icons::ICONS.iter().for_each(|(name, path)| {
                let src_path = src_root.join(path);
                let dst_path = dst_root.join(format!("{}.svg", name));

                std::fs::copy(src_path, dst_path).unwrap();
            });
        }
    }
}
