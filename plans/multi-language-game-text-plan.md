# Description
Set up a simple, hand-built internationalization system with a central text module accessible via dot notation (e.g., `game_text.menu().story_option`). The system will support multiple languages starting with English, with a language setting on the service. `GameText` is a singleton in the DI container (like `Config`), text structs use `&'static str`, and text is organized by screen/scene.

# TODO
- [ ] Create `src/text/` module structure with submodules for each language
- [ ] Define `Language` enum with `Default` impl returning `English`
- [ ] Define text data structs for each screen: `MenuText`, `HudText`, `LoadingText`, `StatsText`, `CommonText`
- [ ] Implement English text providers in `src/text/en/`
- [ ] Define `GameText` service with language setting and accessor methods
- [ ] Wire `GameText` as `Rc<GameText>` into `DiContainer` and `SceneFactory`
- [ ] Migrate existing hardcoded strings to use `GameText`

# TODO Explanation
## Item 1: Create text module structure
Create `src/text/mod.rs` as the module root (defines `Language`, `GameText`, and all text structs). Create `src/text/en/mod.rs` and submodules `src/text/en/menu.rs`, `src/text/en/hud.rs`, `src/text/en/loading.rs`, `src/text/en/stats.rs`, `src/text/en/common.rs` for English text providers.

## Item 2: Define Language enum
`Language` enum with `English` variant and a `Default` impl returning `Language::English`. Lives in `src/text/mod.rs`.

## Item 3: Define text data structs
One struct per screen/scene, each with `&'static str` fields for every hardcoded string found in that screen. Structs are returned by `GameText` accessor methods and enable dot notation.

## Item 4: Implement English text providers
Each `src/text/en/<screen>.rs` file exports a function that returns the corresponding text struct populated with English string literals.

## Item 5: Define GameText service
`GameText` holds `language: Language` and dispatches to the correct language provider module based on the current language. Each accessor method (e.g., `menu()`) returns the text struct for the active language.

## Item 6: Wire GameText into DI
Add `game_text: Rc<GameText>` to `DiContainer` (created with `Language::default()`). Add `game_text()` accessor. Pass `Rc<GameText>` through `SceneFactory::create()` into each scene constructor. Scenes pass it to systems that need text.

## Item 7: Migrate existing hardcoded strings
Replace hardcoded strings in the codebase with `game_text` lookups:
- `sys_menu_input.rs`: `OPTIONS` array → `game_text.menu().story_option` etc.
- `sys_hud.rs`: `"HP"`, `"MP"` → `game_text.hud().hp_label`, `game_text.hud().mp_label`
- `src/scene/mod.rs`: `"Loading..."` → `game_text.loading().text`
- `hero/components.rs`: `CoreStat::name()`/`description()` → `game_text.stats().*`
- `hero/components.rs`: `HeroStats` default name → `game_text.stats().knight_name`
- `enemy/components.rs`: `EnemyStats` default name → `game_text.stats().ram_head_name`
- `config.rs`: window title → `game_text.common().window_title`

# Objects
## Language (new)
Enum representing supported languages. Lives in `src/text/mod.rs`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
}

