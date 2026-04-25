use rustc_hash::FxHashMap;

use crate::{
    calc::perform::AttributeResult,
    data::{SourceId, StatId},
    item::types::{Item, ItemSlot, ItemType},
    modifier::{intern, parser, CalcContext, ModDB, ModFlag, ModType},
};

pub fn calc_attributes(db: &ModDB) -> AttributeResult {
    let ctx = CalcContext::empty();
    // Base attributes already in ModDB from class layer (rebuild_class adds base_str/dex/int)

    let strength = db.calculate(StatId::Strength, &ctx) as i32;
    let dexterity = db.calculate(StatId::Dexterity, &ctx) as i32;
    let intelligence = db.calculate(StatId::Intelligence, &ctx) as i32;

    AttributeResult {
        strength,
        dexterity,
        intelligence,
    }
}

pub fn inject_attribute_bonuses(db: &mut ModDB, attrs: &AttributeResult) {
    let src = SourceId(0);
    db.add_mod(parser::simple_mod(
        StatId::Life,
        ModType::Base,
        (attrs.strength / 2) as f64,
        src,
    ));

    db.add_mod(parser::simple_mod(
        StatId::Accuracy,
        ModType::Base,
        (attrs.dexterity * 2) as f64,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::Evasion,
        ModType::Inc,
        (attrs.dexterity / 5) as f64,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::Mana,
        ModType::Base,
        (attrs.intelligence / 2) as f64,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::EnergyShield,
        ModType::Inc,
        (attrs.intelligence / 10) as f64,
        src,
    ));
    db.add_mod(parser::flagged_mod(
        StatId::PhysicalDamage,
        ModType::Inc,
        (attrs.strength / 5) as f64,
        src,
        ModFlag::MELEE,
    ));
}
/// Returns the PoB weapon flag name for this `ItemType`, or `None` if not a weapon.
/// PoB: `env.data.weaponTypeInfo[type].flag` — the suffix of `Using<Flag>`.
fn weapon_flag(t: ItemType) -> Option<&'static str> {
    match t {
        ItemType::Claw => Some("Claw"),
        ItemType::Dagger | ItemType::RuneDagger => Some("Dagger"),
        ItemType::OneHandSword | ItemType::ThrustingOneHandSword | ItemType::TwoHandSword => {
            Some("Sword")
        }
        ItemType::OneHandAxe | ItemType::TwoHandAxe => Some("Axe"),
        ItemType::OneHandMace | ItemType::TwoHandMace | ItemType::Sceptre => Some("Mace"),
        ItemType::Wand => Some("Wand"),
        ItemType::Bow => Some("Bow"),
        ItemType::Staff | ItemType::Warstaff => Some("Staff"),
        _ => None,
    }
}

fn is_melee_weapon(t: ItemType) -> bool {
    matches!(
        t,
        ItemType::Claw
            | ItemType::Dagger
            | ItemType::RuneDagger
            | ItemType::OneHandSword
            | ItemType::ThrustingOneHandSword
            | ItemType::OneHandAxe
            | ItemType::OneHandMace
            | ItemType::Sceptre
            | ItemType::TwoHandSword
            | ItemType::TwoHandAxe
            | ItemType::TwoHandMace
            | ItemType::Staff
            | ItemType::Warstaff
    )
}

fn is_one_handed(t: ItemType) -> bool {
    matches!(
        t,
        ItemType::Claw
            | ItemType::Dagger
            | ItemType::RuneDagger
            | ItemType::OneHandSword
            | ItemType::ThrustingOneHandSword
            | ItemType::OneHandAxe
            | ItemType::OneHandMace
            | ItemType::Sceptre
            | ItemType::Wand
    )
}

fn is_two_handed(t: ItemType) -> bool {
    matches!(
        t,
        ItemType::Bow
            | ItemType::TwoHandSword
            | ItemType::TwoHandAxe
            | ItemType::TwoHandMace
            | ItemType::Staff
            | ItemType::Warstaff
    )
}

fn is_weapon(t: ItemType) -> bool {
    is_one_handed(t) || is_two_handed(t)
}

