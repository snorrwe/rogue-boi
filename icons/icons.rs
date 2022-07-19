//! This package defines what icons we actually use
//!
use std::collections::HashMap;

mod svg_paths {
    include!(concat!(env!("OUT_DIR"), "/icons_svg.rs"));
}

lazy_static::lazy_static! {
    // return relative paths
    pub static ref ICONS: HashMap<&'static str, &'static str> = {
        [
            ("wall", "delapouite/stone-wall.svg"),
            ("troll", "skoll/troll.svg"),
            ("orc-head", "delapouite/orc-head.svg"),
            ("person", "delapouite/person.svg"),
            ("tombstone", "lorc/tombstone.svg"),
            ("sword", "lorc/pointy-sword.svg"),
            ("hp_potion", "delapouite/health-potion.svg"),
            ("scroll", "lorc/scroll-unfurled.svg"),
        ]
            .iter()
            .map(|x|*x)
            .collect()
    };

    pub static ref ICONS_SVG: HashMap<&'static str, &'static str> = {
        svg_paths::SVG_PATHS
            .iter()
            .map(|x|*x)
            .collect()
    };
}
