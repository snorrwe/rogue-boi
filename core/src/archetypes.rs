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
    StuffTag::ConfusionScroll,
    StuffTag::FireBallScroll,
];
pub const ITEM_WEIGHTS: &[i32] = &[2, 2, 1, 2, 1];

pub fn icon(key: &'static str) -> Icon {
    assert!(icons::ICONS.contains_key(key));
    Icon(key)
}

pub fn register_persistent_components(
    persister: impl cecs::persister::WorldSerializer,
) -> impl cecs::persister::WorldSerializer {
    persister
        .add_component::<StuffTag>()
        .add_component::<LastPos>()
        .add_component::<Pos>()
        .add_component::<Hp>()
        .add_component::<Inventory>()
        .add_component::<Melee>()
        .add_component::<Leash>()
        .add_component::<Heal>()
        .add_component::<Ranged>()
        .add_component::<Aoe>()
        .add_component::<ConfusedAi>()
}

fn insert_transient_components_for_entity(cmd: &mut cecs::commands::EntityCommands, tag: StuffTag) {
    match tag {
        StuffTag::Player => {
            cmd.insert_bundle((
                icon("person"),
                PlayerTag,
                Color("white".into()),
                Name("Player".into()),
            ));
        }
        StuffTag::Wall => {
            cmd.insert_bundle((icon("wall"), Color("#d4dfd7".into()), StaticStuff));
        }
        StuffTag::Troll => {
            cmd.insert_bundle((
                Ai,
                Description("Large brutish troll. Clumsy, but hits hard".to_string()),
                PathCache::default(),
                icon("troll"),
                Name("Troll".into()),
                Velocity::default(),
            ));
        }
        StuffTag::Orc => {
            cmd.insert_bundle((
                Ai,
                Description("Cunning, but brutal".to_string()),
                PathCache::default(),
                icon("orc-head"),
                Color("#06b306".into()),
                Name("Orc".into()),
                Velocity::default(),
            ));
        }
        StuffTag::Sword => {
            cmd.insert_bundle((
                icon("sword"),
                Item,
                Description("Power 1".to_string()),
                Name("Simple Sword".into()),
            ));
        }
        StuffTag::HpPotion => {
            cmd.insert_bundle((
                icon("hp_potion"),
                Item,
                Description("Restores some health".to_string()),
                Name("Health Potion".into()),
                Color("rgb(255, 0, 127)".into()),
            ));
        }
        StuffTag::LightningScroll => {
            cmd.insert_bundle((
                icon("scroll"),
                Item,
                Description(format!("Hurl a lightning bolt at your foe")),
                Name("Lightning Bolt".to_string()),
                Color("#fee85d".into()),
            ));
        }
        StuffTag::ConfusionScroll => {
            cmd.insert_bundle((
                icon("scroll"),
                Item,
                Description(format!("Confuse the target enemy")),
                Name("Confusion Bolt".to_string()),
                Color("#800080".into()),
            ));
        }
        StuffTag::FireBallScroll => {
            cmd.insert_bundle((
                icon("scroll"),
                Item,
                Description(format!("Hurl a fireball dealing damage in an area",)),
                Name("Fire Ball".to_string()),
                Color("#af0808".into()),
            ));
        }
    }
}

/// Insert components that are not saved
pub fn insert_transient_components(mut cmd: Commands, q: Query<(EntityId, &StuffTag)>) {
    for (id, tag) in q.iter() {
        let cmd = cmd.entity(id);
        insert_transient_components_for_entity(cmd, *tag);
    }
}

pub fn init_entity(pos: Vec2, tag: StuffTag, cmd: &mut Commands, grid: &mut Grid<Stuff>) {
    let cmd = cmd.spawn();
    grid[pos] = Some(Default::default());
    cmd.insert_bundle((tag, Pos(pos)));
    insert_transient_components_for_entity(cmd, tag);
    match tag {
        StuffTag::Player => {
            cmd.insert_bundle((
                LastPos(pos),
                Hp::new(10),
                Inventory::new(16),
                Melee { power: 1, skill: 5 },
            ));
        }

        StuffTag::Wall => {
            cmd.insert_bundle((icon("wall"), Color("#d4dfd7".into()), StaticStuff));
        }
        StuffTag::Troll => {
            cmd.insert_bundle((
                Hp::new(6),
                Melee { power: 4, skill: 1 },
                Leash {
                    origin: pos,
                    radius: 20,
                },
            ));
        }
        StuffTag::Orc => {
            cmd.insert_bundle((
                Hp::new(4),
                Melee { power: 1, skill: 3 },
                Leash {
                    origin: pos,
                    radius: 20,
                },
            ));
        }
        StuffTag::Sword => {
            cmd.insert_bundle((Melee { power: 1, skill: 0 },));
        }
        StuffTag::HpPotion => {
            cmd.insert_bundle((Heal { hp: 3 },));
        }
        StuffTag::LightningScroll => {
            let power = 3;
            cmd.insert_bundle((Ranged {
                power,
                range: 5,
                skill: 4,
            },));
        }
        StuffTag::ConfusionScroll => {
            cmd.insert_bundle((Ranged {
                power: 10,
                range: 5,
                skill: 4,
            },));
        }
        StuffTag::FireBallScroll => {
            cmd.insert_bundle((
                Ranged {
                    power: 4,
                    range: 5,
                    skill: 4,
                },
                Aoe { radius: 3 },
            ));
        }
    }
}

pub fn stuff_to_js(
    id: EntityId,
    tag: StuffTag,
    q_wall: Query<(&Icon, Option<&Color>)>,
    q_item: Query<
        (
            &Icon,
            &Name,
            &Description,
            Option<&Ranged>,
            Option<&Heal>,
            Option<&Melee>,
            Option<&Pos>,
            Option<&Color>,
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
            Option<&Color>,
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
            let (icon, color) = q_wall.fetch(id).unwrap();
            json! {{
                "id": id,
                "tag": tag,
                "description": "Wall",
                "icon": icon.0.clone(),
                "color": color.and_then(|c|c.0.as_string())
            }}
        }
        StuffTag::Troll | StuffTag::Orc => {
            let (icon, name, ranged, melee, hp, description, color) = q_ai.fetch(id).unwrap();
            json! {{
                "id": id,
                "name": name.0.clone(),
                "tag": tag,
                "ranged": ranged.clone(),
                "melee": melee.clone(),
                "description": description.clone(),
                "icon": icon.0.clone(),
                "hp": hp,
                "targetable": true,
                "color": color.and_then(|c|c.0.as_string()),
                "creature": true
            }}
        }
        StuffTag::HpPotion
        | StuffTag::Sword
        | StuffTag::LightningScroll
        | StuffTag::ConfusionScroll
        | StuffTag::FireBallScroll => {
            let (icon, name, desc, ranged, heal, melee, pos, color) = q_item.fetch(id).unwrap();
            let usable = pos.is_none()
                && matches!(
                    tag,
                    StuffTag::HpPotion
                        | StuffTag::LightningScroll
                        | StuffTag::ConfusionScroll
                        | StuffTag::FireBallScroll
                );
            json! {{
                "id": id,
                "tag": tag,
                "name": name.0.clone(),
                "range": ranged.clone(),
                "heal": heal.clone(),
                "melee": melee.clone(),
                "description": desc.0.clone(),
                "icon": icon.0.clone(),
                "usable": usable,
                "color": color.and_then(|c|c.0.as_string()),
                "item": true
            }}
        }
    };
    JsValue::from_serde(&payload).unwrap()
}
