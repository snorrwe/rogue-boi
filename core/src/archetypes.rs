use std::collections::HashMap;

use cao_db::prelude::*;
use serde_json::json;
use wasm_bindgen::JsValue;

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
pub const ITEM_WEIGHTS: &[i32] = &[2, 2, 1];

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
                .insert(Melee { power: 1, skill: 2 });
        }

        StuffTag::Wall => {
            cmd.insert(ICONS["wall"]);
        }
        StuffTag::Troll => {
            cmd.insert(Hp::new(6))
                .insert(ICONS["troll"])
                .insert(Ai)
                .insert(PathCache::default())
                .insert(Melee { power: 4, skill: 5 })
                .insert(Description(
                    "Large brutish troll. Clumsy, but hits hard".to_string(),
                ))
                .insert(Leash {
                    origin: pos,
                    radius: 20,
                });
        }
        StuffTag::Orc => {
            cmd.insert(Hp::new(4))
                .insert(ICONS["orc-head"])
                .insert(Ai)
                .insert(Melee { power: 1, skill: 3 })
                .insert(PathCache::default())
                .insert(Description("An orc. Cunning, but brutal".to_string()))
                .insert(Leash {
                    origin: pos,
                    radius: 20,
                });
        }

        StuffTag::Sword => {
            cmd.insert(ICONS["sword"])
                .insert(Melee { power: 1, skill: 0 })
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
                .insert(Ranged {
                    power: 3,
                    range: 5,
                    skill: 2,
                })
                .insert(Item)
                .insert(Description(
                    "Hurl a lightning bolt at your foe for 3 damage.".to_string(),
                ));
        }
    }
}

pub fn stuff_to_js(
    id: EntityId,
    tag: StuffTag,
    q_wall: Query<&Icon>,
    q_item: Query<
        (
            &Icon,
            &Description,
            Option<&Ranged>,
            Option<&Heal>,
            Option<&Melee>,
            Option<&Pos>,
        ),
        With<Item>,
    >,
    q_ai: Query<
        (
            &Icon,
            Option<&Ranged>,
            Option<&Melee>,
            &Hp,
            Option<&Description>,
        ),
        With<Ai>,
    >,
    q_player: Query<(&Icon, &Melee, &Hp), With<PlayerTag>>,
) -> JsValue {
    let payload = match tag {
        StuffTag::Player => {
            if let Some((icon, melee, hp)) = q_player.fetch(id) {
                json! {{
                    "id": id,
                    "tag": tag,
                    "description": "The player",
                    "icon": icon.0.clone(),
                    "hp": hp,
                    "melee": melee.clone(),
                }}
            } else {
                json! {{
                    "id": id,
                    "tag": tag,
                    "description": "The resting place of the player",
                    "icon": ICONS["tombstone"].0.clone(),
                }}
            }
        }
        StuffTag::Wall => {
            let icon = q_wall.fetch(id).unwrap();
            json! {{
                "id": id,
                "tag": tag,
                "description": "Wall",
                "icon": icon.0.clone()
            }}
        }
        StuffTag::Troll | StuffTag::Orc => {
            let (icon, ranged, melee, hp, description) = q_ai.fetch(id).unwrap();
            json! {{
                "id": id,
                "tag": tag,
                "ranged": ranged.clone(),
                "melee": melee.clone(),
                "description": description.clone(),
                "icon": icon.0.clone(),
                "hp": hp,
                "targetable": true
            }}
        }
        StuffTag::HpPotion | StuffTag::Sword | StuffTag::LightningScroll => {
            let (icon, desc, ranged, heal, melee, pos) = q_item.fetch(id).unwrap();
            let usable =
                pos.is_none() && matches!(tag, StuffTag::HpPotion | StuffTag::LightningScroll);
            json! {{
                "id": id,
                "tag": tag,
                "range": ranged.clone(),
                "heal": heal.clone(),
                "melee": melee.clone(),
                "description": desc.0.clone(),
                "icon": icon.0.clone(),
                "usable": usable
            }}
        }
    };
    JsValue::from_serde(&payload).unwrap()
}