/// Sets `Using<Flag>`, `UsingMeleeWeapon`, `UsingOneHandedWeapon` / `UsingTwoHandedWeapon`
/// for a given weapon type. Called for both weapon slots (PoB sets conditions from both hands).
fn set_weapon_conditions(conds: &mut FxHashMap<&'static str, bool>, t: ItemType) {
    if let Some(flag) = weapon_flag(t) {
        conds.insert(intern(&format!("Using{flag}")), true);
    }
    if is_melee_weapon(t) {
        conds.insert(intern("UsingMeleeWeapon"), true);
    }
    if is_one_handed(t) {
        conds.insert(intern("UsingOneHandedWeapon"), true);
    } else if is_two_handed(t) {
        conds.insert(intern("UsingTwoHandedWeapon"), true);
    }
}

pub fn determine_conditions(
    _db: &ModDB,
    attrs: &AttributeResult,
    equipped: &FxHashMap<ItemSlot, Item>,
) -> FxHashMap<&'static str, bool> {
    let mut conds: FxHashMap<&'static str, bool> = FxHashMap::default();

    let weapon1 = equipped.get(&ItemSlot::Weapon1).map(|i| i.item_type);
    let weapon2 = equipped.get(&ItemSlot::Weapon2).map(|i| i.item_type);
    let has_gloves = equipped.contains_key(&ItemSlot::Gloves);

    // ── Off-hand / shield / quiver ───────────────────────────────────────────
    // PoB CalcPerform.lua: doActorAttribsConditions
    match weapon2 {
        Some(ItemType::Shield) => {
            conds.insert(intern("UsingShield"), true);
        }
        Some(ItemType::Quiver) => {
            conds.insert(intern("UsingQuiver"), true);
        }
        None => {
            conds.insert(intern("OffHandIsEmpty"), true);
        }
        _ => {}
    }

    // ── Unarmed ──────────────────────────────────────────────────────────────
    if weapon1.is_none() {
        conds.insert(intern("Unarmed"), true);
        if weapon2.is_none() && !has_gloves {
            conds.insert(intern("Unencumbered"), true);
        }
    }

    // ── Weapon-type conditions from each hand ────────────────────────────────
    if let Some(w1) = weapon1 {
        set_weapon_conditions(&mut conds, w1);
    }
    if let Some(w2) = weapon2 {
        if is_weapon(w2) {
            set_weapon_conditions(&mut conds, w2);
        }
    }

    // ── Dual-wield conditions ────────────────────────────────────────────────
    let w1_is_weapon = weapon1.map_or(false, is_weapon);
    let w2_is_weapon = weapon2.map_or(false, is_weapon);
    if w1_is_weapon && w2_is_weapon {
        conds.insert(intern("DualWielding"), true);
        let w1 = weapon1.unwrap();
        let w2 = weapon2.unwrap();
        if matches!(w1, ItemType::Claw) && matches!(w2, ItemType::Claw) {
            conds.insert(intern("DualWieldingClaws"), true);
        }
        if matches!(w1, ItemType::Dagger | ItemType::RuneDagger)
            && matches!(w2, ItemType::Dagger | ItemType::RuneDagger)
        {
            conds.insert(intern("DualWieldingDaggers"), true);
        }
        // Different weapon types while both 1H → WieldingDifferentWeaponTypes
        if weapon_flag(w1) != weapon_flag(w2) && is_one_handed(w1) && is_one_handed(w2) {
            conds.insert(intern("WieldingDifferentWeaponTypes"), true);
        }
    }

    // ── Attribute comparison conditions ──────────────────────────────────────
    // PoB CalcPerform.lua: doActorAttribsConditions — calculateAttributes()
    let str = attrs.strength;
    let dex = attrs.dexterity;
    let int = attrs.intelligence;

    conds.insert(intern("StrHigherThanDex"), str > dex);
    conds.insert(intern("StrHigherThanInt"), str > int);
    conds.insert(intern("DexHigherThanStr"), dex > str);
    conds.insert(intern("DexHigherThanInt"), dex > int);
    conds.insert(intern("IntHigherThanStr"), int > str);
    conds.insert(intern("IntHigherThanDex"), int > dex);
    conds.insert(intern("StrHighestAttribute"), str >= dex && str >= int);
    conds.insert(intern("DexHighestAttribute"), dex >= str && dex >= int);
    conds.insert(intern("IntHighestAttribute"), int >= str && int >= dex);
    let mut sorted = [str, dex, int];
    sorted.sort();
    conds.insert(intern("TwoHighestAttributesEqual"), sorted[1] == sorted[2]);

    conds
}
