//! Simple entity-component system. Macro-free stable Rust using compile-time reflection!
//!
//! # Example
//! ```
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
//!     // The Ecs.set method returns an EcsResult that will be set to Err if
//!     // the specified entity does not exist. If you're sure that the entity exists, suppress
//!     // Rust's "unused result" warning by prefixing your calls to set(..) with "let _ = ..."
//!     let _ = system.set(forrest, Age{years: 22});
//!     let _ = system.set(forrest, Iq{points: 75}); // "I may not be a smart man..."
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
//!     let _ = system.set(forrest, older);
//!
//!     // Modify a component in-place with a mutable borrow
//!     system.borrow_mut::<Iq>(forrest).map(|iq| iq.points += 5);
//!
//!     // Inspect a component in-place without cloning
//!     assert_eq!(system.borrow::<Age>(forrest), Ok(&Age{years: 23}));
//!
//!     // Inspect a component via cloning
//!     assert_eq!(system.get::<Iq>(forrest), Ok(Iq{points: 80}));
//!
//! }
//! ```

#![allow(unknown_lints)] // for rust-clippy
#![warn(missing_docs)]
use std::any::{TypeId, Any};
use std::collections::{HashMap, HashSet};

type IdNumber = u64;

/// Value type representing an entity in the entity-component system.
///
/// To avoid duplicate entity IDs, these can only be created by calling `Ecs.create_entity()`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityId(IdNumber);

/// Error type for ECS results that require a specific entity or component.
#[derive(Debug, PartialEq, Eq)]
pub enum NotFound {
    /// A requested entity ID was not present in the system.
    Entity(EntityId),
    /// A requested component was not present on an entity.
    Component(TypeId),
}

/// Result type for ECS operations that may fail without a specific entity or component.
pub type EcsResult<T> = Result<T, NotFound>;

/// Marker trait for types which can be used as components.
///
/// `Component` is automatically implemented for all eligible types by the
/// provided `impl`, so you don't have to worry about this. Hooray!
pub trait Component: Any {}
impl<T: Any> Component for T {}

/// List of component types.
///
/// The `Ecs` methods `has_all` and `collect_with` each take a `ComponentFilter` instance. The
/// recommended way to actually create a `ComponentFilter` is with the
/// [`component_filter!` macro](macro.component_filter!.html).
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct ComponentFilter {
    set: HashSet<TypeId>,
}

impl ComponentFilter {
    /// Create a new component filter.
    pub fn new() -> Self {
        Default::default()
    }
    /// Add component type `C` to the filter.
    pub fn add<C: Component>(&mut self) {
        self.set.insert(TypeId::of::<C>());
    }
    /// Remove component type `C` from the filter.
    pub fn remove<C: Component>(&mut self) {
        self.set.remove(&TypeId::of::<C>());
    }
    /// Return `true` if the filter already contains component type `C`; otherwise `false`.
    pub fn contains<C: Component>(&mut self) -> bool {
        self.set.contains(&TypeId::of::<C>())
    }
    /// Create a component filter from a vector/slice of `TypeId` instances. (Not recommended;
    /// used by the `component_filter!` macro.)
    pub fn from_slice(slice: &[TypeId]) -> Self {
        let mut this = Self::new();
        for type_id in slice.iter() {
            this.set.insert(*type_id);
        }
        this
    }
    /// Return an iterator over all the contained component types.
    #[allow(needless_lifetimes)] // https://github.com/Manishearth/rust-clippy/issues/740
    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = TypeId> + 'a> {
        Box::new(self.set.iter().cloned())
    }
}

/// Create a `ComponentFilter` by type name.
///
/// If you want all entities with components `Foo` and `Bar`:
///
/// ```
/// #[macro_use] // The macro won't be imported without this flag!
/// extern crate recs;
/// use recs::{Ecs, EntityId};
///
/// struct Foo;
/// struct Bar;
///
/// fn main() {
///     let sys = Ecs::new();
///     // ... add some entities and components ...
///     let mut ids: Vec<EntityId> = Vec::new();
///     let filter = component_filter!(Foo, Bar);
///     sys.collect_with(&filter, &mut ids);
///     for id in ids {
///         // Will only iterate over entities that have been assigned both `Foo` and `Bar`
///         // components
///     }
/// }
/// ```
#[macro_export]
macro_rules! component_filter {
  ($($x:ty),*) => (
    $crate::ComponentFilter::from_slice(
      &vec![$(std::any::TypeId::of::<$x>()),*]
    )
  );
  ($($x:ty,)*) => (component_filter![$($x),*])
}

/// Primary data structure containing entity and component data.
///
/// Notice that `Ecs` itself has no type parameters. Its methods to interact
/// with components do, but runtime reflection (via `std::any::TypeId`) is
/// used to retrieve components from an internal `HashMap`. Therefore, you
/// can create and use any data structure you want for components.
///
/// Tip: using `#[derive(Clone)]` on your component types will make your life a little easier by
/// enabling the `get` method, which avoids locking down the `Ecs` with a mutable or immutable
/// borrow.
#[derive(Default)]
pub struct Ecs {
    ids: IdNumber,
    data: HashMap<EntityId, ComponentMap>,
}

#[derive(Default)]
struct ComponentMap {
    map: HashMap<TypeId, Box<Any>>,
}

