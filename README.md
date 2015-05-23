Rustic Entity-Component System
==============================
Simple entity-component system in pure Rust. Type reflection - no macros!

[![Build Status](https://travis-ci.org/AndyBarron/rustic-ecs.svg?branch=master)](https://travis-ci.org/AndyBarron/rustic-ecs)

Install
-------
Visit [the crates.io page](https://crates.io/crates/recs), and add the
specified line ("`recs = ...`") to the `[dependencies]` section of your
Cargo.toml. Then `cargo build` should automatically download and compile
Rustic ECS.

Documentation
-------------
Run `cargo doc`, then open `target/doc/recs/index.html` in your browser.

Example
-------
```
extern crate recs;
use recs::{Ecs, EntityId};

#[derive(Clone, PartialEq)]
struct Age{years: u32}

#[derive(Clone, PartialEq)]
struct Brain{iq: i32}

fn main() {
    // Create an ECS instance
    let mut ecs: Ecs = Ecs::new();
    // Add entity to the system
    let me: EntityId = ecs.create_entity();
    // Attach component to the entity
    ecs.set(me, &Age{years: 22});
    // Get attached component data from entity
    let older = ecs.get::<Age>(me).unwrap().years + 1;
    // Modify an entity's component
    ecs.set(me, &Age{years: older});
    // It works!
    assert!(ecs.get::<Age>(me) == Some(Age{years: 23}));
    assert!(ecs.get::<Brain>(me) == None); // Aw man...
}
```

License
-------
MIT. Hooray!