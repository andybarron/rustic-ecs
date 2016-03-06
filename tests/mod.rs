use std::ops::Add;
use std::collections::{HashMap, HashSet};
#[macro_use]
extern crate recs;
use recs::*;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Vector2f {
    x: f32,
    y: f32,
}

impl Vector2f {
    fn new(x: f32, y: f32) -> Self {
        Vector2f { x: x, y: y }
    }
    fn new_i64(x: i64, y: i64) -> Self {
        Self::new(x as f32, y as f32)
    }
}

impl Add for Vector2f {
    type Output = Vector2f;
    fn add(self, other: Self) -> Self {
        Vector2f {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Position(Vector2f);

#[derive(Copy, Clone, PartialEq, Debug)]
struct Velocity(Vector2f);

fn update_position(pos: &Position, vel: &Velocity) -> Position {
    Position(pos.0 + vel.0)
}

#[test]
fn test_update() {
    let a_start = Vector2f::new(1., 3.);
    let b_start = Vector2f::new(-3., 4.);
    let c_start = Vector2f::new(-0., 1.3);
    let a_vel = Vector2f::new(0., 2.);
    let b_vel = Vector2f::new(1., 9.);
    let mut ecs = Ecs::new();
    let a = ecs.create_entity();
    let b = ecs.create_entity();
    let c = ecs.create_entity();
    let _ = ecs.set(a, Position(a_start));
    let _ = ecs.set(a, Velocity(a_vel));
    let _ = ecs.set(b, Position(b_start));
    let _ = ecs.set(b, Velocity(b_vel));
    let _ = ecs.set(c, Position(c_start));
    let mut ids = Vec::new();
    ecs.collect(&mut ids);
    for id in ids {
        let p = ecs.get::<Position>(id);
        let v = ecs.get::<Velocity>(id);
        if let (Ok(pos), Ok(vel)) = (p, v) {
            let _ = ecs.set(id, update_position(&pos, &vel));
        }
    }
    assert!(ecs.get::<Position>(a) == Ok(Position(a_start + a_vel)));
    assert!(ecs.get::<Position>(b) == Ok(Position(b_start + b_vel)));
    assert!(ecs.get::<Position>(c) == Ok(Position(c_start)));
}

#[test]
fn test_collect() {
    let count = 500;
    let mut ids = Vec::with_capacity(count);
    let mut starts = HashMap::with_capacity(count);
    let mut speeds = HashMap::with_capacity(count);
    let mut system = Ecs::new();
    for c in 0..count {
        let i = c as i64;
        let id = system.create_entity();
        let pos = Position(Vector2f::new_i64(4 * i - 7, -2 * i + 3));
        let vel = Velocity(Vector2f::new_i64(-100 * i + 350, 500 * i - 900));
        ids.push(id);
        starts.insert(id, pos);
        speeds.insert(id, vel);
        let _ = system.set(id, pos);
        let _ = system.set(id, vel);
    }
    // check that all ids are contained within ECS
    assert_eq!(ids.iter().cloned().collect::<HashSet<_>>(),
               system.iter().collect::<HashSet<_>>());
    let components = component_filter!(Position, Velocity);
    let mut to_update = Vec::new();
    system.collect_with(&components, &mut to_update);
    for id in to_update.iter().cloned() {
        // We can safely call unwrap() here, because
        // collect_with(..) guarantees that all of these
        // entities have Position and Velocity
        let pos: Position = system.get(id).unwrap();
        let vel: Velocity = system.get(id).unwrap();
        let new_pos = Position(pos.0 + vel.0);
        let _ = system.set(id, new_pos);
    }
    // check that all positions are properly updated
    for id in system.iter() {
        let target_pos = Position(starts[&id].0 + speeds[&id].0);
        assert_eq!(Ok(target_pos), system.get(id));
    }
}
