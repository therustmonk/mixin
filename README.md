# mixin macros

This crate contains `mixin` macros that combines fields and implementations of different structs.

Example:

```rust
#[mixin::declare]
pub struct Named {
    name: String,
}

#[mixin::expand]
impl Named {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[mixin::insert(Named)]
pub struct MyStruct {}

#[test]
fn test_it() {
    let my_struct = MyStruct { name: "MixIn Works" };
    assert_eq!(my_struct.name(), "MixIn Works");
}
```
