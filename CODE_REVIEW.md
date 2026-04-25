# Rusty Builds — Code Review & Architecture Recommendations

Welcome to Rust! Building a complex, state-heavy application like a Path of Exile build planner is an ambitious and excellent way to learn the language. 

Overall, the codebase is very well structured. You have a solid grasp on Tauri's IPC, state management with `Mutex` and `RwLock`, and separating your data definitions from your UI. Your `PLAN.md` and `DATA_ARCHITECTURE.md` are exemplary—this level of documentation is rare and highly valuable.

Below is a thorough review of the current state of the backend (primarily `src-tauri/src/lib.rs`), focusing on code simplicity, performance, and Rust idioms.

---

## 1. The "Split Borrow" Pattern vs. Cloning (Crucial Rust Concept)

### Finding
Throughout `lib.rs`, there are many instances where large collections are cloned just to be passed into a method on `build.mod_db_layers`.

For example, in `add_gem_to_group`:
```rust
let groups = build.skill_groups.clone();
let active = build.active_gem.clone();
build.mod_db_layers.rebuild_gems(&groups, active.as_ref(), &game);
```

### Why this happens
You likely encountered a compiler error (borrow checker) if you tried to do:
`build.mod_db_layers.rebuild_gems(&build.skill_groups, ...)`

The compiler complains because calling a `&mut self` method on `build.mod_db_layers` mutably borrows the *entire* `build` struct, preventing you from simultaneously passing an immutable reference to another field (`build.skill_groups`) from that same struct.

### The Fix: Destructuring for Split Borrows
Rust understands "split borrows" if you destructure the struct or borrow the fields directly before calling the method.

**Suggested Change:**
```rust
// Dereference the MutexGuard to get mutable access to the underlying fields
let BuildInfo { 
    ref skill_groups, 
    ref active_gem, 
    ref mut mod_db_layers, 
    .. 
} = *build;

// Now the borrow checker knows these are distinct, non-overlapping fields!
mod_db_layers.rebuild_gems(skill_groups, active_gem.as_ref(), &game);
```

**Pros:**
*   **Massive Performance Gain:** Eliminates deep heap allocations (cloning `Vec`s of `SkillGroup` and `Item`s) on every UI click.
*   **Memory Efficiency:** Reduces memory fragmentation and GC pressure.
**Cons:**
*   Slightly more verbose syntax.

---

## 2. Error Handling: From Stringly-Typed to `thiserror`

### Finding
Almost every Tauri command handles errors by mapping to a string:
`.map_err(|e| e.to_string())?`

### Suggested Change
Tauri commands can return a custom Error type as long as it implements `serde::Serialize`. Since you already have `thiserror` in your `Cargo.toml`, you can create a unified `AppError` type.

```rust
use thiserror::Error;
use serde::Serialize;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Lock poisoned")]
    LockError,
    #[error("Game data is still loading")]
    DataLoading,
    #[error("Item not found: {0}")]
    NotFound(String),
}

// Tauri requires errors crossing the IPC boundary to be serializable
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Helper trait to easily convert PoisonErrors from Mutex locking
impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        AppError::LockError
    }
}
```

Now, your commands become much cleaner:
```rust
fn get_skill_groups(build_info: State<Mutex<BuildInfo>>) -> Result<Vec<SkillGroup>, AppError> {
    let build = build_info.lock()?; // ? works automatically now!
    Ok(build.skill_groups.clone())
}
```

**Pros:**
*   **Simplicity:** Removes dozens of `.map_err(|e| e.to_string())` closures.
*   **Extensibility:** Easy to match on specific error types in Rust if needed later.

---

## 3. Fast Hashing vs Cryptographic Hashing

### Finding
You are importing and using `std::collections::{HashMap, HashSet}` in several places (like `BuildSelection::selected_node_ids`), but also importing `rustc_hash::FxHashMap`.

### Reasoning
Rust's default `HashMap` uses `SipHash`, which is designed to prevent Denial of Service (DOS) attacks by randomizing keys. This makes it cryptographically secure but relatively slow. For a local desktop app processing game data and integers (like `u32` node IDs), DOS protection is unnecessary.

### Suggested Change
Use `rustc_hash::FxHashMap` and `FxHashSet` universally across the application for any performance-critical lookups. 

```rust
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct BuildSelection {
    selected_node_ids: FxHashSet<u32>, // Was HashSet
}
```

**Pros:**
*   **Performance:** `FxHash` is significantly faster for small keys (like `u32` or short strings).
**Cons:**
*   None for local desktop apps.

---

## 4. `lib.rs` File Size and Module Organization

### Finding
`lib.rs` is over 1500 lines long. It currently houses the Tauri setup, all IPC command functions, and dozens of data models (`BuildInfo`, `UniqueDetail`, `BaseMods`, etc.).

### Suggested Change
While learning Rust, keeping things in one file reduces friction. But as the app grows, splitting it up will help immensely.

1.  **Extract Commands:** Move all `#[tauri::command]` functions into a `commands.rs` file (or a `commands/` directory).
2.  **Extract State/Models:** Move `BuildInfo`, `BuildStats`, and related frontend-facing DTOs into a `models.rs` or `state.rs` file.

```rust
// In src/commands.rs
use crate::models::BuildInfo;

#[tauri::command]
pub fn equip_item(...) { ... }
```

