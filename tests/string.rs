use lstring as kstring;

#[test]
fn test_kformat_macro() {
    use kstring::{kformat, KString};

    let result: KString = kformat!("Hello, {}!", "world");
    assert_eq!(result.as_str(), "Hello, world!");

    let name = "Alice";
    let age = 30;
    let result: KString = kformat!("Name: {}, Age: {}", name, age);
    assert_eq!(result.as_str(), "Name: Alice, Age: 30");

    let empty = "";
    let result: KString = kformat!("{}", empty);
    assert_eq!(result.as_str(), "");

    let number = 42;
    let result: KString = kformat!("The answer is {}", number);
    assert_eq!(result.as_str(), "The answer is 42");
}

#[test]
fn test_search_in_hashmap() {
    let mut m = std::collections::HashMap::<kstring::KString, i32>::new();
    m.insert("aaa".into(), 17);
    assert_eq!(17, *m.get("aaa").unwrap());
}

fn check_props(
    std_str: &str,
    kstr: kstring::KString,
) -> Result<(), proptest::test_runner::TestCaseError> {
    #![allow(clippy::redundant_clone)]
    proptest::prop_assert_eq!(kstr.clone(), std_str);
    proptest::prop_assert_eq!(kstr.as_str(), std_str);
    proptest::prop_assert_eq!(kstr.len(), std_str.len());
    proptest::prop_assert_eq!(kstr.is_empty(), std_str.is_empty());
    Ok(())
}

proptest::proptest! {
    #[test]
    #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
    fn roundtrip_string(s: String) {
        let uut = kstring::KString::from_string(s.clone());
        check_props(s.as_str(), uut)?;
    }

    #[test]
    #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
    fn roundtrip_ref(s: String) {
        let uut = kstring::KString::from_ref(&s);
        check_props(s.as_str(), uut)?;
    }

    #[test]
    #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
    fn roundtrip_static(s: String) {
        let uut = kstring::KString::from_static(std::boxed::Box::leak(s.clone().into_boxed_str()));
        check_props(s.as_str(), uut)?;
    }
}
