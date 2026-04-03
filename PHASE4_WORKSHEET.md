# Phase 4 Worksheet: Item System

> Work through these sections in order. Each step has guided questions, hints, and
> checkpoints. Try to answer the questions and write the code yourself before looking
> at the hints. The full specification is in PLAN.md Phase 4 if you get stuck.
>
> **Difficulty ratings**: `[Easy]` `[Medium]` `[Hard]` `[Boss]`

---

## Step 4.1 — Load Base Item Data `[Easy]`

**Goal**: Parse all 22 `Bases/*.json` files into typed Rust structs. Store in `GameData`.

### Explore the Data First

> Before writing any code, open a few base item JSON files and study their shape.

**Q1.1**: Open `data/pob/Bases/Body Armour.json` and `data/pob/Bases/Bow.json`. What are the top-level keys? What does each entry look like?

<details><summary>Hint</summary>

The top-level keys are **base item names** (e.g. `"Vaal Regalia"`, `"Thicket Bow"`). Each value is an object with fields like `req`, `tags`, `implicit`, `socketLimit`, etc. Some entries have `armour` or `weapon` sub-objects depending on the category.

</details>

**Q1.2**: What fields appear on *every* base item? What fields are only present on weapons? Only on armour? Only on flasks?

<details><summary>Hint</summary>

- **Universal**: `type`, `tags`, `req` (level/str/dex/int), `socketLimit`, `implicit`, `implicitModTypes`, `influenceTags`
- **Weapons only**: `weapon` object with `AttackRateBase`, `CritChanceBase`, `PhysicalMin`, `PhysicalMax`, `Range`
- **Armour only**: `armour` object with `ArmourBaseMin/Max`, `EvasionBaseMin/Max`, `EnergyShieldBaseMin/Max`, `MovementPenalty`, `block`
- **Flasks only**: `flask` object with `life`, `mana`, `duration`, `chargesMax`, `chargesUsed`, `buff`

</details>

**Q1.3**: What is the `influenceTags` field? Why does it matter?

<details><summary>Hint</summary>

It's a map like `{ "shaper": "body_armour_shaper", "elder": "body_armour_elder" }`. It tells the mod pool system which tag set to use when an item has a particular influence — e.g., a Shaper body armour uses different mod weights than a normal one. You'll need this in Step 4.8 (Crafting) for spawn weight filtering.

</details>

### Design Your Struct

**Q1.4**: Write a `BaseItem` struct in `data/bases.rs` that can hold all the fields you found. Consider:
- How will you handle the optional `weapon`/`armour`/`flask` sub-objects?
- What type should `tags` be? (Hint: look at the project conventions for hash maps)
- What type should `implicit` be?

<details><summary>Hint</summary>

Use `Option<WeaponBase>`, `Option<ArmourBase>`, `Option<FlaskBase>` for the optional sub-objects. Use `FxHashMap<String, bool>` for `tags` (matching project conventions). `implicit` is `Vec<String>` since it's an array of mod text lines.

</details>

**Q1.5**: Write separate `WeaponBase`, `ArmourBase`, and `FlaskBase` structs for the sub-objects. What fields does each need?

<details><summary>Hint</summary>

Check PLAN.md Section J/K/L for the base field names, or re-read the JSON files. Remember to match the JSON field names — you'll want `#[serde(rename = "...")]` or `#[serde(rename_all = "camelCase")]` since the JSON uses camelCase.

</details>

### Load the Files

**Q1.6**: How will you load all 22 files from `data/pob/Bases/`? Write a function that:
1. Reads the directory
2. Parses each `.json` file
3. Merges all results into one collection

What type should the final collection be stored as? Where does it go?

<details><summary>Hint</summary>

Each file deserializes as `FxHashMap<String, BaseItem>`. Iterate all files, parse each, then `.extend()` them into a single `FxHashMap`. Store as a new field `bases: FxHashMap<String, BaseItem>` on `GameData`. Follow the pattern already used for gems/skills loading in `data/mod.rs`.

</details>

**Q1.7**: Where in the codebase does `GameData` get loaded at startup? How will you add your base loading there?

<details><summary>Hint</summary>

Look at `GameData::load_from_dir()` and `GameData::load_all_json()` in `data/mod.rs`. You'll add the bases path and loading call alongside the existing gems and skills loading.

</details>

### Checkpoint

- [ ] `BaseItem` struct defined with serde derives
- [ ] `WeaponBase`, `ArmourBase`, `FlaskBase` sub-structs defined
- [ ] Loading function reads all 22 files from `Bases/` directory
- [ ] `GameData.bases` field added and populated at startup
- [ ] Write a test: load `"Vaal Regalia"` and assert `energy_shield` values are present, `socket_limit == 6`

---

