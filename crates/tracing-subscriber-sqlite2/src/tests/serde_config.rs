use crate::Config;

fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}

#[test]
fn config_supports_optional_serde() {
    assert_serde::<Config>();
}
