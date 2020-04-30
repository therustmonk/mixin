#![feature(proc_macro_hygiene)]

use mixin::{mixin, mixin_declare, mixin_expand};

#[test]
fn test_base_check() {
    #[mixin_declare]
    pub struct Themeable {
        theme: bool,
    }

    #[mixin_expand]
    impl Themeable {
        pub fn has_theme(&self) -> bool {
            self.theme
        }
    }

    #[mixin(Themeable)]
    pub struct MyStruct {}

    let my_struct = MyStruct { theme: true };
    assert_eq!(my_struct.has_theme(), true);
}