## Step 4.2 — Load Mod Pool Data `[Medium]`

**Goal**: Parse all `Mod*.json` files into a registry of available item mods.

### Explore the Data First

**Q2.1**: Open `ModItem.json` and examine one entry. What does a mod pool entry look like? What are the numbered keys (`"1"`, `"2"`, etc.)?

<details><summary>Hint</summary>

A typical entry looks like:
```json
"LocalIncreasedPhysicalDamagePercent5": {
  "affix": "Prefix",
  "group": "LocalPhysicalDamagePercent",
  "level": 46,
  "modTags": ["physical", "attack"],
  "statOrder": [1],
  "type": "Prefix",
  "weightKey": ["sword", "axe", "mace", ...],
  "weightVal": [1000, 1000, 1000, ...],
  "1": { "min": 65, "max": 84, "fmt": "#% increased Physical Damage" }
}
```
The numbered keys are stat lines. Most mods have 1-2, some have 3+. Each has a min/max range and a format template.

</details>

**Q2.2**: What's the relationship between `weightKey` and `weightVal`? How do you use them to decide if a mod can appear on an item?

<details><summary>Hint</summary>

They're **paired arrays** — `weightKey[i]` is a tag name and `weightVal[i]` is the spawn weight for items with that tag. To check eligibility: loop through both arrays together. If any of the item's tags matches a `weightKey` entry, return the corresponding `weightVal`. A value of `0` means explicitly blocked. If nothing matches, the mod can't appear on that item type.

</details>

**Q2.3**: How is `ModMaster.json` different from all other Mod files?

<details><summary>Hint</summary>

It's an **array** of objects, not an object keyed by mod ID. Also, instead of `weightKey`/`weightVal`, it uses a `types` object like `{ "Helmet": true, "Body Armour": true }`. You'll need a slightly different deserialization approach for this file.

</details>

### Design Your Structs

**Q2.4**: Design a `ModPoolEntry` struct. The tricky part: how do you handle the numbered stat keys (`"1"`, `"2"`, ...) that vary per entry?

<details><summary>Hint</summary>

You can't use regular struct fields for `"1"`, `"2"`, etc. Options:
- Use `#[serde(flatten)]` with a `HashMap<String, ModStatRange>` and filter for numeric keys
- Write a custom deserializer
- Use `serde_json::Value` for the raw JSON and extract stats manually

The simplest approach: deserialize as `serde_json::Value` first, extract known fields, then iterate remaining keys checking if they parse as numbers.

</details>

**Q2.5**: Design a `ModPoolRegistry` struct that holds all mod pools by category. Which pool files use the standard format? Which are special?

<details><summary>Hint</summary>

Standard format (object keyed by mod ID): `ModItem`, `ModJewel`, `ModFlask`, `ModJewelAbyss`, `ModJewelCharm`, `ModJewelCluster`, `ModGraft`, `ModTincture`, `ModVeiled`, `ModNecropolis`, `ModFoulborn`, `BeastCraft`

Special: `ModMaster.json` (array), `Essence.json` (different schema entirely)

</details>

### Load the Files

**Q2.6**: Write the loading logic. How many files total need to be parsed? Where will you add the loader in the startup sequence?

<details><summary>Hint</summary>

Around 14 files total. Add to `GameData::load_from_dir()` alongside bases. Each standard file: `serde_json::from_str::<FxHashMap<String, ModPoolEntry>>(...)`. ModMaster: `serde_json::from_str::<Vec<ModPoolEntry>>(...)`.

</details>

### Checkpoint

- [ ] `ModPoolEntry` and `ModStatRange` structs defined
- [ ] `ModPoolRegistry` holds all categories
- [ ] All Mod*.json files loaded at startup
- [ ] Write a test: look up a known mod ID (e.g. `"LocalIncreasedPhysicalDamagePercent5"`), verify it has correct level, tags, and stat ranges

---

## Step 4.3 — Load Unique Item Data `[Medium]`

**Goal**: Load ~4000+ unique items from raw text files. Parse metadata only at startup.

### Explore the Data First

**Q3.1**: Open a file in `data/pob/Uniques/` (e.g. `Uniques/Body Armour.json`). What is its structure? Is it what you expected?

<details><summary>Hint</summary>

It's an **array of strings**, NOT structured JSON objects. Each string is a complete item definition in PoE text format with `\n` line breaks inside the string. This is the same format as what you'd get copying an item from the game, but with extra PoB metadata tags.

</details>

**Q3.2**: Look at one of the raw text strings carefully. What metadata lines do you see that aren't in normal PoE clipboard output?

<details><summary>Hint</summary>

