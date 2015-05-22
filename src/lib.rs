use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut, Keys};
use std::ops::{Deref, DerefMut};

pub type EntityId = u64;

pub struct Ecs {
  ids: EntityId,
  data: HashMap<EntityId, ComponentSet>,
}

struct ComponentSet {
  map: HashMap<TypeId, Box<Any>>,
}

pub struct EntityIdIter<'a> {
  iter: Keys<'a, EntityId, ComponentSet>
}

pub struct ComponentIter<'a> {
  iter: Iter<'a, TypeId, Box<Any>>
}

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
  fn contains<C: Any>(&self) -> bool {
    self.map.contains_key(&TypeId::of::<C>())
  }
  fn consume<C: Any>(&mut self, component: C) -> Option<Box<C>> {
    self.map.insert(TypeId::of::<C>(), Box::new(component)).map(|old| {
      old.downcast::<C>().ok().expect(
        "ComponentSet.consume: internal downcast error")
    })
  }
  fn borrow<C: Any>(&self) -> Option<&C> {
    match self.map.get(&TypeId::of::<C>()) {
      Some(c) => Some(c.downcast_ref()
        .expect("ComponentSet.get: internal downcast error")),
      None => None,
    }
  }
  fn borrow_mut<C: Any>(&mut self) -> Option<&mut C> {
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
  /* instantiation */
  pub fn new() -> Self { Self::default() }
  /* entities */
  pub fn create_entity(&mut self) -> EntityId {
    self.ids += 1;
    self.data.insert(self.ids, Default::default());
    self.ids
  }
  pub fn has_entity(&self, id: EntityId) -> bool {
    self.data.contains_key(&id)
  }
  pub fn destroy_entity(&mut self, id: EntityId) {
    match self.data.remove(&id) {
      Some(..) => (), // ok
      None => panic!("Ecs.destroy_entity: nil entity {}", id),
    }
  }
  /* components */
  pub fn set<C: Any + Clone>(&mut self, id: EntityId, comp: &C)
    -> Option<C>
  {
    self.data.get_mut(&id)
      .expect(&format!("Ecs.set: nil entity {}", id))
      .set(comp)
  }
  pub fn get<C: Any + Clone>(&self, id: EntityId) -> Option<C> {
    self.data.get(&id)
      .expect(&format!("Ecs.get: nil entity {}", id))
      .get::<C>()
  }
  pub fn has<C: Any>(&self, id: EntityId) -> bool {
    match self.data.get(&id) {
      Some(components) => components.contains::<C>(),
      None => panic!("Ecs.has: nil entity {}", id),
    }
  }
  pub fn consume<C: Any>(&mut self, id: EntityId, component: C)
    -> Option<Box<C>>
  {
    match self.data.get_mut(&id) {
      Some(components) => components.consume(component),
      None => panic!("Ecs.consume: nil entity {}", id)
    }
  }
  pub fn borrow<C: Any>(&self, id: EntityId) -> Option<&C> {
    match self.data.get(&id) {
      Some(components) => components.borrow::<C>(),
      None => panic!("Ecs.borrow: nil entity {}", id),
    }
  }
  pub fn borrow_mut<C: Any>(&mut self, id: EntityId)
    -> Option<&mut C>
  {
    match self.data.get_mut(&id) {
      Some(components) => components.borrow_mut::<C>(),
      None => panic!("Ecs.borrow_mut: nil entity {}", id),
    }
  }
  pub fn iter_ids(&self) -> EntityIdIter {
    EntityIdIter{iter: self.data.keys()}
  }
  pub fn collect_ids(&self) -> Vec<EntityId> {
    self.iter_ids().collect()
  }
  pub fn iter_components(&self, id: EntityId) -> ComponentIter {
    match self.data.get(&id) {
      Some(components) => components.iter(),
      None => panic!("Ecs.iter_components: nil entity {}", id),
    }
  }
  pub fn iter_components_mut(&mut self, id: EntityId) -> ComponentIterMut
  {
    match self.data.get_mut(&id) {
      Some(components) => components.iter_mut(),
      None => panic!("Ecs.iter_components_mut: nil entity {}", id),
    }
  }
}
