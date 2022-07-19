use cecs::prelude::*;
use serde_json::json;
use wasm_bindgen::JsValue;

use crate::{components::*, grid::Grid, math::Vec2, Stuff};

pub const ENEMY_TAGS: &[StuffTag] = &[StuffTag::Troll, StuffTag::Orc];
pub const ENEMY_WEIGHTS: &[i32] = &[1, 10];

pub const ITEM_TAGS: &[StuffTag] = &[
    StuffTag::Sword,
    StuffTag::HpPotion,
    StuffTag::LightningScroll,
];
pub const ITEM_WEIGHTS: &[i32] = &[2, 2, 1];

pub fn icon(key: &'static str) -> Icon {
    assert!(icons::ICONS.contains_key(key));
    Icon(key)
}

pub fn init_entity(pos: Vec2, tag: StuffTag, cmd: &mut Commands, grid: &mut Grid<Stuff>) {
    let cmd = cmd.spawn();
    grid[pos] = Some(Default::default());
    cmd.insert_bundle((tag, Pos(pos)));
    match tag {
        StuffTag::Player => {
            cmd.insert_bundle((
                StuffTag::Player,
                pos,
                icon("person"),
                Hp::new(10),
                PlayerTag,
                Inventory::new(16),
                Melee { power: 1, skill: 2 },
                Color("white".into()),
                Name("Player".into()),
            ));
        }

        StuffTag::Wall => {
            cmd.insert_bundle((icon("wall"), Color("#d4dfd7".into())));
        }
        StuffTag::Troll => {
            cmd.insert_bundle((
                Hp::new(6),
                icon("troll"),
                Ai,
                PathCache::default(),
                Melee { power: 4, skill: 5 },
                Description("Large brutish troll. Clumsy, but hits hard".to_string()),
                Leash {
                    origin: pos,
                    radius: 20,
                },
                Name("Troll".into()),
            ));
        }
        StuffTag::Orc => {
            cmd.insert_bundle((
                Hp::new(4),
                icon("orc-head"),
                Ai,
                Melee { power: 1, skill: 3 },
                PathCache::default(),
                Description("Cunning, but brutal".to_string()),
                Leash {
                    origin: pos,
                    radius: 20,
                },
                Color("green".into()),
                Name("Orc".into()),
            ));
        }
        StuffTag::Sword => {
            cmd.insert_bundle((
                icon("sword"),
                Melee { power: 1, skill: 0 },
                Item,
                Description("Power 1".to_string()),
                Name("Simple Sword".into()),
            ));
        }
        StuffTag::HpPotion => {
            cmd.insert_bundle((
                icon("hp_potion"),
                Heal { hp: 3 },
                Item,
                Description("Heal 3".to_string()),
                Name("Health Potion".into()),
            ));
        }
        StuffTag::LightningScroll => {
            cmd.insert_bundle((
                icon("scroll"),
                Ranged {
                    power: 3,
                    range: 5,
                    skill: 2,
                },
                Item,
                Description("Hurl a lightning bolt at your foe for 3 damage.".to_string()),
                Name("Lightning Bolt".to_string()),
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
            &Name,
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
            &Name,
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
                    "name": "The player",
                    "description": "Yourself",
                    "icon": icon.0.clone(),
                    "hp": hp,
                    "melee": melee.clone(),
                }}
            } else {
                json! {{
                    "id": id,
                    "tag": tag,
                    "description": "Your resting place",
                    "icon": icon("tombstone").0.clone(),
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
            let (icon, name, ranged, melee, hp, description) = q_ai.fetch(id).unwrap();
            json! {{
                "id": id,
                "name": name.0.clone(),
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
            let (icon, name, desc, ranged, heal, melee, pos) = q_item.fetch(id).unwrap();
            let usable =
                pos.is_none() && matches!(tag, StuffTag::HpPotion | StuffTag::LightningScroll);
            json! {{
                "id": id,
                "tag": tag,
                "name": name.0.clone(),
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