impl Default for Language {
    fn default() -> Self { Language::English }
}
```

## MenuText (new)
Text strings for the menu screen (4 options). Lives in `src/text/mod.rs`.

```rust
pub struct MenuText {
    pub story_option: &'static str,         // "STORY"
    pub arcade_option: &'static str,        // "ARCADE"
    pub achievements_option: &'static str,  // "ACHIEVEMENTS"
    pub options_option: &'static str,       // "OPTIONS"
}
```

## HudText (new)
Text strings for the battle HUD bar labels. Lives in `src/text/mod.rs`.

```rust
pub struct HudText {
    pub hp_label: &'static str,  // "HP"
    pub mp_label: &'static str,  // "MP"
}
```

## LoadingText (new)
Text strings for the loading screen. Lives in `src/text/mod.rs`.

```rust
pub struct LoadingText {
    pub text: &'static str,  // "Loading..."
}
```

## StatsText (new)
Text strings for hero/enemy stat names, descriptions, and default entity names. Lives in `src/text/mod.rs`.

```rust
pub struct StatsText {
    pub hp_name: &'static str,         // "Health Points (HP)"
    pub mp_name: &'static str,         // "Magic Points (MP)"
    pub str_name: &'static str,        // "Strength (STR)"
    pub arm_name: &'static str,        // "Armor (ARM)"
    pub int_name: &'static str,        // "Intelligence (INT)"
    pub eva_name: &'static str,        // "Evasion (EVA)"
    pub acc_name: &'static str,        // "Accuracy (ACC)"
    pub ctr_name: &'static str,        // "Critical Strike (CTR)"
    pub hp_description: &'static str,  // "Determines the maximum amount of damage..."
    pub mp_description: &'static str,  // "A universal resource pool..."
    pub str_description: &'static str, // "Increases physical damage..."
    pub arm_description: &'static str, // "Mitigates all incoming damage..."
    pub int_description: &'static str, // "Increases magical power..."
    pub eva_description: &'static str, // "Increases the chances of dodging..."
    pub acc_description: &'static str, // "Determines the likelihood..."
    pub ctr_description: &'static str, // "The percentage chance for an attack..."
    pub knight_name: &'static str,     // "Knight"
    pub ram_head_name: &'static str,   // "Ram Head"
}
```

## CommonText (new)
Shared text strings not tied to a specific screen. Lives in `src/text/mod.rs`.

```rust
pub struct CommonText {
    pub window_title: &'static str,  // "Heroes of Ironhold"
}
```

## GameText (new)
Main text service. Holds the current language and provides access to text groups. Lives in `src/text/mod.rs`.

```rust
pub struct GameText {
    language: Language,
}
```

## DiContainer (modify)
Add `game_text` field. Lives in `src/di/mod.rs`.

```rust
pub struct DiContainer {
    // ... existing fields ...
    game_text: Rc<GameText>,  // what it holds
}
```

# Services
## GameText (new)
Main text service providing access to all game text via dot notation. Lives in `src/text/mod.rs`.

```rust
impl GameText {
    /// Create a new GameText with the given language
    pub fn new(language: Language) -> Self { ... }

    /// Get menu text for the current language
    pub fn menu(&self) -> MenuText { ... }

    /// Get HUD text for the current language
    pub fn hud(&self) -> HudText { ... }

    /// Get loading screen text for the current language
    pub fn loading(&self) -> LoadingText { ... }

    /// Get stat names/descriptions for the current language
    pub fn stats(&self) -> StatsText { ... }

    /// Get common/shared text for the current language
    pub fn common(&self) -> CommonText { ... }

    /// Set the current language
    pub fn set_language(&mut self, language: Language) { ... }

    /// Get the current language
    pub fn language(&self) -> Language { ... }
}
```

## DiContainer (modify)
Add `game_text()` accessor. Lives in `src/di/mod.rs`.

```rust
impl DiContainer {
    // ... existing methods ...

    /// Returns a clone of the Rc<GameText> singleton
    pub fn game_text(&self) -> Rc<GameText> { ... }
}
```

## SceneFactory (modify)
Pass `game_text` to scenes. Lives in `src/scene/factory.rs`.

```rust
impl SceneFactory {
    // ... existing methods ...

    /// Creates a scene by id, passing game_text along with other DI services
    pub fn create(&self, id: SceneId) -> Box<dyn Scene> { ... }
}
```

## English text providers (new)
Functions that return text structs with English strings. Live in `src/text/en/` submodules.

```rust
// src/text/en/menu.rs
pub fn menu_text() -> MenuText { ... }

// src/text/en/hud.rs
pub fn hud_text() -> HudText { ... }

// src/text/en/loading.rs
pub fn loading_text() -> LoadingText { ... }

// src/text/en/stats.rs
pub fn stats_text() -> StatsText { ... }

// src/text/en/common.rs
pub fn common_text() -> CommonText { ... }
```

# Open Questions
None.

# Out of Scope
- Runtime language switching UI
- Loading text from external files (JSON, YAML, etc.)
- Pluralization or text interpolation (dynamic values like `format!("{}/{}", hp, max_hp)` stay at the call site)
- Right-to-left language support
- Text formatting (bold, italic, etc.)
- Migrating key cap labels (`'a'` in `sys_skill.rs`) — these are input bindings, not translatable text
- Migrating dynamic floating text (`+10`, `-5` damage numbers) — these are computed at runtime