Look for lines like:
- `{variant:1,2}` prefixing mod lines
- `{range:0.5}` for rolled values
- `{crafted}`, `{fractured}`, `{tags:...}`
- Header lines like `League: Delve`, `Source: ...`, `Has Alt Variant: true`
- `Variant: Pre 3.19.0` / `Variant: Current`

</details>

**Q3.3**: Why should you NOT fully parse every unique at startup? What should you extract instead?

<details><summary>Hint</summary>

~4000+ uniques × full parsing with mod resolution = slow startup. Instead, extract just enough for search/browse: **name** (title), **base type**, **league**, **variant list**, and **unreleased** flag. This is a quick first-pass scan of the first few lines of each string. Full parsing happens on-demand when the user selects a unique.

</details>

### Design Your Approach

**Q3.4**: Write a `UniqueItemDef` struct for the lightweight metadata. Then write a function that takes a raw text string and extracts just the metadata fields.

<details><summary>Hint</summary>

The first 3-4 lines of each text string give you rarity + name + base type. Scan remaining lines for `Variant:`, `League:`, `Source:`, `Has Alt Variant:`, `Unreleased`. Stop scanning once you hit `--------` or mod lines. No regex needed for this — simple `line.starts_with()` checks.

</details>

**Q3.5**: What collection type should `GameData.uniques` be? Consider that users will want to search by name and base type.

<details><summary>Hint</summary>

A `Vec<UniqueItemDef>` is simplest. For search, you have a few options:
- Linear scan with `.iter().filter()` — fine for ~4000 items, substring matching is fast
- Pre-build a `FxHashMap<String, Vec<usize>>` index mapping name/base substrings to indices (overkill for now)

Start with `Vec` and iterate. Optimize only if profiling shows it's slow.

</details>

### Checkpoint

- [ ] `UniqueItemDef` struct defined
- [ ] Metadata extraction function parses name, base, league, variants from raw text
- [ ] All `Uniques/*.json` files loaded into `GameData.uniques` at startup
- [ ] Write a test: search for "Tabula Rasa", verify base is "Simple Robe"
- [ ] Write a test: verify total unique count is reasonable (~4000+)

---

## Step 4.4 — Item Type System `[Medium]`

**Goal**: Define the core `Item` struct and all its supporting enums and sub-types.

### Think About Design

**Q4.1**: The `Item` struct has ~40 fields. Should you put them all flat in one struct or group them into sub-structs? What are the trade-offs?

<details><summary>Hint</summary>

PLAN.md lays out the full flat struct, which is what PoB does. For Rust, grouping related fields can improve readability (e.g. `item.flags.corrupted` vs `item.corrupted`). But it adds indirection. Either approach works. If you group, consider:
- `ItemFlags` (10 bools)
- `ItemVariants` (variant_list, variant, has_alt_variant)
- Keep mod lines and computed data as top-level since they're accessed frequently

The flat approach is simpler to start with and matches PoB more directly.

</details>

**Q4.2**: For the `Rarity` enum, what derives do you need? Do you need `Serialize`/`Deserialize`? `specta::Type`?

<details><summary>Hint</summary>

You need `serde::Serialize` + `serde::Deserialize` if you'll send it over IPC. You need `specta::Type` if it appears in any Tauri command signature. Also `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` for general use. Think about whether it needs to parse from strings like `"Rarity: Unique"` in item text — you might want a `FromStr` impl or a `from_str()` constructor.

</details>

**Q4.3**: The `InfluenceSet` uses `bitflags`. Why bitflags instead of a `Vec<Influence>` or `HashSet<Influence>`?

<details><summary>Hint</summary>

An item can have multiple influences (e.g. Shaper + Elder), but there are only 8 total. Bitflags pack all 8 into a single `u8` — no heap allocation, instant set operations (`|`, `&`, `contains()`), and trivially copyable. A `HashSet` would be overkill for 8 boolean flags.

You already have `bitflags` in the project (used for `ModFlag` and `KeywordFlag` in `modifier/types.rs`). Same pattern.

</details>

**Q4.4**: Where should the new `item/` module live in the file structure? How will you set up the module tree?

<details><summary>Hint</summary>

Create `src-tauri/src/item/` directory with:
- `mod.rs` — re-exports
- `types.rs` — `Item`, `Rarity`, `ItemType`, `InfluenceSet`, `Socket`, etc.
- Other files added in later steps

Add `mod item;` to `lib.rs`. Follow the same pattern as the existing `modifier/` module.

</details>

**Q4.5**: Write the `ModLine` struct. What's the difference between `ModLine.mod_list` and `Item.mod_list`?

<details><summary>Hint</summary>

`ModLine.mod_list: Vec<Modifier>` contains the parsed modifiers from a *single display line* (e.g. `"+20 to Maximum Life"` → one `Modifier`). `Item.mod_list: Vec<Modifier>` is the **combined global mod list** from ALL mod lines after local mods are removed. Think of `ModLine` as the intermediate parsed form, and `Item.mod_list` as the final output that goes into ModDB.

