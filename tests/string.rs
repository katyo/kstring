use lstring::{String, format, string};
use std::string::String as StdString;

#[test]
fn test_string_macro() {
    let result: String = string!("Hello, world!");
    assert_eq!(result.as_str(), "Hello, world!");

    let result: String = string!("Hello, ", "world!");
    assert_eq!(result.as_str(), "Hello, world!");

    let result: String = string!("Hello,", " world", "!");
    assert_eq!(result.as_str(), "Hello, world!");

    let result: String = string!("Hello,", " ", "world", "", "!");
    assert_eq!(result.as_str(), "Hello, world!");
}

#[test]
fn test_format_macro() {
    let result: String = format!("Hello, {}!", "world");
    assert_eq!(result.as_str(), "Hello, world!");

    let name = "Alice";
    let age = 30;
    let result: String = format!("Name: {}, Age: {}", name, age);
    assert_eq!(result.as_str(), "Name: Alice, Age: 30");

    let empty = "";
    let result: String = format!("{}", empty);
    assert_eq!(result.as_str(), "");

    let number = 42;
    let result: String = format!("The answer is {}", number);
    assert_eq!(result.as_str(), "The answer is 42");
}

#[test]
fn test_search_in_hashmap() {
    let mut m = std::collections::HashMap::<String, i32>::new();
    m.insert("aaa".into(), 17);
    assert_eq!(17, *m.get("aaa").unwrap());
}

fn check_props(std_str: &str, string: String) -> Result<(), proptest::test_runner::TestCaseError> {
    #![allow(clippy::redundant_clone)]
    proptest::prop_assert_eq!(string.clone(), std_str);
    proptest::prop_assert_eq!(string.as_str(), std_str);
    proptest::prop_assert_eq!(string.len(), std_str.len());
    proptest::prop_assert_eq!(string.is_empty(), std_str.is_empty());
    Ok(())
}

proptest::proptest! {
    #[test]
    #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
    fn roundtrip_string(s: StdString) {
        let uut = String::from_string(s.clone());
        check_props(s.as_str(), uut)?;
    }

    #[test]
    #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
    fn roundtrip_ref(s: StdString) {
        let uut = String::from_ref(&s);
        check_props(s.as_str(), uut)?;
    }

    #[test]
    #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
    fn roundtrip_static(s: StdString) {
        let uut = String::from_static(std::boxed::Box::leak(s.clone().into_boxed_str()));
        check_props(s.as_str(), uut)?;
    }
}
