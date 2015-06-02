//! Simple entity-component system. Pure Rust (macro-free)!
//!
//! # Example
//! ```rust
//! extern crate recs;
//! use recs::{Ecs, EntityId};
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct Age{years: u32}
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct Iq{points: i32}
//!
//! fn main() {
//!
//!     // Create an ECS instance
//!     let mut system: Ecs = Ecs::new();
//!
//!     // Add entity to the system
//!     let forrest: EntityId = system.create_entity();
//!
//!     // Attach components to the entity
//!     system.set(forrest, &Age{years: 22});
//!     system.set(forrest, &Iq{points: 75}); // "I may not be a smart man..."
//!
//!     // Get clone of attached component data from entity
//!     let age = system.get::<Age>(forrest).unwrap();
//!     assert_eq!(age.years, 22);
//!
//!     // Annotating the variable's type may let you skip type parameters
//!     let iq: Iq = system.get(forrest).unwrap();
//!     assert_eq!(iq.points, 75);
//!
//!     // Modify an entity's component
//!     let older = Age{years: age.years + 1};
//!     system.set(forrest, &older);
//!
//!     // Modify a component in-place with a mutable borrow
//!     system.borrow_mut::<Iq>(forrest).map(|iq| iq.points += 5);
//!
//!     // Inspect a component in-place without cloning
//!     assert_eq!(system.borrow::<Age>(forrest), Some(&Age{years: 23}));
//!
//!     // Inspect a component via cloning
//!     assert_eq!(system.get::<Iq>(forrest), Some(Iq{points: 80}));
//!
//! }
//! ```
// TODO iterate every E/C pair
// TODO iterate every E and its C's
use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut, Keys};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;



/// Value type representing an entry in the entity-component system.
///
/// `EntityId` is an alias to `u64`. When storing Entity IDs, `EntityId`
/// should always be used in place of `u64` to ensure forwards compatiblity
/// with potential implementation changes.
pub type EntityId = u64;

/// Marker trait for types which can be used as components.
///
/// `Component` is automatically implemented for all eligible types by the
/// provided `impl`, so you don't have to worry about this! Hooray!
pub trait Component: Clone + Any {}
impl<T: Clone + Any> Component for T {}

/// Primary data structure containing entity and component data.
///
/// Notice that `Ecs` itself has no type parameters. Its methods to interact
/// with components do, but runtime reflection (via `std::any::TypeId`) is
/// used to retrieve components from an internal `HashMap`. Therefore, you
/// can create and use any data structure you want for components, provided
/// that they implement `Clone`.
///
/// Tip: `#[derive(Clone)]` will make your life a little easier :-)
pub struct Ecs {
  ids: EntityId,
  data: HashMap<EntityId, ComponentSet>,
}

struct ComponentSet {
  map: HashMap<TypeId, Box<Any>>,
}

/// Iterator for entity IDs.
pub struct EntityIdIter<'a> {
  iter: Keys<'a, EntityId, ComponentSet>
}

/// Iterator for entity IDs filtered by component.
pub struct EntityComponentFilter<'a, C: Component> {
  iter: Iter<'a, EntityId, ComponentSet>,
  _p: PhantomData<C>,
}

/// Iterator that yields references to ECS components.
pub struct ComponentIter<'a> {
  iter: Iter<'a, TypeId, Box<Any>>
}

/// Iterator that yields mutable references to ECS components.
pub struct ComponentIterMut<'a> {
  iter: IterMut<'a, TypeId, Box<Any>>
}

impl<'a> Iterator for EntityIdIter<'a> {
  type Item = EntityId;
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|id| *id)
  }
}

impl<'a, C> Iterator for EntityComponentFilter<'a, C> where C: Component {
  type Item = (EntityId, &'a C);
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.iter.next() {
        None => return None,
        Some((id, set)) => match set.borrow::<C>() {
          Some(cmp) => return Some((*id, cmp)),
          None => continue,
        }
      }
    }
  }
}

impl<'a> Iterator for ComponentIter<'a> {
  type Item = &'a Any;
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|(_, v)| v.deref())
  }
}

impl<'a> Iterator for ComponentIterMut<'a> {
  type Item = &'a mut Any;
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|(_, v)| v.deref_mut())
  }
}

impl Default for ComponentSet {
  fn default() -> Self {
    ComponentSet {
      map: HashMap::new(),
    }
  }
}

impl ComponentSet {
  fn set<C: Component>(&mut self, component: &C) -> Option<C> {
    self.map.insert(TypeId::of::<C>(), Box::new(component.clone())).map(|old| {
      *old.downcast::<C>().ok().expect(
        "ComponentSet.set: internal downcast error")
    })
  }
  fn get<C: Component>(&self) -> Option<C> {
    self.borrow::<C>().map(Clone::clone)
  }
  fn contains<C: Component>(&self) -> bool {
    self.map.contains_key(&TypeId::of::<C>())
  }
  fn borrow<C: Component>(&self) -> Option<&C> {
    self.map.get(&TypeId::of::<C>()).map(|c| {
      c.downcast_ref()
        .expect("ComponentSet.borrow: internal downcast error")
    })
  }
  fn borrow_mut<C: Component>(&mut self) -> Option<&mut C> {
    match self.map.get_mut(&TypeId::of::<C>()) {
      Some(c) => Some(c.downcast_mut()
        .expect("ComponentSet.borrow_mut: internal downcast error")),
      None => None,
    }
  }
  fn iter(&self) -> ComponentIter {
    ComponentIter{iter: self.map.iter()}
  }
  fn iter_mut(&mut self) -> ComponentIterMut {
    ComponentIterMut{iter: self.map.iter_mut()}
  }
}

