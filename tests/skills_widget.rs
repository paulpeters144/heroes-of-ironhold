use heroes_of_ironhold_core::{SkillIconKind, SkillSlotCfg, SkillsFactory, SkillsParts};

fn six_slot_parts() -> SkillsParts {
    SkillsFactory::create(&[
        SkillSlotCfg::icon(SkillIconKind::Sword),
        SkillSlotCfg::icon(SkillIconKind::Shield),
        SkillSlotCfg::icon(SkillIconKind::Potion),
        SkillSlotCfg::icon(SkillIconKind::Fireball).selected(true),
        SkillSlotCfg::icon(SkillIconKind::Crossed),
        SkillSlotCfg::empty(),
    ])
}

#[test]
fn slots_match_config_order() {
    let parts = six_slot_parts();

    let expected = [
        Some(SkillIconKind::Sword),
        Some(SkillIconKind::Shield),
        Some(SkillIconKind::Potion),
        Some(SkillIconKind::Fireball),
        Some(SkillIconKind::Crossed),
        None,
    ];

    let icons: Vec<Option<SkillIconKind>> = parts
        .slots
        .iter()
        .map(|slot| slot.icon.map(|icon| icon.kind))
        .collect();
    assert_eq!(icons, expected);
}

#[test]
fn selected_flag_is_stored_on_the_skill() {
    let parts = six_slot_parts();

    let selected: Vec<bool> = parts
        .slots
        .iter()
        .map(|slot| slot.skill.selected)
        .collect();
    assert_eq!(selected, [false, false, false, true, false, false]);
}

#[test]
fn single_slot_produces_one_slot_with_icon_and_key() {
    let parts = SkillsFactory::create(&[SkillSlotCfg::icon(SkillIconKind::Sword)]);

    assert_eq!(parts.slots.len(), 1);
    let slot = &parts.slots[0];
    assert_eq!(slot.icon.map(|icon| icon.kind), Some(SkillIconKind::Sword));
    assert_eq!(slot.skill.key, Some('a'));
}

#[test]
fn keys_assign_in_order_and_skip_empty_slots() {
    let parts = six_slot_parts();

    let keys: Vec<Option<char>> = parts.slots.iter().map(|slot| slot.skill.key).collect();
    assert_eq!(
        keys,
        [Some('a'), Some('b'), Some('c'), Some('d'), Some('e'), None,]
    );
}

#[test]
fn keys_are_unique_across_real_skills() {
    let parts = six_slot_parts();

    let mut keys: Vec<char> = parts
        .slots
        .iter()
        .filter_map(|slot| slot.skill.key)
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys, ['a', 'b', 'c', 'd', 'e']);
}
