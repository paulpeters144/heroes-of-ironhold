# System Wiring Rules

- Every system is registered with the scene's `SystemAgg` and run through it. The scene's `update` / `draw` / `draw_ui` methods do nothing but delegate to the aggregate. Never hold a system in a scene field and never call a system's methods directly.

## Do: register with the aggregate, delegate from the scene

```rust
pub struct BattleTestScene {
    // scene state (cfg, assets, store, bus) and the aggregate — no system fields.
    agg: SystemAgg,
    // ...
}

impl BattleTestScene {
    pub fn new(cfg: Rc<Config>, assets: Assets, store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let agg = SystemAgg::new();
        agg.add_update(KnightControlSystem::new(store.clone()));
        agg.add_update(RamHeadAiSystem::new(store.clone(), bus.clone()));
        // ...
        Self { agg, /* ... */ }
    }
}

impl Scene for BattleTestScene {
    fn update(&mut self, ctx: &mut Context) {
        self.agg.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.agg.draw(ctx);
    }

    fn draw_ui(&self, ctx: &Context) {
        self.agg.draw_ui(ctx);
    }
}
```

A system that runs in both update and draw (or needs the UI draw phase) is registered once per phase it belongs to; the aggregate owns every phase.

## Not: systems stored as `Option` fields and called manually

```rust
pub struct BattleTestScene {
    hud: Option<HudDrawSystem>,       // no
    skills_bar: Option<SkillsBarDrawSystem>, // no
    dash_ui: Option<KnightDashSystem>, // no
    ram_ai: Option<RamHeadAiSystem>,  // no
}

impl Scene for BattleTestScene {
    fn draw_ui(&self, ctx: &Context) {
        if let Some(hud) = &self.hud {
            hud.draw(ctx);            // no — delegate to the aggregate instead
        }
        if let Some(skills_bar) = &self.skills_bar {
            skills_bar.draw(ctx);     // no
        }
        if let Some(dash_ui) = &self.dash_ui {
            dash_ui.draw_ui(ctx);     // no
        }
    }
}
```

If a system needs a screen-space pass (`draw_ui`), the aggregate exposes that phase and the system is added to it like any other — not wired through a scene field.
