use heroes_of_ironhold_core::{
    EStore, Skill, SkillIcon, SkillIconKind, SkillSlotCfg, SkillsFactory, SkillsWidget,
};

fn spawn_six_slot_bar(store: &EStore) {
    SkillsFactory::spawn(
        store,
        &[
            SkillSlotCfg::icon(SkillIconKind::Sword),
            SkillSlotCfg::icon(SkillIconKind::Shield),
            SkillSlotCfg::icon(SkillIconKind::Potion),
            SkillSlotCfg::icon(SkillIconKind::Fireball).selected(true),
            SkillSlotCfg::icon(SkillIconKind::Crossed),
            SkillSlotCfg::empty(),
        ],
    );
}

#[test]
fn widget_has_skills_in_spawn_order() {
    let store = EStore::new();
    spawn_six_slot_bar(&store);

    let widget = store.first::<SkillsWidget>().expect("widget");
    let children = store.children(&widget);
    assert_eq!(children.len(), 6);

    let expected = [
        Some(SkillIconKind::Sword),
        Some(SkillIconKind::Shield),
        Some(SkillIconKind::Potion),
        Some(SkillIconKind::Fireball),
        Some(SkillIconKind::Crossed),
        None,
    ];

    for (eref, want) in children.iter().zip(expected) {
        let skill = store.get_by_id::<Skill>(eref.id()).expect("skill");
        let icon = store.get_child::<SkillIcon>(&skill).map(|i| i.kind);
        assert_eq!(icon, want);
    }
}

#[test]
fn selected_flag_is_stored_on_the_skill() {
    let store = EStore::new();
    spawn_six_slot_bar(&store);

    let widget = store.first::<SkillsWidget>().expect("widget");
    let selected: Vec<bool> = store
        .children(&widget)
        .iter()
        .map(|eref| store.get_by_id::<Skill>(eref.id()).unwrap().selected)
        .collect();
    assert_eq!(selected, [false, false, false, true, false, false]);
}

#[test]
fn single_skill_stays_parented() {
    let store = EStore::new();
    SkillsFactory::spawn(&store, &[SkillSlotCfg::icon(SkillIconKind::Sword)]);

    let widget = store.first::<SkillsWidget>().expect("widget");
    let children = store.children(&widget);
    assert_eq!(children.len(), 1);

    let skill = store.get_by_id::<Skill>(children[0].id()).expect("skill");
    let icon = store.get_child::<SkillIcon>(&skill).expect("icon");
    assert_eq!(icon.kind, SkillIconKind::Sword);
}

#[test]
fn keys_assign_in_order_and_skip_empty_slots() {
    let store = EStore::new();
    spawn_six_slot_bar(&store);

    let widget = store.first::<SkillsWidget>().expect("widget");
    let keys: Vec<Option<char>> = store
        .children(&widget)
        .iter()
        .map(|eref| store.get_by_id::<Skill>(eref.id()).unwrap().key)
        .collect();
    assert_eq!(
        keys,
        [Some('a'), Some('b'), Some('c'), Some('d'), Some('e'), None,]
    );
}

#[test]
fn keys_are_unique_across_real_skills() {
    let store = EStore::new();
    spawn_six_slot_bar(&store);

    let widget = store.first::<SkillsWidget>().expect("widget");
    let mut keys: Vec<char> = store
        .children(&widget)
        .iter()
        .filter_map(|eref| store.get_by_id::<Skill>(eref.id()).unwrap().key)
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys, ['a', 'b', 'c', 'd', 'e']);
}
