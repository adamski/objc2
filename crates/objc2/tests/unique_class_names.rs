//! Tests for the `unique-class-names` feature.
#![cfg(feature = "unique-class-names")]
use std::ffi::{CStr, CString};

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, NSObject};
use objc2::{define_class, msg_send, ClassType};

fn assert_unique_name(cls: &AnyClass, base: &str) {
    let name = cls.name().to_str().unwrap();
    let suffix = name
        .strip_prefix(base)
        .unwrap_or_else(|| panic!("{name:?} does not start with {base:?}"));
    assert_eq!(suffix.len(), 17, "{name:?}");
    assert!(suffix.starts_with('_'), "{name:?}");
    assert!(
        suffix[1..].bytes().all(|b| b.is_ascii_hexdigit()),
        "{name:?}"
    );
}

#[test]
fn new_unique_registers_distinct_classes() {
    let base = CStr::from_bytes_with_nul(b"UniqueClassNamesBase\0").unwrap();
    let a = ClassBuilder::new_unique(base, NSObject::class())
        .unwrap()
        .register();
    let b = ClassBuilder::new_unique(base, NSObject::class())
        .unwrap()
        .register();

    assert_ne!(a, b);
    assert_ne!(a.name(), b.name());
    assert_unique_name(a, "UniqueClassNamesBase");
    assert_unique_name(b, "UniqueClassNamesBase");
    assert!(AnyClass::get(base).is_none());
}

#[test]
fn auto_named_class_does_not_reuse_existing_class() {
    define_class!(
        #[unsafe(super(NSObject))]
        struct Collide;
    );

    // Simulate another dynamic library that already registered a class with
    // the same auto-generated name.
    let name = CString::new(Collide::NAME).unwrap();
    let other = ClassBuilder::new(&name, NSObject::class())
        .unwrap()
        .register();

    let cls = Collide::class();
    assert_ne!(cls, other);
    assert_unique_name(cls, Collide::NAME);
    assert_eq!(AnyClass::get(&name), Some(other));

    let obj: Retained<AnyObject> = unsafe { msg_send![cls, new] };
    assert_eq!(obj.class(), cls);
}

#[test]
fn explicit_name_is_kept() {
    define_class!(
        #[unsafe(super(NSObject))]
        #[name = "UniqueClassNamesExplicit"]
        struct Explicit;
    );

    assert_eq!(
        Explicit::class().name().to_str().unwrap(),
        "UniqueClassNamesExplicit"
    );
}
