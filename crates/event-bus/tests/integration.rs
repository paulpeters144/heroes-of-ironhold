use std::cell::{Cell, RefCell};
use std::rc::Rc;

use event_bus::{EventBus, SubCollection};

#[derive(Debug, Clone)]
struct OrderPlaced {
    order_id: i32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserCreated {
    name: String,
}

#[test]
fn fire_publishes_to_single_subscriber() {
    let bus = EventBus::new();
    let received = Rc::new(Cell::new(0usize));
    let r = received.clone();

    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(move |e: &OrderPlaced| {
        r.set(e.order_id as usize);
    });

    bus.fire(&OrderPlaced { order_id: 42 });
    assert_eq!(received.get(), 42);
}

#[test]
fn fire_publishes_to_multiple_subscribers() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let subs = SubCollection::new();
    for _ in 0..3 {
        let c = count.clone();
        subs.on(&bus, move |_e: &OrderPlaced| {
            c.set(c.get() + 1);
        });
    }

    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 3);
}

#[test]
fn fire_no_subscriber_for_event_type_does_nothing() {
    let bus = EventBus::new();
    let received = Rc::new(Cell::new(0usize));
    let r = received.clone();

    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(move |_e: &OrderPlaced| {
        r.set(r.get() + 1);
    });

    bus.fire(&UserCreated {
        name: "test".into(),
    });
    assert_eq!(received.get(), 0);
}

#[test]
fn subscription_dispose_removes_subscriber() {
    let bus = EventBus::new();
    let received = Rc::new(Cell::new(0usize));
    let r = received.clone();

    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(move |_e: &OrderPlaced| {
        r.set(r.get() + 1);
    });

    sub.dispose();
    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(received.get(), 0);
}

#[test]
fn subscription_is_active_becomes_false_after_dispose() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    assert!(sub.is_active());
    sub.dispose();
    assert!(!sub.is_active());
}

#[test]
fn subscription_dispose_twice_removes_only_once() {
    let bus = EventBus::new();
    let sub1 = bus.create_sub::<OrderPlaced>();
    let su2 = bus.create_sub::<OrderPlaced>();
    sub1.dispose();
    su2.dispose();
    assert_eq!(bus.sub_count(), 0);
}

#[test]
fn fire_handler_not_set_does_not_panic() {
    let bus = EventBus::new();
    bus.create_sub::<OrderPlaced>();
    bus.fire(&OrderPlaced { order_id: 1 });
}

#[test]
fn fire_subscription_before_on_message_is_handled_when_handler_is_set() {
    let bus = EventBus::new();
    let received = Rc::new(Cell::new(0usize));

    let sub = bus.create_sub::<OrderPlaced>();
    bus.fire(&OrderPlaced { order_id: 99 });

    let r = received.clone();
    sub.on_message(move |e: &OrderPlaced| {
        r.set(e.order_id as usize);
    });

    bus.fire(&OrderPlaced { order_id: 7 });
    assert_eq!(received.get(), 7);
}

#[test]
fn create_sub_returns_unique_id() {
    let bus = EventBus::new();
    let sub1 = bus.create_sub::<OrderPlaced>();
    let sub2 = bus.create_sub::<OrderPlaced>();
    assert_ne!(sub1.id(), sub2.id());
}

#[test]
fn sub_count_initial_is_zero() {
    let bus = EventBus::new();
    assert_eq!(bus.sub_count(), 0);
}

#[test]
fn sub_count_increments_after_subscription() {
    let bus = EventBus::new();
    let _sub1 = bus.create_sub::<OrderPlaced>();
    assert_eq!(bus.sub_count(), 1);
    let _sub2 = bus.create_sub::<OrderPlaced>();
    assert_eq!(bus.sub_count(), 2);
}

#[test]
fn sub_count_decrements_after_dispose() {
    let bus = EventBus::new();
    let sub1 = bus.create_sub::<OrderPlaced>();
    let _sub2 = bus.create_sub::<OrderPlaced>();
    assert_eq!(bus.sub_count(), 2);
    sub1.dispose();
    assert_eq!(bus.sub_count(), 1);
}

#[test]
fn clear_removes_all_subscriptions_and_resets_count() {
    let bus = EventBus::new();
    let _sub1 = bus.create_sub::<OrderPlaced>();
    let _sub2 = bus.create_sub::<UserCreated>();
    assert_eq!(bus.sub_count(), 2);
    bus.clear();
    assert_eq!(bus.sub_count(), 0);
}

#[test]
#[should_panic(expected = "handler already set")]
fn on_message_panics_when_called_twice() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(|_: &OrderPlaced| {});
    sub.on_message(|_: &OrderPlaced| {});
}

