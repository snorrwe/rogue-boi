use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::Path,
};

use clap::{Parser, Subcommand};
use walkdir::{DirEntry, WalkDir};
use zip::write::FileOptions;

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
    Bundle,
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
        Commands::Bundle => {
            let src_dir = root.join("public");
            let dst_file = root.join("bundle.zip");

            let file = File::create(dst_file).unwrap();

            let walk = WalkDir::new(&src_dir);
            let it = walk.into_iter().filter_map(|x| x.ok());

            zip_dir(it, src_dir, file);
        }
    }
}

fn zip_dir(
    it: impl Iterator<Item = DirEntry>,
    prefix: impl AsRef<Path>,
    writer: impl Write + Seek,
) {
    let mut zip = zip::ZipWriter::new(writer);

    let options = FileOptions::default().unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(&prefix).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options).unwrap();
            let mut f = File::open(path).unwrap();

            f.read_to_end(&mut buffer).unwrap();
            zip.write_all(&buffer).unwrap();
            buffer.clear();
        } else if path.is_dir() && name.as_os_str().len() != 0 {
            zip.add_directory(name.to_string_lossy(), options).unwrap();
        }
    }
    zip.finish().unwrap();
}
