use cecs::prelude::*;
use serde::Serialize;
use serde_json::json;
use wasm_bindgen::JsValue;

use crate::{
    components::*,
    game_config::{insert_default_components, insert_default_transient_components},
    grid::Grid,
    math::Vec2,
    Stuff,
};

pub fn icon(key: &'static str) -> Icon {
    assert!(icons::ICONS.contains_key(key));
    Icon(key)
}

pub fn usable(tag: StuffTag) -> bool {
    matches!(
        tag,
        StuffTag::HpPotion
            | StuffTag::LightningScroll
            | StuffTag::ConfusionScroll
            | StuffTag::FireBallScroll
            | StuffTag::PoisonScroll
            | StuffTag::WardScroll
    )
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
        .with_component::<Poisoned>()
}

fn insert_transient_components_for_entity(cmd: &mut cecs::commands::EntityCommands, tag: StuffTag) {
    insert_default_transient_components(cmd, tag);
    match tag {
        StuffTag::Stairs => {
            cmd.insert_bundle((NextLevel, StaticVisibility));
        }
        StuffTag::Player => {
            cmd.insert_bundle((PlayerTag,));
        }
        StuffTag::Wall => {
            cmd.insert_bundle((StaticStuff, Opaque, StaticVisibility));
        }
        StuffTag::Door => {
            cmd.insert_bundle((StaticStuff, Opaque, StaticVisibility));
        }
        StuffTag::Tombstone => {
            cmd.insert_bundle((StaticStuff,));
        }
        StuffTag::Gargoyle
        | StuffTag::Troll
        | StuffTag::Orc
        | StuffTag::Warlord
        | StuffTag::Goblin
        | StuffTag::Zombie
        | StuffTag::Minotaur => {
            cmd.insert_bundle((Ai, PathCache::default(), Velocity::default()));
        }
        StuffTag::LeatherArmor | StuffTag::ChainMailArmor => {
            cmd.insert_bundle((Item, EquipmentType::Armor, StaticVisibility));
        }
        StuffTag::Sword | StuffTag::RareDagger | StuffTag::RareSword | StuffTag::Dagger => {
            cmd.insert_bundle((Item, EquipmentType::Weapon, StaticVisibility));
        }
        StuffTag::HpPotion => {
            cmd.insert_bundle((Item, StaticVisibility));
        }
        StuffTag::FireBallScroll => {
            cmd.insert_bundle((Item, StaticVisibility, NeedsTargetPosition, FireBall));
        }
        StuffTag::ConfusionScroll => {
            cmd.insert_bundle((Item, StaticVisibility, NeedsTargetEntity, ConfusionBolt));
        }
        StuffTag::LightningScroll => {
            cmd.insert_bundle((Item, StaticVisibility, NeedsTargetEntity, LightningBolt));
        }
        StuffTag::PoisonScroll => {
            cmd.insert_bundle((Item, StaticVisibility, NeedsTargetEntity, PoisionAttack));
        }
        StuffTag::WardScroll => {
            cmd.insert_bundle((Item, StaticVisibility, WardScroll));
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
    insert_default_components(cmd, tag);
    // extra
    match tag {
        StuffTag::Stairs => {}
        StuffTag::Tombstone => {}
        StuffTag::Player => {
            cmd.insert_bundle((
                LastPos(pos),
                Inventory::new(16),
                Level::default(),
                Equipment::default(),
            ));
        }

        StuffTag::Wall => {
            cmd.insert_bundle((icon("wall"), Color("#d4dfd7".into()), StaticStuff));
        }
        StuffTag::Door => {
            cmd.insert_bundle((icon("door"), Color("#d4dfd7".into()), StaticStuff));
        }
        StuffTag::Gargoyle | StuffTag::Goblin | StuffTag::Troll | StuffTag::Orc => {
            cmd.insert_bundle((Leash {
                origin: pos,
                radius: 20,
            },));
        }
        StuffTag::Zombie | StuffTag::Warlord | StuffTag::Minotaur => {
            cmd.insert_bundle((Leash {
                origin: pos,
                radius: 40,
            },));
        }
        StuffTag::LeatherArmor
        | StuffTag::PoisonScroll
        | StuffTag::ChainMailArmor
        | StuffTag::Dagger
        | StuffTag::Sword
        | StuffTag::RareSword
        | StuffTag::RareDagger
        | StuffTag::HpPotion
        | StuffTag::LightningScroll
        | StuffTag::ConfusionScroll
        | StuffTag::WardScroll
        | StuffTag::FireBallScroll => {}
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
            Option<&'a EquipmentType>,
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
    Query<&'a Equipment, With<PlayerTag>>,
)>;

pub fn stuff_to_js(id: EntityId, tag: StuffTag, query: &StuffToJsQuery) -> JsValue {
    let payload = match tag {
        StuffTag::Door | StuffTag::Stairs | StuffTag::Tombstone => {
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
                "icon": icon.0,
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
                "icon": icon.0,
                "color": color.and_then(|c|c.0.as_string())
            }}
        }
        StuffTag::Gargoyle
        | StuffTag::Goblin
        | StuffTag::Troll
        | StuffTag::Orc
        | StuffTag::Warlord
        | StuffTag::Zombie
        | StuffTag::Minotaur => {
            let (icon, name, ranged, melee, hp, description, color, defense) =
                query.q2().fetch(id).unwrap();
            json! {{
                "id": id,
                "name": name.0,
                "tag": tag,
                "ranged": ranged,
                "melee": melee,
                "description": description,
                "icon": icon.0,
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
        | StuffTag::RareSword
        | StuffTag::RareDagger
        | StuffTag::Dagger
        | StuffTag::LightningScroll
        | StuffTag::ConfusionScroll
        | StuffTag::PoisonScroll
        | StuffTag::WardScroll
        | StuffTag::FireBallScroll => {
            let (icon, name, desc, ranged, heal, melee, pos, color, defense, eq_ty) =
                query.q1().fetch(id).unwrap();

            let equipped = query
                .q5()
                .iter()
                .next()
                .map(|eq| eq.contains(id))
                .unwrap_or(false);

            let equipable = pos.is_none() && !equipped && eq_ty.is_some();

            let usable = pos.is_none() && usable(tag);
            json! {{
                "id": id,
                "tag": tag,
                "name": name.0,
                "range": ranged,
                "heal": heal,
                "melee": melee,
                "description": desc.0,
                "icon": icon.0,
                "usable": usable,
                "equipable": equipable,
                "equipped": equipped,
                "color": color.and_then(|c|c.0.as_string()),
                "item": true,
                "defense": defense
            }}
        }
    };
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    payload.serialize(&serializer).unwrap()
}
