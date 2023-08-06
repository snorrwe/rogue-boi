use serde::Deserialize;
use std::{env, fs, path::Path};

const ICONS: &[(&str, &str)] = &[
    ("wall", "delapouite/brick-wall.svg"),
    ("troll", "skoll/troll.svg"),
    ("orc-head", "delapouite/orc-head.svg"),
    ("person", "delapouite/person.svg"),
    ("tombstone", "lorc/tombstone.svg"),
    ("sword", "lorc/pointy-sword.svg"),
    ("hp_potion", "delapouite/health-potion.svg"),
    ("scroll", "lorc/scroll-unfurled.svg"),
    ("stairs", "delapouite/stairs.svg"),
    ("dagger", "lorc/stiletto.svg"),
    ("chain-mail", "willdabeast/chain-mail.svg"),
    ("leather-vest", "lorc/leather-vest.svg"),
    ("warlord-helmet", "caro-asercion/warlord-helmet.svg"),
    ("goblin", "caro-asercion/goblin.svg"),
    ("gargoyle", "delapouite/gargoyle.svg"),
    ("minotaur", "lorc/minotaur.svg"),
    ("door", "delapouite/door.svg"),
    ("zombie", "delapouite/shambling-zombie.svg"),
];

#[derive(Deserialize)]
struct Svg {
    // TODO: support multiple paths?
    path: SvgPath,
}

#[derive(Deserialize)]
struct SvgPath {
    d: String,
}

fn main() {
    println!("cargo:rerun-if-changed=icons.rs");
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&root).parent().unwrap();

    let out_root = env::var_os("OUT_DIR").unwrap();
    let out_root = Path::new(&out_root);

    let archive_path = root.join("icons/icons/game-icons.net.svg.zip");
    println!("cargo:rerun-if-changed={}", archive_path.to_string_lossy());

    let mut icons_archive =
        zip::ZipArchive::new(fs::File::open(archive_path).expect("Failed to open icons file"))
            .unwrap();

    let src_root = Path::new("icons/ffffff/transparent/1x1");
    let mut payload = vec![];
    for (key, path) in ICONS {
        let src_path = src_root.join(path);
        let src_path = src_path.to_string_lossy();
        // TODO: rerun if zip changed, not icons...
        println!("loading icon {}", src_path);
        let f = icons_archive
            .by_name(src_path.as_ref())
            .expect("Failed to fetch icon");

        let data: Svg = serde_xml_rs::from_reader(f).expect("Failed to parse svg");

        payload.push((key, data.path.d));
    }
    let payload = format!(
        r#"
/// The `d` field of the svg path
pub const ICON_LIST: &[(&str, &str)] = &[{}];
pub const SVG_PATHS: &[(&str, &str)] = &[{}];
"#,
        ICONS
            .iter()
            .map(|(key, path)| { format!("(\"{}\",\"{}\")", key, path) })
            .collect::<Vec<_>>()
            .join(","),
        payload
            .iter()
            .map(|(key, path)| { format!("(\"{}\",\"{}\")", key, path) })
            .collect::<Vec<_>>()
            .join(",")
    );

    let dst_path = out_root.join("icons_svg.rs");
    fs::write(dst_path, payload).expect("Failed to write");
}
