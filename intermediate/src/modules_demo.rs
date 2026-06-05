//! The module system: organizing code and controlling visibility.
//!
//! Modules are about PRIVACY first, file layout second. Everything is
//! private by default; `pub` opens it up level by level. A good module
//! exposes a small, deliberate API and hides its plumbing — the
//! compiler then guarantees nobody depends on the plumbing.
//!
//! This file holds a nested module tree inline (`mod x { ... }`) to
//! keep the example self-contained. In real crates each module
//! usually lives in its own file (`mod x;` + `x.rs`) — the semantics
//! are identical; only the file layout differs. The `fundamentals`
//! crate in this workspace is laid out exactly that way.

/// A small "restaurant" with a public front and a private back —
/// the structure mirrors the example in The Book, because it works.
pub mod restaurant {
    // Not pub: the kitchen is an implementation detail. Code outside
    // `restaurant` cannot name it at all.
    mod kitchen {
        // pub(super): visible to the parent module (`restaurant`)
        // but no further. Finer-grained than all-or-nothing pub.
        pub(super) fn cook(dish: &str) -> String {
            format!("hot plate of {dish}")
        }

        pub(super) const TODAYS_SPECIAL: &str = "lentil soup";
    }

    // The public API of the restaurant module.
    pub fn order(dish: &str) -> String {
        // Inside the tree we reach the private kitchen freely.
        kitchen::cook(dish)
    }

    pub fn todays_special() -> String {
        order(kitchen::TODAYS_SPECIAL)
    }

    pub mod menu {
        /// pub(crate): callable anywhere in THIS crate, invisible to
        /// external users of the library. Use it for cross-module
        /// helpers that aren't part of your public contract.
        pub(crate) fn internal_price_check(dish: &str) -> u32 {
            dish.len() as u32 // silly pricing model, real enough for a demo
        }

        pub fn price(dish: &str) -> u32 {
            internal_price_check(dish)
        }
    }
}

// Re-exports: `pub use` lifts a deeply nested item to a shorter path.
// Users write `modules_demo::price` instead of
// `modules_demo::restaurant::menu::price`. Crates use this to keep
// internal organization free to change without breaking users.
pub use restaurant::menu::price;

/// `use` brings paths into scope; idiomatic style imports the PARENT
/// module for functions (`menu::price`, not bare `price`) so call
/// sites show where things come from — but imports the item itself
/// for types (`use std::collections::HashMap`).
pub fn lunch_order() -> (String, u32) {
    use restaurant::menu;
    let dish = "lentil soup";
    (restaurant::order(dish), menu::price(dish))
}

#[cfg(test)]
mod tests {
    // Test modules see their parent through `super` — this is the
    // same module system, not a special test mechanism.
    use super::*;

    #[test]
    fn public_api_works() {
        assert_eq!(restaurant::order("pasta"), "hot plate of pasta");
        assert_eq!(restaurant::todays_special(), "hot plate of lentil soup");
    }

    #[test]
    fn reexports_shorten_paths() {
        // Same function, two paths — the re-export is an alias.
        assert_eq!(price("pasta"), restaurant::menu::price("pasta"));
    }

    #[test]
    fn crate_visible_helpers_work_inside_the_crate() {
        // pub(crate) items are reachable from here (same crate)...
        assert_eq!(restaurant::menu::internal_price_check("soup"), 4);
        // ...but a different crate using this library could not call
        // it — and `restaurant::kitchen` is not even nameable here.
    }

    #[test]
    fn composed_example() {
        let (dish, price) = lunch_order();
        assert_eq!(dish, "hot plate of lentil soup");
        assert_eq!(price, 11);
    }
}

// Exercises
// ---------
// 1. Add a `delivery` module with one public function that uses the
//    kitchen. Decide: should it call `kitchen::cook` directly (needs a
//    visibility change) or go through `order`?
// 2. Move `restaurant` into its own file (`restaurant.rs` + `mod
//    restaurant;`). Confirm nothing else changes.
// 3. In a binary crate, what is the difference between `use
//    crate::foo` and `use self::foo`? When does `super` beat both?
