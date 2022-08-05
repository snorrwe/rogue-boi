use serde::Deserialize;
use std::{env, fs::OpenOptions, path::Path};

const ICONS: &'static [(&str, &str)] = &[
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

    let src_root = root.join("icons/icons/ffffff/transparent/1x1");

    let out_root = env::var_os("OUT_DIR").unwrap();
    let out_root = Path::new(&out_root);

    let mut payload = vec![];
    for (key, path) in ICONS {
        let src_path = src_root.join(path);
        println!("cargo:rerun-if-changed={}", src_path.to_string_lossy());
        let f = OpenOptions::new()
            .read(true)
            .open(src_path)
            .expect("Failed to open svg file for reading");

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
    std::fs::write(dst_path, payload).expect("Failed to write");
}
