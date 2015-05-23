//! Simple entity-component system. Pure Rust (macro-free)!
//!
//! # Example
//! ```
//! extern crate recs;
//! use recs::{Ecs, EntityId};
//!
//! #[derive(Clone, PartialEq)]
//! struct Age{years: u32}
//!
//! #[derive(Clone, PartialEq)]
//! struct Brain{iq: i32}
//!
//! fn main() {
//!     // Create an ECS instance
//!     let mut ecs: Ecs = Ecs::new();
//!     // Add entity to the system
//!     let me: EntityId = ecs.create_entity();
//!     // Attach component to the entity
//!     ecs.set(me, &Age{years: 22});
//!     // Get attached component data from entity
//!     let older = ecs.get::<Age>(me).unwrap().years + 1;
//!     // Modify an entity's component
//!     ecs.set(me, &Age{years: older});
//!     // It works!
//!     assert!(ecs.get::<Age>(me) == Some(Age{years: 23}));
//!     assert!(ecs.get::<Brain>(me) == None); // Aw man...
//! }
//! ```
// TODO iterate every E/C pair
// TODO iterate every E and its C's
use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut, Keys};
use std::ops::{Deref, DerefMut};

/// Value type representing an entry in the entity-component system.
///
/// `EntityId` is an alias to `u64`. When storing Entity IDs, `EntityId`
/// should always be used in place of `u64` to ensure forwards compatiblity
/// with potential implementation changes.
pub type EntityId = u64;

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
  fn set<C: Any + Clone>(&mut self, component: &C) -> Option<C> {
    self.map.insert(TypeId::of::<C>(), Box::new(component.clone())).map(|old| {
      *old.downcast::<C>().ok().expect(
        "ComponentSet.set: internal downcast error")
    })
  }
  fn get<C: Any + Clone>(&self) -> Option<C> {
    self.borrow::<C>().map(Clone::clone)
  }
  fn contains<C: Any + Clone>(&self) -> bool {
    self.map.contains_key(&TypeId::of::<C>())
  }
  fn borrow<C: Any + Clone>(&self) -> Option<&C> {
    self.map.get(&TypeId::of::<C>()).map(|c| {
      c.downcast_ref()
        .expect("ComponentSet.borrow: internal downcast error")
    })
  }
  fn borrow_mut<C: Any + Clone>(&mut self) -> Option<&mut C> {
    match self.map.get_mut(&TypeId::of::<C>()) {
      Some(c) => Some(c.downcast_mut()
        .expect("ComponentSet.get_mut: internal downcast error")),
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
  pub fn set<C: Any + Clone>(&mut self, id: EntityId, comp: &C)
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
  pub fn get<C: Any + Clone>(&self, id: EntityId) -> Option<C> {
    self.data.get(&id)
      .expect(&format!("Ecs.get: nil entity {}", id))
      .get::<C>()
  }
  /// Return `true` Panics if the requested entity has a component of type `C`.
  pub fn has<C: Any + Clone>(&self, id: EntityId) -> bool {
    self.data.get(&id)
      .expect(&format!("Ecs.has: nil entity {}", id))
      .contains::<C>()
  }
  /// Return a reference to the requested entity's component of type `C`, or
  /// `None` if the entity does not have that component.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn borrow<C: Any + Clone>(&self, id: EntityId) -> Option<&C> {
    self.data.get(&id)
      .expect(&format!("Ecs.borrow: nil entity {}", id))
      .borrow()
  }
  /// Return a mutable reference to the requested entity's component of type
  /// `C`, or `None` if the entity does not have that component.
  ///
  /// # Panics
  /// Panics if the requested entity does not exist.
  pub fn borrow_mut<C: Any + Clone>(&mut self, id: EntityId)
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
