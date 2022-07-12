//! This package defines what icons we actually use
//!
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref ICONS: HashMap<&'static str, &'static str> = {
        [
            ("wall", "delapouite/brick-wall.svg"),
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
}
