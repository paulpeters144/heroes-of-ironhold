use heroes_of_ironhold_core::entity::{
    SkillDirection, SkillIconKind, SkillSlotCfg, SkillsFactory, SkillsParts,
};

fn four_attacks() -> SkillsParts {
    SkillsFactory::create(&[
        SkillSlotCfg::icon(SkillIconKind::BladeBarrage).direction(SkillDirection::Up),
        SkillSlotCfg::icon(SkillIconKind::Shield).direction(SkillDirection::Right),
        SkillSlotCfg::icon(SkillIconKind::Fireball).direction(SkillDirection::Down),
        SkillSlotCfg::empty().direction(SkillDirection::Left),
    ])
}

#[test]
fn slots_match_config_order() {
    let parts = four_attacks();

    let expected = [
        Some(SkillIconKind::BladeBarrage),
        Some(SkillIconKind::Shield),
        Some(SkillIconKind::Fireball),
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
fn directions_are_tagged_on_each_skill() {
    let parts = four_attacks();

    let directions: Vec<Option<SkillDirection>> = parts
        .slots
        .iter()
        .map(|slot| slot.skill.direction)
        .collect();
    assert_eq!(
        directions,
        [
            Some(SkillDirection::Up),
            Some(SkillDirection::Right),
            Some(SkillDirection::Down),
            Some(SkillDirection::Left),
        ]
    );
}

#[test]
fn empty_slot_has_direction_but_no_icon() {
    let parts = four_attacks();

    let left = &parts.slots[3];
    assert_eq!(left.icon.map(|icon| icon.kind), None);
    assert_eq!(left.skill.direction, Some(SkillDirection::Left));
}

#[test]
fn icon_without_direction_has_none_direction() {
    let parts = SkillsFactory::create(&[SkillSlotCfg::icon(SkillIconKind::BladeBarrage)]);

    assert_eq!(parts.slots.len(), 1);
    let slot = &parts.slots[0];
    assert_eq!(
        slot.icon.map(|icon| icon.kind),
        Some(SkillIconKind::BladeBarrage)
    );
    assert_eq!(slot.skill.direction, None);
}