impl ComponentMap {
    fn set<C: Component>(&mut self, component: C) -> Option<C> {
        self.map
            .insert(TypeId::of::<C>(), Box::new(component))
            .map(|old| *old.downcast::<C>().expect("ComponentMap.set: internal downcast error"))
    }
    fn borrow<C: Component>(&self) -> EcsResult<&C> {
        self.map
            .get(&TypeId::of::<C>())
            .map(|c| {
                c.downcast_ref()
                 .expect("ComponentMap.borrow: internal downcast error")
            })
            .ok_or_else(|| NotFound::Component(TypeId::of::<C>()))
    }
    #[allow(map_clone)]
    fn get<C: Component + Clone>(&self) -> EcsResult<C> {
        self.borrow::<C>()
            .map(Clone::clone)
    }
    fn contains_type_id(&self, id: &TypeId) -> bool {
        self.map.contains_key(id)
    }
    fn contains<C: Component>(&self) -> bool {
        self.contains_type_id(&TypeId::of::<C>())
    }
    fn borrow_mut<C: Component>(&mut self) -> EcsResult<&mut C> {
        match self.map.get_mut(&TypeId::of::<C>()) {
            Some(c) => {
                Ok(c.downcast_mut()
                    .expect("ComponentMap.borrow_mut: internal downcast error"))
            }
            None => Err(NotFound::Component(TypeId::of::<C>())),
        }
    }
}

impl Ecs {
    /// Create a new and empty entity-component system (ECS).
    pub fn new() -> Self {
        Default::default()
    }
    /// Create a new entity in the ECS without components and return its ID.
    pub fn create_entity(&mut self) -> EntityId {
        let new_id = EntityId(self.ids);
        self.ids += 1;
        self.data.insert(new_id, Default::default());
        new_id
    }
    /// Return `true` if the provided entity exists in the system.
    pub fn exists(&self, id: EntityId) -> bool {
        self.data.contains_key(&id)
    }
    /// Destroy the provided entity, automatically removing any of its components.
    ///
    /// Return `NotFound::Entity` if the entity does not exist or was already deleted.
    pub fn destroy_entity(&mut self, id: EntityId) -> EcsResult<()> {
        self.data.remove(&id).map(|_| ()).ok_or_else(|| NotFound::Entity(id))
    }
    /// For the specified entity, add a component of type `C` to the system.
    ///
    /// If the entity already has a component `prev` of type `C`, return `Some(prev)`. If not,
    /// return `None`. If the entity does not exist, return `NotFound::Entity`.
    ///
    /// To modify an existing component in place, see `borrow_mut`.
    pub fn set<C: Component>(&mut self, id: EntityId, comp: C) -> EcsResult<Option<C>> {
        self.data
            .get_mut(&id)
            .ok_or_else(|| NotFound::Entity(id))
            .map(|map| map.set(comp))
    }
    /// Return a clone of the requested entity's component of type `C`, or a `NotFound` variant
    /// if the entity does not exist or does not have that component.
    ///
    /// To examine or modify a component without making a clone, see `borrow` and `borrow_mut`.
    pub fn get<C: Component + Clone>(&self, id: EntityId) -> EcsResult<C> {
        self.data
            .get(&id)
            .ok_or_else(|| NotFound::Entity(id))
            .and_then(|map| map.get())
    }
    /// Return `true` if the specified entity has a component of type `C` in the system, or
    /// `NotFound::Entity` if the entity does not exist.
    pub fn has<C: Component>(&self, id: EntityId) -> EcsResult<bool> {
        self.data
            .get(&id)
            .ok_or_else(|| NotFound::Entity(id))
            .map(|map| map.contains::<C>())
    }
    /// Return `true` if each component type in the filter is present on the entity `id`.
    pub fn has_all(&self, id: EntityId, set: &ComponentFilter) -> EcsResult<bool> {
        let map = try!(self.data.get(&id).ok_or_else(|| NotFound::Entity(id)));
        Ok(set.iter().all(|type_id| map.contains_type_id(&type_id)))
    }
    /// Return a shared reference to the requested entity's component of type `C`, or a
    /// `NotFound` variant if the entity does not exist or does not have that component.
    pub fn borrow<C: Component>(&self, id: EntityId) -> EcsResult<&C> {
        self.data
            .get(&id)
            .ok_or_else(|| NotFound::Entity(id))
            .and_then(|map| map.borrow())
    }
    /// Return a mutable reference to the requested entity's component of type `C`, or a
    /// `NotFound` variant if the entity does not exist or does not have that component.
    pub fn borrow_mut<C: Component>(&mut self, id: EntityId) -> EcsResult<&mut C> {
        self.data
            .get_mut(&id)
            .ok_or_else(|| NotFound::Entity(id))
            .and_then(|map| map.borrow_mut())
    }
    /// Return an iterator over every ID in the system.
    #[allow(needless_lifetimes)] // https://github.com/Manishearth/rust-clippy/issues/740
    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = EntityId> + 'a> {
        Box::new(self.data.keys().cloned())
    }
    /// Collect all entity IDs into a vector (after emptying the vector).
    ///
    /// Useful for accessing entity IDs without borrowing the ECS.
    pub fn collect(&self, dest: &mut Vec<EntityId>) {
        dest.clear();
        dest.extend(self.iter());
    }
    /// Collect the IDs of all entities containing a certain set of component types into a vector.
    ///
    /// After calling this method, the vector `dest` will contain *only* those entities who have
    /// at least each type of component specified in the filter.
    pub fn collect_with<'a>(&'a self, components: &'a ComponentFilter, dest: &mut Vec<EntityId>) {
        let ids = self.data.keys().cloned();
        dest.clear();
        dest.extend(ids.filter(|e| {
            self.has_all(*e, components)
                .expect("Ecs.collect_with: internal id filter error")
        }))
    }
}