**Pros:**
*   **Readability:** Easier to navigate the codebase.
*   **Maintainability:** Easier to spot logical boundaries and prevent tight coupling.

---

## 5. Minor Rust Idioms & Elegance

### A. `unwrap_or_default()`
In `get_gem_list`:
```rust
name: gem.display_name.clone().unwrap_or_default(),
```
Because you already filtered out items where `display_name.is_some()`, you can safely `unwrap()`. However, if you want to avoid `unwrap()`, `unwrap_or_default()` is perfectly fine here.

### B. `retain` vs Filter/Collect
In `delete_skill_group`:
```rust
build.skill_groups.retain(|g| g.id != group_id);
```
Excellent use of `Vec::retain`! This is highly idiomatic and mutates the vector in-place without allocating a new one. 

### C. Unnecessary `.into_iter()` on HashSets
In `get_item_classes`:
```rust
let mut result: Vec<String> = classes.into_iter().collect();
```
Since `classes` is owned by the current scope and isn't used again, `into_iter` is correct. However, you can make this slightly cleaner:
```rust
let mut result: Vec<String> = game.bases.values()
    .map(|base| base.item_class.clone())
    .collect::<FxHashSet<_>>() // Dedupes automatically
    .into_iter()
    .collect();
```

### D. Option flattening
In `get_equipped_items`:
```rust
armour: item.armour_data.as_ref().map(|a| a.armour).filter(|&v| v > 0.0),
```
This is excellent, clean Rust. `filter` on an `Option` is exactly the right idiom here.

---

## 6. Tauri Blocking vs Non-Blocking Commands

### Finding
Heavy calculation functions like `compute_stats` run synchronously on the IPC command thread.

### Implication
Tauri commands run on a dedicated thread pool, so they won't freeze the *Rust* backend, but a heavy synchronous calculation might stall the *IPC bridge* if many commands are fired rapidly (e.g., dragging a slider in the UI that updates stats in real-time).

### Suggestion (For Phase 8 / Performance)
If you notice UI stutter during calculations later, you can wrap the calculation in `tokio::task::spawn_blocking`. Because you are using `Arc` and `Mutex`, this is relatively easy to adapt later if needed. For now, keep it synchronous for simplicity until you hit a proven bottleneck.

---

### Summary
The most impactful takeaway here is learning how to do **Split Borrows** by dereferencing/destructuring your `MutexGuard`. Applying that pattern will teach you a lot about how the borrow checker reasons about memory layout, and it will immediately speed up your application by eliminating redundant `.clone()` operations. Great work so far!

---

## 7. Encapsulation: The `compute_stats` Boilerplate

### Finding
Currently, `compute_stats` is a standalone private helper function taking 8 separate arguments. It is called identically in at least 10 different Tauri commands (`update_build_info`, `update_selected_nodes`, `equip_item`, etc.).

```rust
let stats = compute_stats(
    &build.mod_db_layers,
    build.level,
    &build.class,
    build.active_gem.as_ref(),
    &build.skill_groups,
    &build.equipped,
    game,
    &build.selected_nodes,
);
build.stats = stats.clone();
```

### Suggested Change
Because all of these arguments (except `game_data`) belong to `BuildInfo`, this logic belongs as an implementation method on the `BuildInfo` struct itself.

```rust
impl BuildInfo {
    /// Recomputes stats based on current internal state and updates `self.stats`.
    pub fn recalculate_stats(&mut self, game_data: &GameData) {
        let t0 = std::time::Instant::now();
        let result = calc::calculate(
            &self.mod_db_layers,
            self.level,
            &self.class,
            self.active_gem.as_ref(),
            &self.skill_groups,
            &self.equipped,
            game_data,
        );
        
        self.stats = BuildStats {
            total_strength: result.attributes.strength,
            // ... mapping other fields ...
            calc_time_us: t0.elapsed().as_micros() as u32,
        };
    }
}
```

Now, your commands shrink dramatically:
```rust
build.recalculate_stats(game);
Ok(build.stats.clone())
```

