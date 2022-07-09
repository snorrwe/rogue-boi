use std::collections::HashMap;

use cao_db::commands::Commands;

use crate::{components::*, grid::Grid, math::Vec2, Stuff};

lazy_static::lazy_static! {
    pub static ref ICONS: HashMap<&'static str, Icon> = {
        [
            ("wall", Icon("delapouite/brick-wall.svg")),
            ("troll", Icon("skoll/troll.svg")),
            ("orc-head", Icon("delapouite/orc-head.svg")),
            ("person", Icon("delapouite/person.svg")),
            ("tombstone", Icon("lorc/tombstone.svg")),
            ("sword", Icon("lorc/pointy-sword.svg")),
            ("hp_potion", Icon("delapouite/health-potion.svg")),
            ("scroll", Icon("lorc/scroll-unfurled.svg")),
        ]
            .iter()
            .map(|x|*x)
            .collect()
    };
}

pub const ENEMY_TAGS: &[StuffTag] = &[StuffTag::Troll, StuffTag::Orc];
pub const ENEMY_WEIGHTS: &[i32] = &[1, 10];

pub const ITEM_TAGS: &[StuffTag] = &[
    StuffTag::Sword,
    StuffTag::HpPotion,
    StuffTag::LightningScroll,
];
pub const ITEM_WEIGHTS: &[i32] = &[1, 2, 1];

pub fn init_entity(pos: Vec2, tag: StuffTag, cmd: &mut Commands, grid: &mut Grid<Stuff>) {
    let cmd = cmd.spawn();
    grid[pos] = Some(Default::default());
    cmd.insert(tag).insert(Pos(pos));
    match tag {
        StuffTag::Player => {
            cmd.insert(StuffTag::Player)
                .insert(pos)
                .insert(ICONS["person"])
                .insert(Hp::new(10))
                .insert(PlayerTag)
                .insert(Inventory::new(16))
                .insert(Melee { power: 1 });
        }

        StuffTag::Wall => {
            cmd.insert(ICONS["wall"]);
        }
        StuffTag::Troll => {
            cmd.insert(Hp::new(8))
                .insert(ICONS["troll"])
                .insert(Ai)
                .insert(Melee { power: 3 });
        }
        StuffTag::Orc => {
            cmd.insert(Hp::new(4))
                .insert(ICONS["orc-head"])
                .insert(Ai)
                .insert(Melee { power: 1 });
        }

        StuffTag::Sword => {
            cmd.insert(ICONS["sword"])
                .insert(Melee { power: 1 })
                .insert(Item)
                .insert(Description("Simple sword. Power 1".to_string()));
        }
        StuffTag::HpPotion => {
            cmd.insert(ICONS["hp_potion"])
                .insert(Heal { hp: 3 })
                .insert(Item)
                .insert(Description("Health potion. Heal 3".to_string()));
        }
        StuffTag::LightningScroll => {
            cmd.insert(ICONS["scroll"])
                .insert(Ranged { power: 3, range: 5 })
                .insert(Item)
                .insert(Description(
                    "Hurl a lightning bolt at your foe for 3 damage.".to_string(),
                ));
        }
    }
}