</details>

### Checkpoint

- [ ] `item/` module created with `types.rs`
- [ ] `Rarity`, `ItemType`, `InfluenceSet`, `CatalystType`, `SocketColor` enums defined
- [ ] `Item` struct defined with all fields from PLAN.md
- [ ] `ModLine`, `Affix`, `Socket`, `Requirements` structs defined
- [ ] `WeaponData`, `ArmourData`, `FlaskData`, `TinctureData`, `JewelData` structs defined
- [ ] Everything compiles (even if nothing uses it yet)

---

## Step 4.5 — Item Text Parser `[Hard]`

**Goal**: Parse PoE clipboard text and PoB unique text into an `Item` struct.

This is the most complex parser in the project. Take it in stages.

### Understand the Input Format

**Q5.1**: Copy an item from PoE (or find an example online). Identify all the sections separated by `--------`. What does each section contain?

<details><summary>Hint</summary>

Typical order:
1. **Header**: `Rarity: Rare`, item name, base type name
2. **Properties**: weapon/armour stats, quality, sockets, item level
3. **Requirements**: Level, Str, Dex, Int
4. **Implicits**: (some items don't have this section)
5. **Explicits**: prefix/suffix mod lines
6. **Flavour text**: (uniques only, italic text)
7. **Footer flags**: `Corrupted`, `Mirrored`, etc.

Not all sections are present on every item. The parser must handle variable section counts.

</details>

**Q5.2**: What's the simplest way to split the text into sections?

<details><summary>Hint</summary>

Split on `--------`. In Rust:
```rust
let sections: Vec<&str> = text.split("--------").collect();
```
Then process each section. The first section is always the header (rarity/name). The meaning of subsequent sections depends on their content — you'll need to detect what each section contains.

</details>

### Build the Parser Incrementally

**Q5.3**: Start with just the header parser. Write a function that takes the first section and extracts `rarity`, `name`, and `base_name`. How do unique items differ from rares here?

<details><summary>Hint</summary>

The first line is always `Rarity: XXX`. For uniques, line 2 is the unique **title** and line 3 is the **base type**. For rares/magics, line 2 is the random name and line 3 is the base type. For normals, line 2 IS the base type (no separate name).

```
Rarity: Unique         Rarity: Rare           Rarity: Normal
Tabula Rasa            Havoc Ward             Vaal Regalia
Simple Robe            Vaal Regalia
```

</details>

**Q5.4**: Write a property parser for lines like `Quality: +20%`, `Sockets: R-R-G-B R`, and `Item Level: 84`. How will you extract sockets and their link groups?

<details><summary>Hint</summary>

For sockets: `"R-R-G-B R"` — dashes mean linked, spaces mean unlinked. Parse left to right:
- Start with group 0
- Each letter is a socket in the current group
- `-` means next socket is in the same group
- ` ` (space) means increment the group counter

So `R-R-G-B R` → `[Socket(R,0), Socket(R,0), Socket(G,0), Socket(B,0), Socket(R,1)]`

</details>

**Q5.5**: How will you distinguish between implicit mod lines and explicit mod lines? (They look the same textually.)

<details><summary>Hint</summary>

It's section-based, not content-based. In PoE clipboard format, implicits and explicits are in different sections separated by `--------`. The section *after* requirements/properties but *before* the main mod section contains implicits (if present). The tricky part: how many `--------` before you reach implicits vs explicits?

Strategy: parse properties/requirements first (they have recognizable patterns like `Level:`, `Quality:`). Anything after properties that's a mod line but before the next `--------` is implicit. Everything after that separator is explicit.

</details>

**Q5.6**: Now for PoB metadata tags. How will you handle `{variant:1,2}`, `{range:0.5}`, `{crafted}`, `{tags:attack,physical}` that prefix mod lines?

<details><summary>Hint</summary>

Before processing each mod line, scan for `{...}` tags at the start. Use a simple loop or regex:
```rust
// Pseudocode
while line starts with '{' {
    extract tag up to '}'
    match tag:
        "variant:X,Y" → set variant filter
        "range:N"     → set range value
        "crafted"     → set crafted flag
        "tags:a,b"    → set mod tags
        "fractured"   → set fractured flag
        ...
    line = remainder after '}'
}
```
Then the remaining `line` is the actual mod text to parse.

</details>

**Q5.7**: The final step in parsing is calling `parse_display_text()` (from Phase 2) on each mod line to get `Vec<Modifier>`. That function already exists. Where is it, and how do you call it?

<details><summary>Hint</summary>

It's in `modifier/parser.rs`. Signature: `pub fn parse_display_text(text: &str, source: SourceId) -> Vec<Modifier>`. Pass each mod line's text and a source ID. For items, the source could be a unique identifier or the item slot.

</details>

### Checkpoint

- [ ] Header parser: extracts rarity, name, base_name for all rarity types
- [ ] Property parser: quality, item level, sockets with linked groups
- [ ] Requirements parser: level, str, dex, int
- [ ] Section splitter: correctly categorizes implicits vs explicits
- [ ] PoB metadata tag extractor: variant, range, crafted, tags, fractured
- [ ] Mod lines parsed via `parse_display_text()` into `Vec<Modifier>`
- [ ] Influence/flag detection: `Corrupted`, `Shaper Item`, etc.
- [ ] Test: parse Tabula Rasa text → verify all fields
- [ ] Test: parse a rare item with 6 mods → verify prefix/suffix separation
- [ ] Test: parse a PoB unique with variants → verify variant_list populated

---

## Step 4.6 — Local Mod Extraction `[Boss]`

**Goal**: Separate local mods from global mods and compute weapon DPS / armour / flask values.

This is one of the hardest parts of Phase 4. PoB's `calcLocal()` is ~200 lines of nuanced logic.

### Understand the Local vs Global Distinction

**Q6.1**: What does "local" mean for item mods? Give 3 examples of local mods and 3 examples of global mods.

<details><summary>Hint</summary>

**Local** mods modify the item's own stats (tooltip numbers):
- `+25 to Physical Damage` (adds to weapon base damage)
- `15% increased Attack Speed` (on a weapon — modifies the weapon's attack rate)
- `120% increased Energy Shield` (on a body armour — modifies the item's ES)

**Global** mods modify your character:
- `+50 to Maximum Life` (goes into ModDB Life stat)
- `10% increased Attack Speed` (on a ring — speeds up ALL attacks)
- `+30% to Fire Resistance` (goes into ModDB FireResist)

The **same mod text** can be local or global depending on the item type! `15% increased Attack Speed` is local on a weapon but global on a ring.

</details>

**Q6.2**: How do you determine if a mod is local? What flags/conditions make it local?

<details><summary>Hint</summary>

The parser (`parse_display_text`) already assigns `ModFlag` flags to modifiers. A mod is local when:
- **On a weapon**: `ModFlag` contains `ATTACK` and the stat relates to damage/speed/crit — but you need to check *which* flags the parser sets. PoB uses weapon-type-specific flags.
- **On armour**: The stat is `Armour`, `Evasion`, `EnergyShield`, or `Ward` with no non-local flags
- **On a flask**: The stat is `FlaskDuration`, `FlaskCharges`, `FlaskRecovery`, etc.

This is where you'll likely need to study PoB's `BuildModListForSlotNum` to see exactly which stat IDs get classified as local. It's not just about flags — it's also about the stat ID itself.

</details>

### Implement Weapon DPS Calculation

**Q6.3**: Write the weapon DPS calculation. The formula is in PLAN.md Section J. Start with just physical DPS:
1. Get base phys min/max from `WeaponBase`
2. Sum flat added physical damage from local mods
3. Apply `% increased Physical Damage` (multiplicative)
4. Apply quality bonus
5. Compute DPS = `(min + max) / 2 * attack_rate`

What's the order of operations? Does quality apply before or after `% increased`?

<details><summary>Hint</summary>

Order for physical damage (from PoB):
1. `phys = base + flat_added`
2. `phys = phys * (1 + quality/100 + inc_phys/100)` — quality and inc% are ADDITIVE with each other
3. Then DPS from phys_min/phys_max

For attack speed: `rate = base_rate * (1 + local_speed_inc/100)`
For crit: `crit = base_crit * (1 + local_crit_inc/100)`

Pay careful attention: quality and `% increased Physical Damage` are summed together in the multiplier, NOT multiplied separately.

</details>

**Q6.4**: Extend to elemental and chaos damage. These are simpler — no quality bonus. Just flat adds from mods.

### Implement Armour Value Calculation

**Q6.5**: Write the armour calculation. How does it differ from weapons?

<details><summary>Hint</summary>

Similar pattern but the base value comes from `ArmourBase` min/max range. The base is typically the max value (item level determines where in the range). Quality adds to the `% increased` like weapons: `total = base * (1 + quality/100 + inc/100)`. The "base percentile" is `(actual_base - min) / (max - min)` — this is for tooltip display only, not calculations.

</details>

### Remove Local Mods From mod_list

**Q6.6**: After computing local stats, the local mods must be REMOVED from `item.mod_list`. Why? What happens if you don't?

<details><summary>Hint</summary>

If local mods stay in `mod_list`, they'll be added to the global ModDB when the item is equipped. Then a weapon's `+25 Physical Damage` would add 25 flat physical to your character's ModDB — doubling its effect (once via weapon data, once via ModDB). Local mods already affected the weapon's base numbers; they must NOT also enter the global pool.

Use `Vec::retain()` or `drain_filter()` to separate them.

</details>

### Checkpoint

- [ ] `is_local_mod(modifier, item_type)` function correctly classifies local vs global
- [ ] `compute_weapon_data(base, mods, quality)` returns correct DPS
- [ ] `compute_armour_data(base, mods, quality)` returns correct values
- [ ] `compute_flask_data(base, mods)` returns correct values
- [ ] Local mods removed from `item.mod_list` after computation
- [ ] Test: create weapon with known base + mods, verify DPS matches PoB
- [ ] Test: create armour with known base + mods, verify ES/AR/EV match PoB

---

## Step 4.7 — Equipment Manager `[Medium]`

**Goal**: Manage equipped items in slots and integrate with the ModDB layer system.

### Design the Slot System

**Q7.1**: Define an `ItemSlot` enum. How many fixed slots are there? What about dynamic slots like abyssal sockets and tree jewels?

<details><summary>Hint</summary>

17 fixed slots (Weapon1, Weapon2, Helmet, BodyArmour, Gloves, Boots, Amulet, Ring1, Ring2, Belt, Flask1-5, Graft1-2, Weapon1Swap, Weapon2Swap, Ring3).

Dynamic slots are trickier. Options:
- Enum variants with data: `AbyssalSocket(ItemSlot, u8)`, `JewelSocket(u32)`
- Separate collections: `abyssal_items: HashMap<(ItemSlot, u8), Item>`, `jewel_items: HashMap<u32, Item>`

The separate collections approach is simpler to start with. You can always unify later.

</details>

**Q7.2**: How should you validate that an item fits a slot? E.g., you can't equip a helmet in a ring slot.

<details><summary>Hint</summary>

Write a function `fn is_compatible(slot: ItemSlot, item: &Item) -> bool` that checks `item.item_type` against the slot's expected type. Some slots accept multiple types: Weapon2 can be weapon, shield, or quiver depending on what's in Weapon1 (dual wield vs 2H+quiver). Start simple and add the Weapon2 logic later.

</details>

### Integrate with ModDBLayers

**Q7.3**: Look at `ModDBLayers` in `modifier/mod_db.rs`. It currently has `tree`, `class`, and `gems` layers. How will you add the `items` layer?

<details><summary>Hint</summary>

Add `pub items: ModDB` to the struct, initialize it in `new()`, include it in `merged()`. Then write `rebuild_items(&mut self, equipment: &Equipment)` following the same pattern as `rebuild_tree()`:
```rust
pub fn rebuild_items(&mut self, equipment: &Equipment) {
    self.items = ModDB::new();
    for (slot, item) in &equipment.slots {
        let source = SourceId(slot_to_source_id(slot));
        for modifier in &item.mod_list {
            self.items.add_mod(modifier.clone());
        }
    }
}
```

</details>

**Q7.4**: When should `rebuild_items()` be called? Every equip/unequip? What about weapon swap?

<details><summary>Hint</summary>

Call it after any equipment change: equip, unequip, swap weapons, change item set. Weapon swap doesn't re-parse items — it just changes which slots are "active" and rebuilds the layer.

</details>

### Tauri Commands

**Q7.5**: What new Tauri commands do you need? Remember the pattern from existing commands: `#[tauri::command]` + `#[specta::specta]`, return `Result<T, String>`.

<details><summary>Hint</summary>

At minimum:
- `equip_item(slot, item_text)` — parse the text into `Item`, equip it, rebuild items layer, recalculate stats
- `unequip_item(slot)` — remove item, rebuild, recalculate
- `get_equipped_items()` — return summaries of all equipped items for UI display
- `search_uniques(query)` — search unique metadata by name/base

Don't forget to register them in `collect_commands![]` in `lib.rs` and add `Serialize + Deserialize + Type` derives to any types crossing IPC.

</details>

### Checkpoint

- [ ] `ItemSlot` enum defined with all fixed slots
- [ ] `Equipment` struct with `FxHashMap<ItemSlot, Item>`
- [ ] `is_compatible()` slot validation
- [ ] `items` layer added to `ModDBLayers`
- [ ] `rebuild_items()` collects all equipped item mods into the layer
- [ ] `merged()` includes items layer
- [ ] Tauri commands: equip, unequip, get equipped, search uniques
- [ ] Test: equip item → verify mods appear in merged ModDB
- [ ] Test: unequip item → verify mods removed

---

## Step 4.8 — Crafting Engine `[Hard]`

**Goal**: Create and modify items programmatically — select base, add affixes, validate limits.

### Understand Affix Rules

**Q8.1**: What are the affix limits for each rarity? How do jewels differ?

<details><summary>Hint</summary>

| Rarity | Max Prefixes | Max Suffixes | Total |
|--------|-------------|-------------|-------|
| Normal | 0 | 0 | 0 |
| Magic | 1 | 1 | 2 |
| Rare | 3 | 3 | 6 |
| Unique | varies | varies | varies |

Jewels are special: max 2 prefixes + 2 suffixes (4 total) for rare jewels.

</details>

**Q8.2**: What is a mod "group" and why does it matter for crafting?

<details><summary>Hint</summary>

The `group` field on `ModPoolEntry` defines mutual exclusion. Two mods in the same group can't both be on an item. For example, "Tier 1 Life" and "Tier 5 Life" share `group: "IncreasedLife"` — you can only have one tier of life. Your crafting validator must check: no two equipped affixes share a group.

</details>

### Implement Spawn Weight Filtering

**Q8.3**: Write `get_mod_spawn_weight()`. Given a mod entry and an item+base, determine if the mod can appear and with what weight.

*This is the core of the crafting system — get it right and everything else follows.*

<details><summary>Hint</summary>

```rust
fn get_mod_spawn_weight(entry: &ModPoolEntry, base: &BaseItem, item: &Item) -> i32 {
    for (key, val) in entry.weight_key.iter().zip(&entry.weight_val) {
        // Check base item tags
        if base.tags.contains_key(key) {
            return *val;
        }
        // Check influence tags
        if item_has_influence_tag(item, base, key) {
            return *val;
        }
    }
    0 // Not eligible
}
```

The influence tag check: if the item has Shaper influence, check `base.influence_tags["shaper"]` to get the influence-specific tag, then see if that matches the weight key.

</details>

**Q8.4**: Write a function that returns all eligible mods for a given item (base + rarity + influences). Filter by:
1. Spawn weight > 0
2. Item level >= mod level
3. Correct affix type (prefix/suffix) matching available slots
4. No group conflicts with existing affixes

<details><summary>Hint</summary>

Chain your filters:
```rust
mod_pool.iter()
    .filter(|m| get_mod_spawn_weight(m, base, item) > 0)
    .filter(|m| item.item_level >= m.level)
    .filter(|m| match m.affix {
        Prefix => item.prefixes.len() < item.prefix_limit,
        Suffix => item.suffixes.len() < item.suffix_limit,
    })
    .filter(|m| !item_has_group(&item, &m.group))
    .collect()
```

</details>

### Checkpoint

- [ ] `create_item(base_name, rarity)` creates a blank item from a base
- [ ] `add_affix(item, mod_id, range)` adds an affix with validation
- [ ] Affix limit enforcement (rejects when at max)
- [ ] Group mutual exclusion (rejects duplicate groups)
- [ ] `get_eligible_mods(item)` returns filtered mod list
- [ ] Spawn weight filtering uses base tags + influence tags
- [ ] Catalyst quality scaling applied to matching mod lines
- [ ] Test: create rare, add 3P+3S, verify 4th prefix rejected
- [ ] Test: verify group mutual exclusion

---

## Step 4.9 — Item Tooltip `[Easy]`

**Goal**: Generate structured tooltip output for frontend rendering.

**Q9.1**: What format should the tooltip output be? Plain text? HTML? Structured data?

<details><summary>Hint</summary>

Structured data is best for Svelte rendering. Return a `Vec<TooltipLine>` where each line has `{ text: String, color: String }`. The frontend component iterates and applies the colours. Don't embed HTML — let the Svelte component handle presentation.

The colour string can be a CSS hex colour or a named enum — either works. Named enums (`"MAGIC"`, `"CRAFTED"`) let the frontend theme them.

</details>

**Q9.2**: In what order should tooltip sections appear? (Check PLAN.md Section 4.9 for the full list.)

**Q9.3**: How will you handle the separator lines (`--------`) in the output?

<details><summary>Hint</summary>

Add a special `TooltipLine` variant or a `separator: bool` flag. The frontend renders it as a horizontal rule. Or use a sentinel string like `"---"` that the frontend recognizes.

</details>

### Checkpoint

- [ ] `TooltipLine` struct (or enum) defined
- [ ] `build_tooltip(item) -> Vec<TooltipLine>` generates all sections
- [ ] Colour codes assigned based on mod line flags (crafted, fractured, etc.)
- [ ] Weapon DPS/armour values shown in properties section
- [ ] Test: generate tooltip for a unique, verify it contains name + base + mods

---

## Step 4.10 — ItemsTab Frontend `[Medium]`

**Goal**: Build the Svelte UI for managing items.

### Plan the Component Structure

**Q10.1**: The PLAN.md suggests 6 sub-components. Do you need all of them right away? What's the minimum viable UI?

<details><summary>Hint</summary>

Start with just:
1. `ItemsTab.svelte` — layout with slot list
2. `EquipmentSlots.svelte` — clickable slot grid
3. A simple text area for pasting item text + an "Equip" button

That's enough to test the full pipeline: paste → parse → equip → see stats change. Add the unique search, crafting UI, and tooltips incrementally.

</details>

**Q10.2**: How will the frontend know about equipped items? What Tauri command provides this data?

<details><summary>Hint</summary>

Call `commands.getEquippedItems()` on mount and after any equip/unequip. Store the result with `$state`. The command returns a map of slot name → item summary (name, base, rarity, tooltip preview).

Remember: all IPC goes through `commands` from `src/bindings.ts` (auto-generated by tauri-specta).

</details>

**Q10.3**: How does the Svelte component call Tauri commands? What rune pattern should you use for the response data?

<details><summary>Hint</summary>

```svelte
<script lang="ts">
  import { commands } from '../bindings';

  let equippedItems = $state<Record<string, ItemSummary>>({});

  async function loadEquipment() {
    equippedItems = await commands.getEquippedItems();
  }

  async function equipItem(slot: string, text: string) {
    await commands.equipItem(slot, text);
    await loadEquipment(); // Refresh
  }
</script>
```

Use `$state` for reactive data, `$effect` for on-mount loading, `$derived` for computed views.

</details>

**Q10.4**: How will you add a route for the items tab? Look at how `/skilltree` is set up.

<details><summary>Hint</summary>

Create `src/routes/items/+page.svelte`. It'll automatically be available at `/items`. Look at `src/routes/skilltree/+page.svelte` for the pattern — it imports components and composes the page layout. Add a navigation link from the header or home page.

</details>

### Checkpoint

- [ ] `ItemsTab.svelte` component created
- [ ] Equipment slot display showing all 17+ slots
- [ ] Text paste area + equip button working
- [ ] Equipping updates stats (visible in sidebar/debug page)
- [ ] Unequip button per slot
- [ ] Route set up at `/items`
- [ ] Navigation from header to items page

---

## Bonus Challenges

Once the core is working, try these extensions:

### B1 — Unique Search `[Easy]`
Add a searchable unique item browser. Type-ahead filtering on the pre-indexed `UniqueItemDef` metadata. Clicking a result equips it to the selected slot.

### B2 — Stat Comparison `[Medium]`
When hovering an item over an equipped slot, show the stat diff ("+15 Life", "-3% Fire Res"). This requires computing stats with both the old and new item equipped and diffing the results.

### B3 — Full Crafting UI `[Hard]`
Complete crafting interface: base selector, affix browser with tier dropdowns, range sliders, catalyst/enchant/corruption buttons. This is a major UI effort.

### B4 — Item Sets `[Medium]`
Support multiple equipment sets with quick switching. Save/load named sets. Weapon swap (I/II).

---

## Quick Reference

### Key Files to Study
| File | What to look at |
|---|---|
| `src-tauri/src/data/mod.rs` | How `GameData` loads at startup — follow this pattern |
| `src-tauri/src/data/tree.rs` | Example of complex JSON deserialization with serde |
| `src-tauri/src/modifier/types.rs` | `ModFlag`, `KeywordFlag` bitflags, `Modifier` struct |
| `src-tauri/src/modifier/mod_db.rs` | `ModDBLayers` — where you add the items layer |
| `src-tauri/src/modifier/parser.rs` | `parse_display_text()` — you'll call this for item mod lines |
| `src-tauri/src/lib.rs` | Tauri command registration, `BuildInfo` state |
| `src/components/SkillTree.svelte` | Complex Svelte 5 component example |
| Any `Bases/*.json` file | Base item data shape |
| Any `Uniques/*.json` file | Raw unique text format |
| `ModItem.json` | Mod pool entry shape |

### Common Pitfalls
1. **Forgetting `#[serde(rename_all = "camelCase")]`** — JSON fields are camelCase, Rust conventionally uses snake_case
2. **Not handling `Option` fields** — many JSON fields are absent on some items (use `#[serde(default)]`)
3. **Local mod double-counting** — if you forget to remove local mods from `mod_list`, stats will be wrong
4. **ModMaster.json is an array** — every other Mod file is an object; don't use the same deserialize type
5. **Uniques are raw text** — `serde_json::from_str::<Vec<String>>(...)`, not `Vec<Item>`
6. **Weapon quality applies to physical only** — don't multiply elemental damage by quality
7. **Quality and %increased are additive** — `(1 + quality/100 + inc/100)`, not `(1 + quality/100) * (1 + inc/100)`
