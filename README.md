Rustic Entity-Component System
==============================
Simple entity-component system in pure Rust. Type reflection - no macros!

[![Build Status](https://travis-ci.org/AndyBarron/rustic-ecs.svg?branch=master)](https://travis-ci.org/AndyBarron/rustic-ecs)

Install
-------
Visit [the crates.io page](https://crates.io/crates/recs), and add the
specified line ("`recs = ...`") to the `[dependencies]` section of your
Cargo.toml. From then on, `cargo build` should automatically download and compile
Rustic ECS.

Documentation
-------------
<https://andybarron.github.io/rustic-ecs>

Example
-------
```rust
extern crate recs;
use recs::{Ecs, EntityId};

#[derive(Clone, PartialEq, Debug)]
struct Age{years: u32}

#[derive(Clone, PartialEq, Debug)]
struct Iq{points: i32}

fn main() {

    // Create an ECS instance
    let mut system: Ecs = Ecs::new();

    // Add entity to the system
    let forrest: EntityId = system.create_entity();

    // Attach components to the entity
    // The Ecs.set method returns an EcsResult that will be set to Err if
    // the specified entity does not exist. If you're sure that the entity exists, suppress
    // Rust's "unused result" warning by prefixing your calls to set(..) with "let _ = ..."
    let _ = system.set(forrest, Age{years: 22});
    let _ = system.set(forrest, Iq{points: 75}); // "I may not be a smart man..."

    // Get clone of attached component data from entity
    let age = system.get::<Age>(forrest).unwrap();
    assert_eq!(age.years, 22);

    // Annotating the variable's type may let you skip type parameters
    let iq: Iq = system.get(forrest).unwrap();
    assert_eq!(iq.points, 75);

    // Modify an entity's component
    let older = Age{years: age.years + 1};
    let _ = system.set(forrest, older);

    // Modify a component in-place with a mutable borrow
    system.borrow_mut::<Iq>(forrest).map(|iq| iq.points += 5);

    // Inspect a component in-place without cloning
    assert_eq!(system.borrow::<Age>(forrest), Ok(&Age{years: 23}));

    // Inspect a component via cloning
    assert_eq!(system.get::<Iq>(forrest), Ok(Iq{points: 80}));

}
```

License
-------
MIT. Hooray!

(See `LICENSE.txt` for details.)