**Pros:**
*   **DRY (Don't Repeat Yourself):** Removes roughly 100 lines of repetitive parameter-passing across `lib.rs`.
*   **Encapsulation:** The commands no longer need to know *how* stats are computed, just that they need to tell the build to update itself.

---

## 8. Concurrency: `Mutex` vs `RwLock` for Application State

### Finding
During app initialization, you set up the state like this:
`app.manage(Mutex::new(BuildInfo::default()));`

### Reasoning
A `Mutex` only allows one thread to access the data at a time, regardless of whether it's reading or writing. As your frontend grows, it might fire off multiple asynchronous requests to read data simultaneously (e.g., `get_equipped_items`, `get_skill_groups`, and `get_debug_stats` loading in different UI components). With a `Mutex`, these read requests block each other.

### Suggested Change
Use an `RwLock` (Read-Write Lock) instead of a `Mutex` for `BuildInfo`.

`app.manage(RwLock::new(BuildInfo::default()));`

Commands that only read data (like `get_skill_groups`) use `build_info.read()`, allowing infinite concurrent readers. Commands that mutate data (like `equip_item`) use `build_info.write()`, which ensures exclusive access.

**Pros:**
*   **UI Responsiveness:** Prevents read-heavy frontend components from stalling if they request data at the same time.

---

## 9. Decoupling: The `get_game` Helper Signature

### Finding
Your helper function for extracting the game data specifies a very tight constraint:
```rust
fn get_game<'g>(
    lock: &'g std::sync::RwLockReadGuard<'g, Option<data::GameData>>,
) -> Result<&'g data::GameData, String>
```

### Suggested Change
Functions should take the simplest type they need to do their job. `get_game` doesn't actually care that the data is inside an `RwLockReadGuard`; it only cares that it is an `Option<GameData>`.

```rust
fn get_game(data: &Option<data::GameData>) -> Result<&data::GameData, AppError> {
    data.as_ref().ok_or(AppError::DataLoading)
}
```

Called via:
```rust
let gd_lock = game_data.read().map_err(|e| e.to_string())?;
let game = get_game(&*gd_lock)?; // Deref the guard to get the Option
```

**Pros:**
*   **Flexibility:** If you ever change how `GameData` is stored (e.g., removing the `RwLock` or passing it differently), `get_game` won't break.
*   **Simpler Lifetimes:** Removes the need to explicitly annotate lifetimes, as Rust's lifetime elision rules handle this automatically.

---

## 10. Good Practice Highlight: `OnceLock` for Regex

### Finding
In `get_unique_detail` and `add_unique_to_inventory`, you are using `std::sync::OnceLock` to compile the regular expression exactly once:

```rust
static RANGE_RE: OnceLock<Regex> = OnceLock::new();
let range_re = RANGE_RE.get_or_init(|| { Regex::new(...).unwrap() });
```

### Review
**Excellent work!** 
Compiling a regex is a relatively expensive operation. Putting it in a `OnceLock` ensures it's only compiled the first time the function is called, and cached for every subsequent call. 

Many older Rust tutorials suggest pulling in the external `lazy_static` crate for this, but `std::sync::OnceLock` was stabilized in Rust 1.70, making your approach the most modern, idiomatic, and dependency-light way to handle this pattern in Rust today.

---

## 11. Safer String Replacement with Regex Closures

### Finding
In `add_unique_to_inventory`, you are manually slicing strings and keeping track of indices to substitute values into regex matches:

```rust
let mut substituted = String::new();
let mut last_end = 0usize;
for m in range_re.find_iter(clean) {
    substituted.push_str(&clean[last_end..m.start()]);
    let v = rolls.get(roll_idx).copied().unwrap_or(0.0);
    roll_idx += 1;
    // ... format logic
    last_end = m.end();
}
substituted.push_str(&clean[last_end..]);
```

### Suggested Change
Manual string slicing based on byte indices is error-prone in Rust (especially with Unicode characters). The `regex` crate provides a `replace_all` method that accepts an `FnMut` closure, allowing you to mutate external variables (like `roll_idx`) while it handles the string reconstruction safely and optimally.

```rust
let substituted = range_re.replace_all(clean, |_: &regex::Captures| {
    let v = rolls.get(roll_idx).copied().unwrap_or(0.0);
    roll_idx += 1;
    if v == v.trunc() {
        format!("{:.0}", v)
    } else {
        format!("{}", v)
    }
}).into_owned();
```

**Pros:**
*   **Simplicity & Readability:** Removes all manual index tracking (`last_end`, `m.start()`, `m.end()`).
*   **Safety:** Zero risk of panicking from an out-of-bounds string slice if byte boundaries mismatch.

---

## 12. Consistent Usage of `bitflags!`

### Finding
You are importing `InfluenceSet` (which your `PLAN.md` notes is a `bitflags!` struct). In `item_to_detail`, you correctly use it:
```rust
if inf.contains(InfluenceSet::SHAPER) { ... }
```
However, in `get_unique_detail`, you manually parse the bits from a `u8`:
```rust
let bits = def.influences;
if bits & 0b0000_0001 != 0 { inf.push("Shaper".to_string()); }
```

### Suggested Change
If `def.influences` is coming across as a `u8`, convert it to an `InfluenceSet` immediately using the `bitflags` crate's `from_bits_truncate` method.

```rust
let inf_set = InfluenceSet::from_bits_truncate(def.influences);
if inf_set.contains(InfluenceSet::SHAPER) { inf.push("Shaper".to_string()); }
// ...
```

**Pros:**
*   **Maintainability:** If the bit layout ever changes (e.g., Shaper becomes bit 3), you only change the `bitflags!` definition, and `get_unique_detail` won't break.

---

## 13. Architectural Note: "Fear of Cloning" and Tauri IPC DTOs

### Finding
Throughout functions like `get_base_categories`, `search_uniques`, and `get_inventory_items`, you frequently use `.clone()` to copy strings out of the game data and into DTOs (Data Transfer Objects) like `UniqueSearchResult` and `EquippedItemSummary`.

### Educational Note
Because I pointed out cloning as a performance issue in Section 1 (the Split Borrow pattern), you might wonder if cloning Strings into these return structs is also "bad Rust." 

**It is absolutely correct and necessary here.** 

Tauri commands are sent across an asynchronous IPC boundary. To safely transfer data to the frontend, Tauri requires the return type to own its data (`'static` lifetime) so it can serialize it to JSON. You cannot return references (`&str`) to data locked inside your `RwLock<GameData>`. 

Therefore:
*   Cloning in the deep **calculation pipeline** (`ModDB`, resolving stats) = **Bad** (slows down the core engine).
*   Cloning at the **boundary** to build IPC responses = **Good** (necessary to bridge Rust's memory to the webview). Your code currently handles this boundary perfectly.

---

## 14. "Fat Controllers": Single Responsibility Principle in Tauri Commands

### Finding
Some of your Tauri commands are extremely large. For example, `add_unique_to_inventory` (approx. 90 lines) mixes several completely different concerns:
1.  Tauri state extraction & Mutex locking
2.  Game data lookups
3.  Complex String manipulation (Regex replacement)
4.  Reconstructing a simulated PoE text block
5.  Calling the parser
6.  Mutating the inventory and generating an ID
7.  Triggering a recalculation

### Reasoning
Tauri commands are essentially HTTP Controllers in a web backend. If you put business logic inside the command, you **cannot test that logic** without spinning up a full Tauri application context. 

### Suggested Change
Keep Tauri commands "thin". Extract the core logic into pure Rust functions that take normal arguments and return `Result<Item, AppError>`.

```rust
// In your domain logic (e.g. `item/builder.rs`):
pub fn build_unique_with_rolls(def: &UniqueItemDef, rolls: &[f64], game: &GameData) -> Result<Item, AppError> {
    // ... all the regex and string building logic ...
    // ... call the parser ...
    Ok(item)
}
```

```rust
// In lib.rs (The "Thin" Command):
#[tauri::command]
fn add_unique_to_inventory(
    name: String,
    rolls: Vec<f64>,
    build_info: State<Mutex<BuildInfo>>,
    game_data: State<GdState>,
) -> Result<BuildStats, AppError> {
    let game_lock = game_data.read()?;
    let game = get_game(&game_lock)?;
    let def = game.uniques.iter().find(|u| u.name == name).ok_or(AppError::NotFound)?;
    
    // 1. Call pure domain logic
    let item = build_unique_with_rolls(def, &rolls, game)?;
    
    // 2. Lock state and mutate
    let mut build = build_info.lock()?;
    build.add_item_to_inventory(item); // See section 15
    build.recalculate_stats(game);     // See section 7
    
    Ok(build.stats.clone())
}
```
**Pros:**
*   **Testability:** You can now write `#[test]` unit tests for `build_unique_with_rolls` completely isolated from Tauri.
*   **Readability:** The command clearly reads as: "Get dependencies -> Build item -> Add to inventory -> Recalculate stats".

---

## 15. Encapsulation of Inventory ID Generation

### Finding
In `equip_item`, `add_unique_to_inventory`, and `add_crafted_item`, the following logic is repeated verbatim:
```rust
build.next_item_id += 1;
item.inventory_id = build.next_item_id;
build.inventory.push(item);
```

### Suggested Change
Any time you have paired mutations (incrementing a sequence AND assigning it), encapsulate it in a method to prevent bugs where an item is added but the ID wasn't assigned.

```rust
impl BuildInfo {
    pub fn add_item_to_inventory(&mut self, mut item: Item) {
        self.next_item_id += 1;
        item.inventory_id = self.next_item_id;
        self.inventory.push(item);
    }
}
```

**Pros:**
*   Removes repetition and guarantees that every item pushed to the inventory gets a valid, incremented ID.

---

## 16. Using `#[serde(flatten)]` to Delete Boilerplate DTO Mapping

### Finding
In `item_to_detail`, you have massive `if let Some(wd) = &item.weapon_data` blocks that extract internal properties (like `phys_min`) into huge tuples, just to inject them back into `ItemDetail`'s flat structure.

### Reasoning
The `serde` crate (which handles JSON serialization) has a superpower: `#[serde(flatten)]`. It allows you to take nested structs in Rust and "flatten" them into the parent JSON object automatically during serialization.

### Suggested Change
Change your `ItemDetail` struct to directly hold `Option<WeaponData>` and `Option<ArmourData>` but tell Serde to flatten them:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemDetail {
    pub name: String,
    pub item_class: String,
    // ... standard fields ...
    
    #[serde(flatten)]
    pub weapon_data: Option<WeaponData>,
    
    #[serde(flatten)]
    pub armour_data: Option<ArmourData>,
}
```

Because of `#[serde(flatten)]`, the resulting JSON sent to the frontend will still look exactly like this:
`{ "name": "...", "phys_min": 10.0, "phys_max": 20.0 }`

**Pros:**
*   **Massive Code Deletion:** You can completely delete the ~60 lines of manual tuple unpacking in `item_to_detail`. You just assign `weapon_data: item.weapon_data.clone()`.

---

## 17. Background Thread Safety during App Setup

### Finding
In `tauri::Builder::setup` (around line 345), you spawn a background thread to load the 5MB passive tree JSON so the UI window can appear instantly. Inside that thread, you use `.unwrap()` and `.expect()`:

```rust
let mut bi = build_state.lock().expect("BuildInfo mutex poisoned");
// ...
*gd_state.write().unwrap() = Some(gd);
```

### Reasoning
If `GameData::load_with_progress` fails (e.g., the JSON file is missing or corrupted), the Rust thread will log the error and `return`. However, the main Tauri app *keeps running*. The frontend will be stuck in a permanent "Loading..." state because the data will never arrive, and the backend will silently fail every future command with "Game data is still loading".

### Suggested Change
If an error occurs in the background thread, use `handle.emit("loading_error", e.to_string())` before returning. On the frontend, listen for `loading_error` and display a clear UI message (e.g., "Corrupted game files. Please reinstall"). This provides a much better user experience than an infinite loading spinner.

---

## 18. IPC Bottleneck: The "Fat Payload" Anti-Pattern (CQRS)

### Finding
Currently, almost every mutation command in `lib.rs` (like `update_selected_nodes`, `equip_item`, `add_gem_to_group`) triggers a full recalculation and returns the *entire* `BuildStats` object across the IPC boundary.

```rust
let stats = compute_stats(...);
build.stats = stats.clone();
Ok(stats) // Serializes the entire state and sends it to JS
```

### Reasoning
As your app scales to Phase 5, `BuildStats` is going to become `CalcResult`—a massive, deeply nested struct containing every single offense, defense, and configuration stat in Path of Exile. Serializing this to JSON and passing it across the Tauri bridge every time the user clicks a tree node (or drags a slider) is the exact cause of your IPC performance bottleneck.

### Suggested Change: CQRS (Command Query Responsibility Segregation)
Separate your **Commands** (which mutate state) from your **Queries** (which read state). 

A mutation should only return a success signal:
```rust
#[tauri::command]
fn update_selected_nodes(...) -> Result<(), AppError> {
    // ... mutate state and recalculate ...
    Ok(()) // Return nothing! Just a success signal.
}
```

Then, let the frontend UI components explicitly ask for *only* the data they need to render:
```rust
#[tauri::command]
fn get_sidebar_stats(...) -> Result<SidebarStatsSummary, AppError> { ... }

#[tauri::command]
fn get_calcs_tab_stats(...) -> Result<DetailedCalcResult, AppError> { ... }
```

**Pros:**
*   **Massive IPC Performance Boost:** You stop serializing 5MB of calculation breakdown data when the user is only looking at the Sidebar.
*   **UI Component Isolation:** Svelte components fetch exactly what they need, making the frontend highly modular.

---

## 19. ModDB Inner Loop Complexity: The `ModTag` Bottleneck

### Finding
In `modifier/mod_db.rs`, the core query methods (`sum_base`, `sum_inc`, etc.) are doing heavy lifting inside their iterator closures. 

For every single modifier in the `FxHashMap`, they call `matches_context()` and `effective_value()`, which both contain `for tag in &modifier.tags` loops evaluating complex `match` statements against the `CalcContext`.

### Reasoning
The `calculate()` function will be called hundreds of times per frame during the Phase 5 DPS pipeline (especially with recursive damage conversion). If you have 200 `Life` modifiers, `sum_base` evaluates `matches_context` 200 times. That means evaluating `mod.tags` 200 times. 

This creates a massive "hot-path" branching penalty for the CPU.

### Suggested Change: "Pre-compiled" Active ModDB
When the `CalcContext` changes (e.g., the user toggles "Low Life" in the config tab), you should do a one-time "flattening" of the `ModDBLayers::merged()` database into an `ActiveModDB`.

An `ActiveModDB` evaluates all the `Condition` and `Multiplier` tags *once* upon creation, dropping modifiers that don't match the context, and baking the multipliers directly into the `value` field.

Then, your core query methods become blindingly fast:
```rust
impl ActiveModDB {
    pub fn sum_base(&self, stat: StatId) -> f64 {
        // No context checking! No tag loops! Just pure contiguous math.
        self.mods.get(&stat)
            .map(|mods| mods.iter().filter(|m| m.mod_type == ModType::Base).map(|m| m.value).sum())
            .unwrap_or(0.0)
    }
}
```

**Pros:**
*   **Engine Performance:** Removes nested loops and branch prediction failures from the absolute deepest, hottest part of the calculation engine.

---

## 20. Avoiding Strings in the Core Pipeline

### Finding
In `DATA_ARCHITECTURE.md` and `lib.rs`, there is a translation chain where raw stat IDs (e.g., `"attack_speed_+%"`) are passed around as `String`s from the `GameData`, through the gem state, until they hit `parser::resolve()` which looks them up in the `SkillStatMapDB`.

### Reasoning
Strings require heap allocation. Passing, cloning, and hashing `String`s inside the calculation or layer-rebuilding pipeline is inherently slow. Your `PLAN.md` mentions a "Pre-Resolution Cache" in Phase 3.14e—this is an absolutely critical architectural goal.

### Suggested Change
Ensure that the `String` representation of a stat *dies immediately* upon app startup. 

When `GameData::load_from_dir` reads `Default.json` (the RePoE passive tree), it should immediately convert the `{"base_strength": 10}` string keys into your parsed `Vec<Modifier>` representation and store *that* in memory. 

The runtime `RePoETree` should look like this:
```rust
pub struct RePoEPassive {
    pub node_id: u32,
    pub pre_resolved_mods: Vec<Modifier>, // No strings here!
}
```

Then, `rebuild_tree` becomes a simple, zero-allocation memory copy:
```rust
for &node_id in node_ids {
    if let Some(passive) = game.repoe_tree.get_passive(node_id) {
        for m in &passive.pre_resolved_mods {
            self.tree.add_mod(m.clone());
        }
    }
}
```

---

---

# Analysis of Findings

This section critically evaluates each point in the review against the actual codebase, rates validity, and weighs the trade-offs of implementing each fix.

---

## #1 — Split Borrows ✅ Valid, worth doing selectively

**Verified:** `skill_groups.clone()` appears at 6 real call sites (lines 793, 833, 877, 935, 980, and one already commented out at 612). The finding is real.

**Validity:** High. The borrow checker problem is correctly diagnosed, and the destructuring fix is the standard Rust solution.

**Caveats:**
- The reviewer calls this a "massive performance gain" which is overstated. A typical build has ~5 skill groups with a few gems each — the clone is fast. The gain is real but modest, not massive.
- The destructuring syntax (`let BuildInfo { ref skill_groups, ref mut mod_db_layers, .. } = *build;`) works but becomes fragile if new fields are added to `BuildInfo`. Every new field needs to be accounted for in the destructuring pattern.
- A simpler alternative: extract the necessary data *before* acquiring the mutable borrow, which is already done in the commented-out line 612 pattern.
- **Best approach:** Apply only in the commands that call `compute_stats` (the actual hot path), not universally.

**Recommendation:** Implement, but narrow scope to the commands that trigger recalculation.

---

## #2 — `AppError` with `thiserror` ✅ Valid

**Verified:** `map_err(|e| e.to_string())` appears 20+ times across commands in `lib.rs`.

**Validity:** High. The pattern is correct — `impl Serialize for AppError` that serializes to string is the standard Tauri approach, and it makes `?` work on `Mutex::lock()` via the `From<PoisonError>` impl.

**Caveats:**
- Changing all command return types from `Result<T, String>` to `Result<T, AppError>` triggers tauri-specta to regenerate `bindings.ts`. Since `AppError` serializes identically to `String` at the IPC boundary, the TypeScript type stays `string` — no breaking change.
- The main concrete benefit is ergonomics: `build_info.lock()?` just works. This removes a lot of noise.
- Requires touching every command signature, which is a wide but mechanical change.

**Recommendation:** Implement. The ergonomic gain is real and the risk is low.

---

## #3 — FxHashMap universally ✅ Valid but largely already done

**Verified:** `grep` for `std::collections::HashMap` in `lib.rs` returns **zero matches** — `lib.rs` is already clean. The reviewer may have found instances in other files.

**Validity:** Medium. The project convention (copilot instructions) already mandates `FxHashMap` everywhere. This is about catching drift, not a fundamental architectural issue.

**Caveats:**
- Worth running a project-wide check, but if `lib.rs` is already clean the remaining instances are likely in less critical paths.
- The performance difference only matters in hot loops. In one-time setup code or small maps, it's irrelevant.

**Recommendation:** Do a project-wide sweep — it takes 15 minutes and maintains the convention.

---

## #4 — Extract `lib.rs` into modules ✅ Valid, low urgency for a solo project

**Verified:** `lib.rs` exceeds 2,500 lines. `commands.rs` already exists as an empty placeholder stub.

**Validity:** High for maintainability. Navigation is already painful at this size.

**Caveats:**
- All commands share the same state types (`BuildInfo`, `GdState`, etc.), so extraction requires careful re-exports. Not difficult, just tedious.
- For a solo project there is no team coordination cost from a large file — the benefit is purely navigational. With good IDE tooling this is less critical.
- Should be done *after* #7 and #14 (which will clean up the commands themselves), otherwise you're moving messy code into a new file.

**Recommendation:** Defer until after #7 and #14 are done. Then extract commands to `commands/` as a cleanup step.

---

## #5 — Minor Idioms — Mixed

- **A (`unwrap_or_default`):** The reviewer acknowledges this is fine. No action needed.
- **B (`retain`):** Reviewer confirms this is already idiomatic. No action.
- **C (`into_iter` refactor):** The suggested refactor (intermediate `collect::<FxHashSet<_>>()`) is *less* readable than the existing code — it adds an extra allocation pass and is harder to scan. The original is correct. **Reject this suggestion.**
- **D (Option flattening):** Reviewer confirms existing code is already the right idiom. No action.

---

## #6 — `spawn_blocking` for heavy commands ⚠️ Valid concern, correctly deferred

**Validity:** The underlying concern is correct — synchronous heavy work on the IPC thread pool can cause backpressure. However, Tauri's async command system already runs commands on a thread pool (not the main thread), so the UI doesn't freeze. The risk only materialises if many commands fire simultaneously and all block on the same `Mutex`.

**Recommendation:** Defer to Phase 8 as the reviewer suggests. Only act on profiling evidence.

---

## #7 — `recalculate_stats` method on `BuildInfo` ✅ Highly valid, high priority

**Verified:** `compute_stats(` is called **12 times** in `lib.rs`, each time passing the same 8 arguments. Each call site has the same 8-line boilerplate.

**Validity:** Very high. This is the single most impactful cleanup available. A `build.recalculate_stats(game)` method collapses 12 × 8-line call sites to 12 single-line calls — removing ~80 lines of repeated parameter-passing.

**Caveats:**
- `selected_nodes` is only used for `node_count` in the result — since `BuildInfo` owns it, this is trivially accessible as `self.selected_nodes` inside the method.
- The only external dependency is `game_data: &GameData`, which is correctly not part of `BuildInfo` (it lives in a separate `RwLock`).
- This change also unblocks #1 (split borrows are less necessary once the call sites are simplified) and is a prerequisite for #14.

**Recommendation:** Implement early. This is the highest-value, lowest-risk change in the review.

---

## #8 — `Mutex` → `RwLock` for `BuildInfo` ⚠️ Questionable priority

**Validity:** The concurrent-reader scenario the reviewer describes is unlikely in practice:
- The Svelte frontend runs in a single JS event loop — it `await`s each IPC call sequentially.
- Even if two components mounted simultaneously (e.g., Sidebar + CalcTab both requesting data on `onMount`), Tauri serialises those calls before they reach the Rust handler.
- Virtually every command that reads `BuildInfo` also *writes* it (to update `build.stats` after recalculation), so they all need write access anyway.

**Caveats:**
- Switching to `RwLock` means distinguishing read vs write locks across every command — adding surface area for bugs (e.g., accidentally taking a write lock where a read suffices, or deadlocking if a read lock is held and a write is requested on the same thread).
- `RwLock` write-lock acquisition is slightly slower than `Mutex` lock acquisition when there are no concurrent readers, which is always the case here.
- If Phase 5 introduces a live-update slider that fires commands rapidly, this becomes worth revisiting.

**Recommendation:** Do not implement now. Revisit with profiling data in Phase 5.

---

## #9 — Simpler `get_game` signature ✅ Valid, trivial

**Validity:** High. The current lifetime annotation is unnecessarily restrictive. The function only needs `&Option<GameData>` — it doesn't care that it came from a `RwLockReadGuard`. The simpler signature also works correctly with lifetime elision.

**Caveats:** None. Pure improvement.

**Recommendation:** Implement alongside #2 (the `AppError` change), since `get_game` would return `AppError` instead of `String` after that change.

---

## #10 — `OnceLock` for Regex — Already correct, no action

The reviewer confirms the existing approach is modern and idiomatic. No changes needed.

---

## #11 — `Regex::replace_all` closure ✅ Valid, mild priority

**Validity:** The reviewer's "Unicode panic" concern is theoretically valid but practically irrelevant here — PoE stat strings are guaranteed ASCII. The real benefit is readability: replacing 10 lines of manual index tracking with a single `replace_all` closure.

**Caveats:**
- The `replace_all` closure approach requires the closure to be `FnMut`, which it already is (capturing `roll_idx` by mutable reference). This works correctly.
- The existing manual approach is not wrong, just noisier.

**Recommendation:** Implement as a readability improvement. Low risk.

---

## #12 — `InfluenceSet` consistency ✅ Valid, easy fix

**Verified:** Lines 1434–1449 in `lib.rs` use raw `bits & 0b...` masking while `item_to_detail` correctly uses `InfluenceSet::contains`. The inconsistency is real.

**Validity:** High. If the bit layout ever changes (e.g., a new influence type is added between existing bits), the raw masking silently breaks while the `bitflags!` version stays correct.

**Caveats:** None. `InfluenceSet::from_bits_truncate(def.influences)` is a one-line drop-in replacement.

**Recommendation:** Implement immediately — it's 10 minutes and genuinely improves correctness.

---

## #13 — IPC boundary cloning — Already correct, no action

The reviewer confirms that cloning strings into IPC DTOs is correct and necessary. No changes.

---

## #14 — Thin controllers / single responsibility ✅ Valid, apply selectively

**Validity:** High for complex commands like `add_unique_to_inventory` (regex, string building, parser calls, inventory mutation, recalculation all in one function). The testability argument is real — you cannot currently test the regex-substitution logic without a full Tauri context.

**Caveats:**
- Simpler commands (`get_skill_groups`, `equip_item`) don't need splitting — they are already thin enough to read at a glance.
- Extracting to `item/builder.rs` fits the existing module structure cleanly.
- This is a prerequisite for #4 (extracting to `commands/`) to be worthwhile — don't move fat commands into a new file, clean them first.
- Requires #7 to be done first, since `build.recalculate_stats(game)` makes the command body much smaller to begin with.

**Recommendation:** Apply to the two or three largest commands after #7 is done. Don't apply uniformly to all commands.

---

## #15 — Encapsulate inventory ID generation ✅ Valid, easy win

**Verified:** The `next_item_id += 1; item.inventory_id = ...; inventory.push(item)` pattern appears verbatim at 3 call sites (lines 1104–1106, 1669–1671, 1844–1846).

**Validity:** High. The pairing of ID increment and assignment is an invariant that belongs in one place. If a future developer adds an item to inventory without using the helper, the bug is silent — the item gets `inventory_id = 0`.

**Caveats:** None. This is an unambiguous improvement.

**Recommendation:** Implement immediately.

---

## #16 — `#[serde(flatten)]` for `ItemDetail` ⚠️ Clever but verify compatibility first

**Validity:** The idea is sound — `#[serde(flatten)]` would eliminate ~60 lines of manual field mapping in `item_to_detail`. The serialised JSON output would be identical.

**Caveats:**
- **`specta::Type` compatibility is unknown.** `tauri-specta` uses specta to generate TypeScript types. `#[serde(flatten)]` on `Option<WeaponData>` may or may not be handled correctly — specta's support for flattened optionals has historically been incomplete. If it doesn't work, the TypeScript type in `bindings.ts` would be wrong or the build would fail.
- If specta does support it, all `WeaponData` fields become optional top-level fields on `ItemDetail` in TypeScript — which is what the frontend already expects.
- Worth testing in a branch before committing.

**Recommendation:** Investigate specta compatibility before implementing. If specta handles it, implement. If not, skip — the manual mapping is verbose but correct.

---

## #17 — Background thread error event ✅ Valid UX fix

**Validity:** High. Currently if `data.json` is missing or corrupted, the app silently spins on the loading overlay forever with no user-visible feedback. Emitting a `loading_error` event and handling it on the frontend is the correct Tauri pattern.

**Caveats:** None. This is a pure UX improvement with no downside.

**Recommendation:** Implement. It's 30 minutes of work and meaningfully improves the error experience.

---

## #18 — CQRS (mutations return `()`, separate query commands) ⚠️ Correct direction, premature now

**Validity:** The reviewer is right that returning full `BuildStats` from every mutation will become a problem in Phase 5 when `BuildStats` grows into a large nested `CalcResult`. However:
- The current `BuildStats` is small and cheap to serialise.
- The entire frontend state model (`buildState.svelte.ts`) is built around receiving stats in command responses — switching to a pull model requires rewriting all state management.
- Tauri's IPC is not an HTTP API with hundreds of concurrent clients; it's a local pipe. Serialising a moderate struct on each click is not the bottleneck for current Phase 3–4 scope.

**Recommendation:** Defer until Phase 5 when `CalcResult` actually becomes large. Design the new query commands at that point rather than prematurely splitting now.

---

## #19 — Pre-compiled `ActiveModDB` ⚠️ Valid optimization, Phase 5+ only

**Validity:** The reviewer correctly identifies `matches_context()` + tag evaluation inside the inner loop as the hottest code path. Pre-flattening for a fixed context would eliminate that branching.

**Caveats:**
- The `CalcContext` is **not** fixed per-session — it changes per skill (SPELL vs ATTACK flags), per configuration option, and recursively during damage conversion. A single pre-compiled `ActiveModDB` cannot cover all these contexts simultaneously.
- A more realistic design: pre-compile per unique `CalcContext` with an LRU cache, invalidated when tree/items/gems change. This is non-trivial.
- The current implementation is fast enough for Phases 3–4; benchmarks should drive this decision.

**Recommendation:** Defer to Phase 5/8. Profile before implementing.

---

## #20 — Eliminate strings from the hot path ✅ Valid, partially already done

**Validity:** High. Strings in the calculation pipeline cause heap allocations and hash overhead.

**Caveats:**
- The gem layer already pre-resolves stats at load time via `RePoEGem.resolved_stats` — this is done.
- The passive tree layer (`rebuild_tree`) already uses pre-resolved `Vec<Modifier>` via `PassiveNode` — also done.
- The remaining string-heavy path is likely the modifier text parsing pipeline (stat descriptions → `ModLine::raw_stats`), which runs only at item-equip time, not during calculations.
- This is largely already addressed by the existing pre-resolution architecture.

**Recommendation:** No immediate action — verify whether any string lookups survive into `calc::calculate()`. If they do, address them in Phase 3 as planned.

---

---

# Recommended Implementation Order

Grouped by dependency and effort. Each group can be done independently; items within a group should be done in order.

### Group A — Quick wins (< 1 hour total, do now)
1. **#15** — `add_item_to_inventory` method on `BuildInfo` *(prevents silent inventory ID bugs, 20 min)*
2. **#12** — Replace raw `bits & 0b...` with `InfluenceSet::from_bits_truncate` *(correctness, 10 min)*
3. **#9** — Simplify `get_game` signature to take `&Option<GameData>` *(10 min, prep for #2)*
4. **#3** — Project-wide sweep for any remaining `std::collections::HashMap` *(15 min)*

### Group B — Code quality (2–4 hours, next milestone)
5. **#11** — Refactor string substitution in `add_unique_to_inventory` to use `regex::replace_all` *(30 min)*
6. **#17** — Emit `loading_error` Tauri event from background load thread + frontend handler *(30 min)*
7. **#2** — Add `AppError` type with `impl Serialize + From<PoisonError>` *(2 hours, touches all command signatures — do in one commit)*

### Group C — Architecture (4–8 hours, follow-on)
8. **#7** — Move `compute_stats` logic into `BuildInfo::recalculate_stats(&mut self, game: &GameData)` *(2 hours, largest single DRY improvement)*
9. **#1** — Apply split-borrow destructuring in the 4–5 commands that call `recalculate_stats` most frequently *(1 hour, easier after #7 reduces surrounding code)*
10. **#14** — Extract large command bodies (primarily `add_unique_to_inventory`) into pure functions in `item/builder.rs` *(2 hours, depends on #7 being done first)*
11. **#4** — Move all `#[tauri::command]` functions to `commands/` module *(2 hours, do after #14 to move clean code)*

### Group D — Investigate before deciding
12. **#16** — Test `#[serde(flatten)]` on `ItemDetail` in a branch; implement only if specta generates correct TypeScript types.

### Group E — Defer to Phase 5+
13. **#18** — CQRS separation of mutation and query commands *(when `CalcResult` grows large in Phase 5)*
14. **#19** — Pre-compiled `ActiveModDB` per `CalcContext` *(when benchmarks justify it)*
15. **#8** — Switch `BuildInfo` from `Mutex` to `RwLock` *(only if profiling shows lock contention)*

### Do not implement
- **#5C** — The suggested `collect::<FxHashSet<_>>()` refactor is less readable than the existing code.
- **#6** — `spawn_blocking` (reviewer explicitly defers this; Tauri's thread pool already handles it adequately).
