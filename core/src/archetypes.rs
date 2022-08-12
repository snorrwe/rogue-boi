use cecs::prelude::*;
use serde_json::json;
use wasm_bindgen::JsValue;

use crate::{components::*, grid::Grid, math::Vec2, Stuff};

// sorted from low to high level
// [floor number: [(tag, weight)]]
pub const ENEMY_CHANCES: &[(u32, &[(StuffTag, i32)])] = &[
    (0, &[(StuffTag::Orc, 80)]),
    (3, &[(StuffTag::Troll, 15)]),
    (5, &[(StuffTag::Troll, 30)]),
    (7, &[(StuffTag::Troll, 60)]),
];
pub const ITEM_CHANCES: &[(u32, &[(StuffTag, i32)])] = &[
    (
        0,
        &[
            (StuffTag::HpPotion, 80),
            (StuffTag::Dagger, 40),
            (StuffTag::LeatherArmor, 40),
        ],
    ),
    (
        2,
        &[
            (StuffTag::ConfusionScroll, 10),
            (StuffTag::Sword, 30),
            (StuffTag::ChainMailArmor, 30),
        ],
    ),
    (4, &[(StuffTag::LightningScroll, 25)]),
    (6, &[(StuffTag::FireBallScroll, 25)]),
];

pub fn icon(key: &'static str) -> Icon {
    assert!(icons::ICONS.contains_key(key));
    Icon(key)
}

pub fn register_persistent_components(
    persister: impl cecs::persister::WorldSerializer,
) -> impl cecs::persister::WorldSerializer {
    persister
        .with_component::<StuffTag>()
        .with_component::<LastPos>()
        .with_component::<Pos>()
        .with_component::<Hp>()
        .with_component::<Inventory>()
        .with_component::<Melee>()
        .with_component::<Leash>()
        .with_component::<Heal>()
        .with_component::<Ranged>()
        .with_component::<Aoe>()
        .with_component::<ConfusedAi>()
        .with_component::<Level>()
        .with_component::<Exp>()
        .with_component::<Equipment>()
        .with_component::<Defense>()
}

