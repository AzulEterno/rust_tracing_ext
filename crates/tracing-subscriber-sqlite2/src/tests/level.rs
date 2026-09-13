use crate::EventLevel;

#[test]
fn severity_values_increase_toward_error() {
    assert!(EventLevel::Trace < EventLevel::Debug);
    assert!(EventLevel::Debug < EventLevel::Info);
    assert!(EventLevel::Info < EventLevel::Warn);
    assert!(EventLevel::Warn < EventLevel::Error);
}
