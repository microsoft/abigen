use crate::{AbiConfig, AbiExceptionConfig};
use std::{collections::HashSet, fmt::Debug, hash::Hash, sync::LazyLock};

pub const CODE: &'static str = r#"
    class MyClass {
    public:
        const Config& config() const noexcept;
        void set_config(const Config& config);
    };
"#;

pub static CONFIG: LazyLock<AbiConfig> = LazyLock::new(|| AbiConfig {
    exception: AbiExceptionConfig {
        value_type: "Result".to_string(),
        nullref_value: "Result::ArgumentNull".to_string(),
    },
});

pub fn collection_eq<A, E>(
    actual: impl IntoIterator<Item = A>,
    expected: impl IntoIterator<Item = E>,
) where
    A: Debug,
    E: Debug + PartialEq + Eq + Hash + PartialEq<A>,
{
    let mut expected: HashSet<_> = expected.into_iter().collect();

    let mut unexpected = vec![];
    for item in actual {
        let expected = expected.extract_if(|i| i == &item).next();
        if expected.is_none() {
            unexpected.push(item);
        }
    }
    let unmatched: Vec<_> = expected.into_iter().collect();

    assert_eq!(unmatched, unexpected, "expected (left) != actual (right)");
}
