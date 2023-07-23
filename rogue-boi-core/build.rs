use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::{env, path::Path};

use serde::Deserialize;

fn chances_const(name: &str, groups: &mut [ChanceRow]) -> String {
    let mut payload = format!(
        r"pub const {}: &[(u32, &[(StuffTag, i32)])] = &[
",
        name
    );
    let pl = &mut payload;
    groups.sort_unstable_by_key(|row| row.level);
    for (level, group) in &groups.iter_mut().group_by(|row| row.level) {
        writeln!(pl, "({}, &[", level).unwrap();
        for row in group {
            writeln!(pl, "(StuffTag::{}, {}),", row.tag, row.weight).unwrap();
        }
        writeln!(pl, "]),").unwrap();
    }
    writeln!(pl, "];").unwrap();
    payload
}

fn read_weights(name: &str, sheet: calamine::Range<calamine::DataType>) -> String {
    let iter = RangeDeserializerBuilder::new().from_range(&sheet).unwrap();

    let mut weights = Vec::with_capacity(1024);
    for result in iter {
        let row: ChanceRow = result.expect("Failed to deserialize row");
        weights.push(row);
    }

    chances_const(name, weights.as_mut_slice())
}

#[derive(Deserialize, Debug)]
struct ChanceRow {
    level: u32,
    tag: String,
    weight: i32,
}

#[derive(Deserialize, Debug)]
struct StuffDescription {
    tag: String,
    icon: String,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
    exp: Option<u32>,
    hp: Option<i32>,
    melee_power: Option<i32>,
    melee_skill: Option<i32>,
    defense: Option<i32>,
    heal: Option<i32>,
    ranged_power: Option<i32>,
    ranged_range: Option<i32>,
    ranged_skill: Option<i32>,
    aoe: Option<u32>,
}

fn optional_stuff<T>(
    name: &str,
    map: impl FnOnce(T) -> String,
    value: Option<T>,
    result: &mut String,
) {
    match value {
        Some(x) => {
            writeln!(result, "{}: Some({}),", name, map(x)).unwrap();
        }
        None => {
            writeln!(result, "{}: None,", name).unwrap();
        }
    }
}

fn stuff_descriptors(sheet: calamine::Range<calamine::DataType>) -> String {
    let iter = RangeDeserializerBuilder::new()
        .from_range(&sheet)
        .unwrap()
        .enumerate();

    let mut body = String::with_capacity(2048);
    let mut tags = HashMap::new();
    for (i, result) in iter {
        let row: StuffDescription = result.expect("Failed to deserialize row");

        writeln!(body, "(StuffTag::{}, StuffPrototype {{", row.tag).unwrap();
        assert!(!tags.contains_key(&row.tag), "Duplicate tag: {}", row.tag);
        tags.insert(row.tag, i);
        writeln!(body, "icon: Icon(\"{}\"),", row.icon).unwrap();
        optional_stuff(
            "name",
            |x| format!("Name(\"{}\".to_string())", x),
            row.name,
            &mut body,
        );
        optional_stuff(
            "description",
            |x| format!("Description(\"{}\".to_string())", x),
            row.description,
            &mut body,
        );
        optional_stuff(
            "color",
            |x| format!("Color(\"{}\".into())", x),
            row.color,
            &mut body,
        );
        optional_stuff(
            "exp",
            |x| format!("Exp{{ amount:{} }}", x),
            row.exp,
            &mut body,
        );
        optional_stuff("hp", |x| format!("Hp::new({})", x), row.hp, &mut body);
        optional_stuff(
            "melee",
            |(power, skill)| format!("Melee{{ power: {}, skill: {} }}", power, skill),
            row.melee_power
                .and_then(|p| row.melee_skill.map(|s| (p, s))),
            &mut body,
        );
        optional_stuff(
            "defense",
            |x| format!("Defense::new({})", x),
            row.defense,
            &mut body,
        );
        optional_stuff("heal", |x| format!("Heal::new({})", x), row.heal, &mut body);
        optional_stuff(
            "ranged",
            |(p, s, r)| format!("Ranged{{ power: {}, skill: {}, range: {} }}", p, s, r),
            row.ranged_power.and_then(|p| {
                row.ranged_range
                    .and_then(|r| row.ranged_skill.map(|s| (r, s)))
                    .map(|(r, s)| (p, r, s))
            }),
            &mut body,
        );
        optional_stuff(
            "aoe",
            |a| format!("Aoe{{ radius: {} }}", a),
            row.aoe,
            &mut body,
        );
        writeln!(body, "}}),").unwrap();
    }

    // preserve ordering of tags if possible, otherwise existing save data gets corrupted
    let mut tags = tags.into_iter().collect::<Vec<_>>();
    tags.sort_unstable_by_key(|(_, i)| *i);

    let payload = format!(
        r"fn stuff_list() -> Vec<(StuffTag, StuffPrototype)> {{
        vec![
    {}
]
}}

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize, Hash)]
#[repr(u8)]
pub enum StuffTag {{
    {}
}}
",
        body,
        tags.into_iter().map(|(tag, _)| tag).join(",\n")
    );

    payload
}

fn main() {
    println!("cargo:rerun-if-changed=assets/game-config.xlsx");

    let xls_file = env::var("CARGO_MANIFEST_DIR").unwrap();
    let xls_file = Path::new(&xls_file).join("assets/game-config.xlsx");

    let out_root = env::var_os("OUT_DIR").unwrap();
    let out_root = Path::new(&out_root);

    let mut xls: Xlsx<_> = open_workbook(xls_file).expect("Failed to open config xls");

    let enemy_chances = xls
        .worksheet_range("enemy-chances")
        .expect("Failed to open enemy chances worksheet")
        .expect("Failed to create range for enemy-chances");
    let item_chances = xls
        .worksheet_range("item-chances")
        .expect("Failed to open item chances worksheet")
        .expect("Failed to create range for item-chances");
    let stuff = xls
        .worksheet_range("stuff-descriptor")
        .expect("Failed to open stuff worksheet")
        .expect("Failed to create range for stuff");

    let enemy_weights = read_weights("ENEMY_CHANCES", enemy_chances);
    let item_weights = read_weights("ITEM_CHANCES", item_chances);

    let stuff = stuff_descriptors(stuff);

    let payload = format!("{}\n{}\n{}", enemy_weights, item_weights, stuff);

    fs::write(out_root.join("game_config_gen.rs"), payload).unwrap();
}