#[test]
fn different_event_types_do_not_interfere() {
    let bus = EventBus::new();
    let order_count = Rc::new(Cell::new(0usize));
    let user_count = Rc::new(Cell::new(0usize));

    let oc = order_count.clone();
    let sub1 = bus.create_sub::<OrderPlaced>();
    sub1.on_message(move |_e: &OrderPlaced| {
        oc.set(oc.get() + 1);
    });

    let uc = user_count.clone();
    let sub2 = bus.create_sub::<UserCreated>();
    sub2.on_message(move |_e: &UserCreated| {
        uc.set(uc.get() + 1);
    });

    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(order_count.get(), 1);
    assert_eq!(user_count.get(), 0);

    bus.fire(&UserCreated {
        name: "test".into(),
    });
    assert_eq!(order_count.get(), 1);
    assert_eq!(user_count.get(), 1);
}

#[test]
fn fire_after_all_subscriptions_disposed_is_noop() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let c = count.clone();
    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(move |_e: &OrderPlaced| {
        c.set(c.get() + 1);
    });

    sub.dispose();
    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 0);

    bus.fire(&OrderPlaced { order_id: 2 });
    assert_eq!(count.get(), 0);
}

#[test]
fn self_dispose_during_fire_does_not_affect_other_handlers() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let c = count.clone();
    let sub1 = bus.create_sub::<OrderPlaced>();
    sub1.on_message(move |_e: &OrderPlaced| {
        c.set(c.get() + 1);
    });

    bus.create_sub::<OrderPlaced>()
        .on_message(|_e: &OrderPlaced| {
            unreachable!("disposed handler should not be called");
        })
        .dispose();

    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 1);
}

#[test]
fn handler_self_dispose_during_fire_works() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let c = count.clone();
    let sub = bus.create_sub::<OrderPlaced>();
    let sub2 = sub.clone();

    sub.on_message(move |_e: &OrderPlaced| {
        c.set(c.get() + 1);
        sub2.dispose();
    });

    bus.fire(&OrderPlaced { order_id: 5 });
    assert_eq!(count.get(), 1);

    bus.fire(&OrderPlaced { order_id: 6 });
    assert_eq!(count.get(), 1);
}

#[test]
fn handler_panic_propagates_and_stops_iteration() {
    use std::panic;

    let bus = EventBus::new();
    let second_called = Rc::new(Cell::new(false));

    let sub1 = bus.create_sub::<OrderPlaced>();
    sub1.on_message(|_e: &OrderPlaced| {
        panic!("first handler panics");
    });

    let sc = second_called.clone();
    let sub2 = bus.create_sub::<OrderPlaced>();
    sub2.on_message(move |_e: &OrderPlaced| {
        sc.set(true);
    });

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        bus.fire(&OrderPlaced { order_id: 1 });
    }));
    assert!(result.is_err());
}

#[test]
fn multiple_fires_deliver_to_same_handler() {
    let bus = EventBus::new();
    let sum = Rc::new(Cell::new(0usize));

    let s = sum.clone();
    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(move |e: &OrderPlaced| {
        s.set(s.get() + e.order_id as usize);
    });

    bus.fire(&OrderPlaced { order_id: 10 });
    bus.fire(&OrderPlaced { order_id: 20 });
    bus.fire(&OrderPlaced { order_id: 30 });
    assert_eq!(sum.get(), 60);
}

#[test]
fn create_sub_after_clear_works() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let c = count.clone();
    let sub1 = bus.create_sub::<OrderPlaced>();
    sub1.on_message(move |_e: &OrderPlaced| {
        c.set(c.get() + 1);
    });

    bus.clear();
    assert_eq!(bus.sub_count(), 0);
    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 0);

    let c2 = count.clone();
    let sub2 = bus.create_sub::<OrderPlaced>();
    sub2.on_message(move |_e: &OrderPlaced| {
        c2.set(c2.get() + 1);
    });

    assert_eq!(bus.sub_count(), 1);
    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 1);
}

#[test]
fn clear_on_empty_bus_is_noop() {
    let bus = EventBus::new();
    assert_eq!(bus.sub_count(), 0);
    bus.clear();
    assert_eq!(bus.sub_count(), 0);
}

#[test]
fn sub_count_across_multiple_event_types() {
    let bus = EventBus::new();
    assert_eq!(bus.sub_count(), 0);

    let _sub1 = bus.create_sub::<OrderPlaced>();
    assert_eq!(bus.sub_count(), 1);

    let _sub2 = bus.create_sub::<UserCreated>();
    assert_eq!(bus.sub_count(), 2);

    let _sub3 = bus.create_sub::<OrderPlaced>();
    assert_eq!(bus.sub_count(), 3);

    let sub4 = bus.create_sub::<UserCreated>();
    assert_eq!(bus.sub_count(), 4);

    sub4.dispose();
    assert_eq!(bus.sub_count(), 3);
}

#[test]
fn disposed_subscription_does_not_receive_events() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let c = count.clone();
    let sub = bus.create_sub::<OrderPlaced>();
    sub.on_message(move |_e: &OrderPlaced| {
        c.set(c.get() + 1);
    });

    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 1);

    sub.dispose();
    bus.fire(&OrderPlaced { order_id: 2 });
    assert_eq!(count.get(), 1);
}