impl Default for Ecs {
  fn default() -> Self {
    Ecs {
      ids: 0,
      data: HashMap::new(),
    }
  }
}

impl Ecs {
  /// Create a new and empty entity-component system (ECS).
  pub fn new() -> Self { Self::default() }
  /// Create a new entity in the ECS with no components, and return its ID.
  pub fn create_entity(&mut self) -> EntityId {
    self.ids += 1;
    self.data.insert(self.ids, Default::default());
    self.ids
  }
  /// Return `true` if the provided entity exists in this system.
  pub fn has_entity(&self, id: EntityId) -> bool {
    self.data.contains_key(&id)
  }
  /// Destroy the provided entity, automatically removing any of its
  /// components.
  ///
  /// Return `true` if the entity existed and was successfully deleted;
  /// return `false` if the provided entity ID was not found in the system.
  pub fn destroy_entity(&mut self, id: EntityId) -> bool {
    self.data.remove(&id).is_some()
  }
  /// For the specified entity, add a component of type `C` to the system.
  ///
  /// If the entity already has a component of type `C`, it is returned
  /// and overwritten.
  ///
  /// To modify a component in place, see `borrow_mut`.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn set<C: Component>(&mut self, id: EntityId, comp: &C)
    -> Option<C>
  {
    self.data.get_mut(&id)
      .expect(&format!("Ecs.set: nil entity {}", id))
      .set(comp)
  }
  /// Return a clone of the requested entity's component of type `C`, or
  /// `None` if the entity does not have that component.
  ///
  /// To examine a component without copying, see `borrow`.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn get<C: Component>(&self, id: EntityId) -> Option<C> {
    self.data.get(&id)
      .expect(&format!("Ecs.get: nil entity {}", id))
      .get::<C>()
  }
  /// Return `true` if the specified entity has a component of type `C
  /// in the system.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn has<C: Component>(&self, id: EntityId) -> bool {
    self.data.get(&id)
      .expect(&format!("Ecs.has: nil entity {}", id))
      .contains::<C>()
  }
  /// Return a reference to the requested entity's component of type `C`, or
  /// `None` if the entity does not have that component.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn borrow<C: Component>(&self, id: EntityId) -> Option<&C> {
    self.data.get(&id)
      .expect(&format!("Ecs.borrow: nil entity {}", id))
      .borrow()
  }
  /// Return a mutable reference to the requested entity's component of type
  /// `C`, or `None` if the entity does not have that component.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn borrow_mut<C: Component>(&mut self, id: EntityId)
    -> Option<&mut C>
  {
    self.data.get_mut(&id)
      .expect(&format!("Ecs.borrow: nil entity {}", id))
      .borrow_mut()
  }
  /// Return an iterator over every ID in the system.
  pub fn iter_ids(&self) -> EntityIdIter {
    EntityIdIter{iter: self.data.keys()}
  }
  /// Return a vector containing copies of every ID in the system.
  ///
  /// Useful for accessing entity IDs without borrowing the ECS.
  pub fn collect_ids(&self) -> Vec<EntityId> {
    self.iter_ids().collect()
  }
  /// For every entity with a component of type `C`, yield a tuple containing
  /// the entity's ID as well as a reference to its `C` component.
  pub fn iter_with<C: Component>(&self) -> EntityComponentFilter<C> {
    EntityComponentFilter{iter: self.data.iter(), _p: PhantomData}
  }
  /// Return a vector containing the results of `iter_with`, but with cloned
  /// components (rather than references).
  ///
  /// Useful for accessing all entities with a given component without
  /// initiating a borrow of the ECS.
  pub fn collect_with<C: Component>(&self) -> Vec<(EntityId, C)> {
    self.iter_with::<C>().map(|(id, c)| (id, c.clone())).collect()
  }
  /// Collect all entities with `C1` and `C2` components.
  pub fn collect_with_2<C1: Component, C2: Component>(&self)
      -> Vec<(EntityId, C1, C2)>
  {
    let mut ret = Vec::with_capacity(self.data.len());
    for id in self.iter_ids() {
      match (self.get::<C1>(id), self.get::<C2>(id)) {
        (Some(c1), Some(c2)) => ret.push((id, c1, c2)),
        _ => {}
      }
    }
    ret
  }
  /// Collect all entities with `C1`, `C2`, and `C3` components.
  pub fn collect_with_3<C1: Component, C2: Component, C3: Component>(&self)
      -> Vec<(EntityId, C1, C2, C3)>
  {
    let mut ret = Vec::with_capacity(self.data.len());
    for id in self.iter_ids() {
      match (self.get::<C1>(id), self.get::<C2>(id), self.get::<C3>(id)) {
        (Some(c1), Some(c2), Some(c3)) => ret.push((id, c1, c2, c3)),
        _ => {}
      }
    }
    ret
  }
  /// Return an iterator yielding references to all components of the
  /// requested entity.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn iter_components(&self, id: EntityId) -> ComponentIter {
    match self.data.get(&id) {
      Some(components) => components.iter(),
      None => panic!("Ecs.iter_components: nil entity {}", id),
    }
  }
  /// Return an iterator yielding mutable references to all components of the
  /// requested entity.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn iter_components_mut(&mut self, id: EntityId) -> ComponentIterMut
  {
    match self.data.get_mut(&id) {
      Some(components) => components.iter_mut(),
      None => panic!("Ecs.iter_components_mut: nil entity {}", id),
    }
  }
}
