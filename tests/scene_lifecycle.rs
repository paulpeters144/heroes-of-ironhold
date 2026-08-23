use event_bus::{EventBus, SubCollection};

struct TestEvent;

#[test]
fn scene_subscriptions_dropped_when_collection_dropped() {
    let bus = EventBus::new();
    assert_eq!(bus.sub_count(), 0);

    let subs = SubCollection::new();
    subs.on(&bus, |_: &TestEvent| {});
    subs.on(&bus, |_: &TestEvent| {});
    assert_eq!(bus.sub_count(), 2);

    drop(subs);
    assert_eq!(bus.sub_count(), 0);
}
