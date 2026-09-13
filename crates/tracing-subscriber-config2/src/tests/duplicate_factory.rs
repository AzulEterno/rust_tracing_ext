use crate::error::RegisterError;
use crate::runtime::TracingBuilder;
use crate::tests::support::factory;

#[test]
fn builder_rejects_duplicate_factory_kinds() {
    let mut builder = TracingBuilder::new();
    builder.register(factory().0).expect("register factory");
    let error = builder
        .register(factory().0)
        .err()
        .expect("duplicate kind must fail");

    assert!(matches!(error, RegisterError::DuplicateKind("count")));
}
