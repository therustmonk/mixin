#![feature(proc_macro_hygiene)]

#[test]
fn test_base_case() {
    #[mixin::declare]
    pub struct Themeable {
        theme: bool,
    }

    #[mixin::expand]
    impl Themeable {
        pub fn has_theme(&self) -> bool {
            self.theme
        }
    }

    #[mixin::insert(Themeable)]
    pub struct MyStruct {}

    let my_struct = MyStruct { theme: true };
    assert_eq!(my_struct.has_theme(), true);
}

#[test]
fn test_empty_vase() {
    #[mixin::declare]
    pub struct Themeable {}

    #[mixin::insert(Themeable)]
    pub struct MyStruct {}

    MyStruct {};
}

#[test]
fn test_own_fields() {
    #[mixin::declare]
    pub struct Themeable {}

    #[mixin::insert(Themeable)]
    pub struct MyStruct {
        _own: u8,
    }

    MyStruct { _own: 1 };
}

#[test]
fn test_can_derive() {
    #[mixin::declare]
    pub struct Value {
        value: u8,
    }

    #[mixin::insert(Value)]
    #[derive(Debug, Clone)]
    pub struct MyStruct {}

    let my_struct = MyStruct { value: 1 };
    format!("{:?}", my_struct.clone());
}

#[test]
fn test_fields_not_corrupted() {
    #[mixin::declare]
    pub struct Value {
        value: u8,
    }

    #[mixin::insert(Value)]
    #[derive(Debug, Clone)]
    pub struct MyStruct {
        own_value: u8,
    }

    let my_struct = MyStruct {
        value: 1,
        own_value: 2,
    };
    format!("{:?}", my_struct.clone());
}

#[test]
fn test_multiple() {
    #[mixin::declare]
    pub struct Themeable {
        theme: bool,
    }

    #[mixin::expand]
    impl Themeable {
        pub fn has_theme(&self) -> bool {
            self.theme
        }
    }

    #[mixin::declare]
    pub struct Worker {
        working: bool,
    }

    #[mixin::expand]
    impl Worker {
        pub fn is_working(&self) -> bool {
            self.working
        }
    }

    #[mixin::insert(Themeable, Worker)]
    pub struct MyStruct {}

    let my_struct = MyStruct {
        theme: true,
        working: false,
    };
    assert_eq!(my_struct.has_theme(), true);
    assert_eq!(my_struct.is_working(), false);
}