fn insert_transient_components_for_entity(cmd: &mut cecs::commands::EntityCommands, tag: StuffTag) {
    match tag {
        StuffTag::Stairs => {
            cmd.insert_bundle((
                icon("stairs"),
                Name("Stairs".into()),
                Description("Move deeper into the dungeon".to_string()),
                NextLevel,
                Color("white".into()),
            ));
        }
        StuffTag::Tombstone => {
            cmd.insert_bundle((icon("tombstone"), Name("Tombstone".into())));
        }
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
        StuffTag::LeatherArmor => {
            cmd.insert_bundle((
                icon("leather-vest"),
                Item,
                Description("Comfy".to_string()),
                Name("Leather vest".into()),
                EquipmentType::Armor,
                Color("#00BFFF".into()),
            ));
        }
        StuffTag::ChainMailArmor => {
            cmd.insert_bundle((
                icon("chain-mail"),
                Item,
                Description("Stronk".to_string()),
                Name("Chain mail".into()),
                EquipmentType::Armor,
                Color("#00BFFF".into()),
            ));
        }
        StuffTag::Sword => {
            cmd.insert_bundle((
                icon("sword"),
                Item,
                Description("Larger weapon".to_string()),
                Name("Simple Sword".into()),
                EquipmentType::Weapon,
                Color("#00BFFF".into()),
            ));
        }
        StuffTag::Dagger => {
            cmd.insert_bundle((
                icon("dagger"),
                Item,
                Description("Small weapon".to_string()),
                Name("Simple dagger".into()),
                EquipmentType::Weapon,
                Color("#00BFFF".into()),
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
        StuffTag::Stairs => {}
        StuffTag::Tombstone => {}
        StuffTag::Player => {
            cmd.insert_bundle((
                LastPos(pos),
                Hp::new(10),
                Inventory::new(16),
                Melee { power: 1, skill: 5 },
                Level::default(),
                Equipment::default(),
                Defense::new(0),
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
                Exp { amount: 100 },
                Defense::new(4),
            ));
        }
        StuffTag::Orc => {
            cmd.insert_bundle((
                Hp::new(4),
                Melee { power: 2, skill: 3 },
                Leash {
                    origin: pos,
                    radius: 20,
                },
                Exp { amount: 35 },
                Defense::new(0),
            ));
        }
        StuffTag::LeatherArmor => {
            cmd.insert_bundle((Defense { melee_defense: 1 },));
        }
        StuffTag::ChainMailArmor => {
            cmd.insert_bundle((Defense { melee_defense: 3 },));
        }
        StuffTag::Dagger => {
            cmd.insert_bundle((Melee { power: 2, skill: 0 },));
        }
        StuffTag::Sword => {
            cmd.insert_bundle((Melee { power: 4, skill: 0 },));
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

pub type StuffToJsQuery<'a> = QuerySet<(
    Query<(&'a Icon, Option<&'a Color>)>,
    Query<
        (
            &'a Icon,
            &'a Name,
            &'a Description,
            Option<&'a Ranged>,
            Option<&'a Heal>,
            Option<&'a Melee>,
            Option<&'a Pos>,
            Option<&'a Color>,
            Option<&'a Defense>,
        ),
        With<Item>,
    >,
    Query<
        (
            &'a Icon,
            &'a Name,
            Option<&'a Ranged>,
            Option<&'a Melee>,
            &'a Hp,
            Option<&'a Description>,
            Option<&'a Color>,
            Option<&'a Defense>,
        ),
        With<Ai>,
    >,
    Query<(&'a Icon, &'a Melee, &'a Hp, &'a Defense), With<PlayerTag>>,
    Query<(&'a Icon, Option<&'a Name>, Option<&'a Description>)>,
)>;

pub fn stuff_to_js(id: EntityId, tag: StuffTag, query: StuffToJsQuery) -> JsValue {
    let payload = match tag {
        StuffTag::Stairs | StuffTag::Tombstone => {
            let (icon, name, desc) = query.q4().fetch(id).unwrap();
            json! {{
                "id": id,
                "tag": tag,
                "icon": icon.0,
                "name": name.as_ref().map(|Name(n)|n),
                "description": desc
            }}
        }
        StuffTag::Player => {
            let (icon, melee, hp, defense) = query.q3().fetch(id).unwrap();
            json! {{
                "id": id,
                "tag": tag,
                "name": "The player",
                "description": "Yourself",
                "icon": icon.0.clone(),
                "hp": hp,
                "melee": melee,
                "defense": defense
            }}
        }
        StuffTag::Wall => {
            let (icon, color) = query.q0().fetch(id).unwrap();
            json! {{
                "id": id,
                "tag": tag,
                "description": "Wall",
                "icon": icon.0.clone(),
                "color": color.and_then(|c|c.0.as_string())
            }}
        }
        StuffTag::Troll | StuffTag::Orc => {
            let (icon, name, ranged, melee, hp, description, color, defense) =
                query.q2().fetch(id).unwrap();
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
                "creature": true,
                "defense": defense
            }}
        }
        StuffTag::HpPotion
        | StuffTag::ChainMailArmor
        | StuffTag::LeatherArmor
        | StuffTag::Sword
        | StuffTag::Dagger
        | StuffTag::LightningScroll
        | StuffTag::ConfusionScroll
        | StuffTag::FireBallScroll => {
            let (icon, name, desc, ranged, heal, melee, pos, color, defense) =
                query.q1().fetch(id).unwrap();
            let equipable = pos.is_none()
                && matches!(
                    tag,
                    StuffTag::Dagger
                        | StuffTag::Sword
                        | StuffTag::LeatherArmor
                        | StuffTag::ChainMailArmor
                );
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
                "equipable": equipable,
                "color": color.and_then(|c|c.0.as_string()),
                "item": true,
                "defense": defense
            }}
        }
    };
    JsValue::from_serde(&payload).unwrap()
}
