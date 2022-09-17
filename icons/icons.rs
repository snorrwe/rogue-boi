//! This package defines what icons we actually use
//!
use std::collections::HashMap;

mod svg_paths {
    include!(concat!(env!("OUT_DIR"), "/icons_svg.rs"));
}

lazy_static::lazy_static! {
    // return relative paths
    pub static ref ICONS: HashMap<&'static str, &'static str> = {
        svg_paths::ICON_LIST
            .iter()
            .copied()
            .collect()
    };

    pub static ref ICONS_SVG: HashMap<&'static str, &'static str> = {
        svg_paths::SVG_PATHS
            .iter()
            .copied()
            .collect()
    };
}
