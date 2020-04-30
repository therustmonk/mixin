#![feature(proc_macro_hygiene)]

use mixin::{mixin, mixin_new};

#[test]
fn test_base_check() {
    #[mixin_new]
    pub struct Themeable {
        field: bool,
    }

    #[mixin(Themeable)]
    pub struct MyStruct {}

    let my_struct = MyStruct { field: true };
}
