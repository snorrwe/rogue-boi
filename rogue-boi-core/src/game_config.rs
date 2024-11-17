use crate::components::*;
use crate::HashMap;

#[derive(Debug, Clone)]
pub struct StuffPrototype {
    pub icon: Icon,
    pub name: Option<Name>,
    pub description: Option<Description>,
    pub color: Option<Color>,
    pub exp: Option<Exp>,
    pub hp: Option<Hp>,
    pub melee: Option<Melee>,
    pub defense: Option<Defense>,
    pub heal: Option<Heal>,
    pub ranged: Option<Ranged>,
    pub aoe: Option<Aoe>,
}

fn insert_optional<T: cecs::Component>(cmd: &mut cecs::commands::EntityCommands, stuff: Option<T>) {
    if let Some(s) = stuff {
        cmd.insert(s);
    }
}

pub fn insert_default_transient_components(
    cmd: &mut cecs::commands::EntityCommands,
    tag: StuffTag,
) {
    let desc = STUFF_PROTOTYPES[&tag].clone();
    cmd.insert(desc.icon);
    insert_optional(cmd, desc.name);
    insert_optional(cmd, desc.description);
    insert_optional(cmd, desc.color);
    insert_optional(cmd, desc.exp);
}

// components that are saved should not be inserted when loading
pub fn insert_default_components(cmd: &mut cecs::commands::EntityCommands, tag: StuffTag) {
    let desc = STUFF_PROTOTYPES[&tag].clone();
    insert_default_transient_components(cmd, tag);
    insert_optional(cmd, desc.hp);
    insert_optional(cmd, desc.melee);
    insert_optional(cmd, desc.defense);
    insert_optional(cmd, desc.heal);
    insert_optional(cmd, desc.ranged);
    insert_optional(cmd, desc.aoe);
}

include!(concat!(env!("OUT_DIR"), "/game_config_gen.rs"));

lazy_static::lazy_static! {
    pub static ref STUFF_PROTOTYPES: HashMap<StuffTag, StuffPrototype> = {
        stuff_list().into_iter().collect()
    };
}
