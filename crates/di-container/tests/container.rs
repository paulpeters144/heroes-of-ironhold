use std::future::Future;
use std::pin::Pin;

use di_container::{BuildContext, ContainerBuilder, Error, Injectable};

fn block_on<F: Future>(f: F) -> F::Output {
    pollster::block_on(f)
}

#[derive(Injectable)]
struct ConsoleLogger;

impl ConsoleLogger {
    fn log(&self, msg: &str) -> String {
        format!("logged: {msg}")
    }
}

#[test]
fn resolves_singleton_dependency() {
    let container = block_on(
        ContainerBuilder::new()
            .singleton::<ConsoleLogger, ConsoleLogger>()
            .build_and_leak(),
    )
    .unwrap();
    let logger = container.get::<ConsoleLogger>().unwrap();
    assert_eq!(logger.log("Hello"), "logged: Hello");
}

struct Db {
    _id: usize,
}

impl Db {
    fn next_id() -> usize {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl Injectable for Db {
    fn inject(
        _ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(std::future::ready(Ok(Self {
            _id: Self::next_id(),
        })))
    }
}

#[test]
fn singleton_returns_the_same_instance() {
    let c = block_on(
        ContainerBuilder::new()
            .singleton::<Db, Db>()
            .build_and_leak(),
    )
    .unwrap();
    let first = c.get::<Db>().unwrap();
    let second = c.get::<Db>().unwrap();
    assert!(std::ptr::eq(first, second));
}

#[test]
fn unregistered_service_returns_an_error() {
    #[derive(Debug)]
    struct Never;

    let c = block_on(ContainerBuilder::new().build_and_leak()).unwrap();
    let result = c.get::<Never>();
    assert!(matches!(
        result,
        Err(Error::ServiceNotRegistered { ref type_name }) if type_name.contains("Never")
    ));
    let err = result.unwrap_err();
    assert!(format!("{err}").contains("Never"));
}

#[test]
fn derive_supports_unit_tuple_and_named_structs() {
    #[derive(Injectable)]
    struct Unit;

    #[derive(Injectable)]
    struct Tuple(Unit, ConsoleLogger);

    #[derive(Injectable)]
    #[allow(dead_code)]
    struct Named {
        logger: ConsoleLogger,
        unit: Unit,
    }

    let c = block_on(
        ContainerBuilder::new()
            .singleton::<Unit, Unit>()
            .singleton::<ConsoleLogger, ConsoleLogger>()
            .singleton::<Tuple, Tuple>()
            .singleton::<Named, Named>()
            .build_and_leak(),
    )
    .unwrap();

    let _unit = c.get::<Unit>().unwrap();
    let _tuple = c.get::<Tuple>().unwrap();
    let _named = c.get::<Named>().unwrap();
    let logger = c.get::<ConsoleLogger>().unwrap();
    assert_eq!(logger.log("x"), "logged: x");
}
