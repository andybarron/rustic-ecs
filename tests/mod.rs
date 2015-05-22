use std::ops::Add;
extern crate recs;
use recs::*;

#[derive(Copy, Clone, PartialEq)]
struct Vector2f {
    x: f32,
    y: f32,
}

impl Vector2f {
    fn new(x: f32, y: f32) -> Self {
        Vector2f{x: x, y: y}
    }
}

impl Add for Vector2f {
    type Output = Vector2f;
    fn add(self, other: Self) -> Self {
        Vector2f{x: self.x+other.x, y: self.y+other.y}
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Position(Vector2f);

#[derive(Copy, Clone, PartialEq)]
struct Velocity(Vector2f);

fn update_position(pos: &Position, vel: &Velocity) -> Position {
    Position(pos.0 + vel.0)
}

#[test]
fn test() {
    let a_start = Vector2f::new(1., 3.);
    let b_start = Vector2f::new(-3., 4.);
    let c_start = Vector2f::new(-0., 1.3);
    let a_vel = Vector2f::new(0., 2.);
    let b_vel = Vector2f::new(1., 9.);
    let mut ecs = Ecs::new();
    let a = ecs.create_entity();
    let b = ecs.create_entity();
    let c = ecs.create_entity();
    ecs.set(a, &Position(a_start));
    ecs.set(a, &Velocity(a_vel));
    ecs.set(b, &Position(b_start));
    ecs.set(b, &Velocity(b_vel));
    ecs.set(c, &Position(c_start));
    for id in ecs.collect_ids() {
        let p = ecs.get::<Position>(id);
        let v = ecs.get::<Velocity>(id);
        if let (Some(pos), Some(vel)) = (p, v) {
            ecs.set(id, &update_position(&pos, &vel));
        }
    }
    assert!(ecs.get::<Position>(a) == Some(Position(a_start + a_vel)));
    assert!(ecs.get::<Position>(b) == Some(Position(b_start + b_vel)));
    assert!(ecs.get::<Position>(c) == Some(Position(c_start)));
}