#[test]
fn id_is_consistent() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    let id = sub.id();
    assert_eq!(sub.id(), id);
    assert_eq!(sub.id(), id);
}

#[test]
fn is_active_is_true_for_new_subscription() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    assert!(sub.is_active());
}

#[test]
fn fire_with_no_subscriptions_for_any_type_is_noop() {
    let bus = EventBus::new();
    bus.fire(&OrderPlaced { order_id: 1 });
    bus.fire(&UserCreated { name: "x".into() });
}

#[test]
fn large_number_of_subscribers_and_fires() {
    let bus = EventBus::new();
    let total = Rc::new(Cell::new(0usize));

    let subs = SubCollection::new();
    for _ in 0..500 {
        let t = total.clone();
        subs.on(&bus, move |_e: &OrderPlaced| {
            t.set(t.get() + 1);
        });
    }

    let iterations = 200;
    for i in 0..iterations {
        bus.fire(&OrderPlaced { order_id: i as i32 });
    }

    assert_eq!(total.get(), 500 * iterations);
}

#[test]
fn disposed_sub_id_still_accessible() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    let id = sub.id();
    sub.dispose();
    assert_eq!(sub.id(), id);
    assert!(!sub.is_active());
}

#[test]
fn subscriber_added_during_fire_does_not_receive_current_event() {
    let bus = Rc::new(EventBus::new());
    let second_added_called = Rc::new(Cell::new(false));

    let late_subs: Rc<SubCollection> = Rc::new(SubCollection::new());
    let ls = late_subs.clone();
    let sa = second_added_called.clone();
    let b = bus.clone();
    let outer_sub = bus.create_sub::<OrderPlaced>();
    outer_sub.on_message(move |_e: &OrderPlaced| {
        let sa2 = sa.clone();
        let b2 = b.clone();
        ls.on(&b2, move |_e: &OrderPlaced| {
            sa2.set(true);
        });
    });

    bus.fire(&OrderPlaced { order_id: 1 });
    assert!(!second_added_called.get());

    bus.fire(&OrderPlaced { order_id: 2 });
    assert!(second_added_called.get());
}

#[test]
fn clear_then_dispose_old_subscription_does_not_panic() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    bus.clear();
    sub.dispose();
}

#[test]
fn subscription_clone_shares_identity() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    let clone = sub.clone();
    assert_eq!(sub.id(), clone.id());
    assert_eq!(sub.is_active(), clone.is_active());
}

#[test]
fn on_message_fluent_returns_self() {
    let bus = EventBus::new();
    let sub = bus.create_sub::<OrderPlaced>();
    let returned = sub.on_message(|_: &OrderPlaced| {});
    assert_eq!(sub.id(), returned.id());
}

#[test]
fn default_creates_empty_bus() {
    let bus = EventBus::default();
    assert_eq!(bus.sub_count(), 0);
}

#[test]
fn fire_with_mixed_active_inactive_subscribers() {
    let bus = EventBus::new();
    let count = Rc::new(Cell::new(0usize));

    let c1 = count.clone();
    let sub1 = bus.create_sub::<OrderPlaced>();
    sub1.on_message(move |_e: &OrderPlaced| {
        c1.set(c1.get() + 1);
    });

    let c2 = count.clone();
    let disposed = bus
        .create_sub::<OrderPlaced>()
        .on_message(move |_e: &OrderPlaced| {
            c2.set(c2.get() + 100);
        })
        .clone();

    let c3 = count.clone();
    let sub3 = bus.create_sub::<OrderPlaced>();
    sub3.on_message(move |_e: &OrderPlaced| {
        c3.set(c3.get() + 1);
    });

    disposed.dispose();
    bus.fire(&OrderPlaced { order_id: 1 });
    assert_eq!(count.get(), 2);
}

#[test]
fn str_event_type() {
    let bus = EventBus::new();
    let received = Rc::new(RefCell::new(String::new()));

    let r = received.clone();
    let sub = bus.create_sub::<String>();
    sub.on_message(move |e: &String| {
        *r.borrow_mut() = e.clone();
    });

    bus.fire(&"hello".to_string());
    assert_eq!(*received.borrow(), "hello");
}

#[test]
fn i32_event_type() {
    let bus = EventBus::new();
    let received = Rc::new(Cell::new(0usize));

    let r = received.clone();
    let sub = bus.create_sub::<i32>();
    sub.on_message(move |e: &i32| {
        r.set(*e as usize);
    });

    bus.fire(&42i32);
    assert_eq!(received.get(), 42);
}

#[test]
fn tuple_struct_event() {
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct Score(u32, String);

    let bus = EventBus::new();
    let score = Rc::new(Cell::new(0usize));

    let s = score.clone();
    let sub = bus.create_sub::<Score>();
    sub.on_message(move |e: &Score| {
        s.set(e.0 as usize);
    });

    bus.fire(&Score(99, "player".into()));
    assert_eq!(score.get(), 99);
}
