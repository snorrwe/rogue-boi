use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
use itertools::Itertools;
use std::fmt::Write;
use std::fs;
use std::{env, path::Path};

use serde::Deserialize;

fn chances_const(name: &str, groups: &mut [ChanceRow]) -> String {
    let mut payload = format!(
        r"
    pub const {}: &[(u32, &[(StuffTag, i32)])] = &[
",
        name
    );
    let pl = &mut payload;
    groups.sort_unstable_by_key(|row| row.level);
    for (level, group) in &groups.into_iter().group_by(|row| row.level) {
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
    let mut iter = RangeDeserializerBuilder::new().from_range(&sheet).unwrap();

    let mut weights = Vec::with_capacity(1024);
    while let Some(result) = iter.next() {
        let row: ChanceRow = result.expect("Failed to deserialize row");
        weights.push(row);
    }

    chances_const(name, weights.as_mut_slice())
}

#[derive(Deserialize, Debug)]
pub struct ChanceRow {
    pub level: u32,
    pub tag: String,
    pub weight: i32,
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

    let enemy_weights = read_weights("ENEMY_CHANCES", enemy_chances);
    let item_weights = read_weights("ITEM_CHANCES", item_chances);

    let payload = format!("{}\n{}", enemy_weights, item_weights);

    fs::write(out_root.join("game_config.rs"), payload).unwrap();
}
