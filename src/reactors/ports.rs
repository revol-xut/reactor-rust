#![feature(maybe_uninit_ref)]


use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::reactors::assembler::AssemblyError;
use crate::reactors::id::{GlobalId, Identified, PortId};
use crate::reactors::ports::BindStatus::Unbound;

/// The nature of a port (input or output)
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PortKind { Input, Output }


/// Represents a port, which carries values of type `T`.
/// Ports reify the data inputs and outputs of a reactor.
///
/// They may be bound to another port, in which case the
/// upstream port forwards all values to the output port
/// (logically instantaneously). A port may have only one
/// upstream binding.
///
/// Output ports may also be explicitly [set](super::ReactionCtx::set_port)
/// within a reaction, in which case they may not have an
/// upstream port binding.
///
pub struct Port<T> {
    kind: PortKind,
    id: PortId,
    _phantom_t: PhantomData<T>,

    upstream_binding: Rc<RefCell<Binding<T>>>,
}

impl<T> Port<T> {
    pub(in super) fn new(kind: PortKind, global_id: GlobalId, initial: T) -> Self {
        Port::<T> {
            kind,
            id: PortId(global_id),
            _phantom_t: PhantomData,
            upstream_binding: Rc::new(RefCell::new((Unbound, Rc::new(PortEquivClass::<T>::new(initial))))),
        }
    }

    pub fn kind(&self) -> PortKind {
        self.kind
    }

    pub fn is_output(&self) -> bool {
        self.kind == PortKind::Output
    }

    pub fn is_input(&self) -> bool {
        self.kind == PortKind::Input
    }


    pub(in super) fn port_id(&self) -> &PortId {
        &self.id
    }

    pub(in super) fn bind_status(&self) -> BindStatus {
        let binding: &RefCell<Binding<T>> = Rc::borrow(&self.upstream_binding);
        let (status, _) = *binding.borrow();
        status
    }

    pub(in super) fn downstream_ports(&self) -> HashSet<PortId> {
        let binding: &RefCell<Binding<T>> = Rc::borrow(&self.upstream_binding);
        let (_, class) = &*binding.borrow();
        let c: &PortEquivClass<T> = Rc::borrow(class);
        let map = &*c.downstreams.borrow();
        HashSet::from_iter(map.keys().map(Clone::clone))
    }


    pub(in super) fn forward_to(&self, downstream: &Port<T>) -> Result<(), AssemblyError> {
        let mut mut_downstream_cell = (&downstream.upstream_binding).borrow_mut();
        let (downstream_status, ref downstream_class) = *mut_downstream_cell;

        match downstream_status {
            #[cold] BindStatus::PortBound => Err(AssemblyError::InvalidBinding(String::from("Downstream is already bound to another port"),
                                                                               self.global_id().clone(),
                                                                               downstream.global_id().clone())),
            // #[cold] BindStatus::DependencyBound => Err(AssemblyError::InvalidBinding("Port {} receives values from a reaction", self.global_id().clone(), downstream.global_id().clone())),
            BindStatus::Unbound => {
                let mut self_cell = self.upstream_binding.borrow_mut();
                let (_, my_class) = self_cell.deref_mut();

                my_class.downstreams.borrow_mut().insert(
                    downstream.id.clone(),
                    Rc::clone(&downstream.upstream_binding),
                );

                let new_binding = (BindStatus::PortBound, Rc::clone(&my_class));

                downstream_class.check_cycle(&self.id, &downstream.id)?;

                downstream_class.set_upstream(my_class);
                *mut_downstream_cell.deref_mut() = new_binding;

                Ok(())
            }
        }
    }
}

impl<T> Port<T> {
    pub(in crate) fn get_mut(&self) -> Rc<RefCell<T>> {
        let cell: &RefCell<Binding<T>> = self.upstream_binding.borrow();
        let cell_ref: Ref<Binding<T>> = cell.borrow();

        let (_, class) = cell_ref.deref();

        Rc::clone(&class.deref().cell)
    }

    pub(in crate) fn copy_get(&self) -> T where T: Copy {
        let cell: &RefCell<Binding<T>> = self.upstream_binding.borrow();
        let cell_ref: Ref<Binding<T>> = RefCell::borrow(cell);
        let binding: &Binding<T> = cell_ref.deref();

        let (_, class) = binding;

        let class_cell: &PortEquivClass<T> = Rc::borrow(class);

        let rc = Rc::clone(&class_cell.cell);
        let cell_borrow: Ref<T> = rc.deref().borrow();
        T::clone(cell_borrow.deref())
    }

    pub(in crate) fn set(&self, new_value: T) {
        assert!(self.bind_status() == Unbound, "Cannot set a bound port ({})", self.global_id());

        let cell: &RefCell<Binding<T>> = self.upstream_binding.borrow();
        let cell_ref: Ref<Binding<T>> = RefCell::borrow(cell);
        let binding: &Binding<T> = cell_ref.deref();

        let (_, class) = binding;

        let class_cell: &PortEquivClass<T> = Rc::borrow(class);

        // TODO this creates a ManuallyDrop
        // let prev = class_cell.cell.get();
        // unsafe { std::mem::drop(prev.assume_init()); }
        // class_cell.cell.set(MaybeUninit::new(new_value))

        class_cell.cell.replace(new_value);
    }
}


impl<T> Identified for Port<T> {
    fn global_id(&self) -> &GlobalId {
        &self.id.global_id()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(in super) enum BindStatus {
    Unbound,
    PortBound,
    // DependencyBound,
}


type Binding<T> = (BindStatus, Rc<PortEquivClass<T>>);


/// An equivalence class is a set of ports that are
/// bound together transitively. Then, if anyone is
/// set (there can be only one, that is unbound), then
/// the value must be forwarded to all the others.
///
/// No forwarding actually happens. Ports of the same
/// equivalence class have a reference to the equivalence class,
/// which has a unique cell to store data.
struct PortEquivClass<T> {
    /// This the container for the value
    cell: Rc<RefCell<T>>,

    /// This is the set of ports that are "forwarded to".
    /// When you bind 2 ports A -> B, then the binding of B
    /// is updated to point to the equiv class of A. The downstream
    /// field of that equiv class is updated to contain B.
    ///
    /// Why?
    /// When you have bound eg A -> B and *then* bind U -> A,
    /// then both the equiv class of A and B (the downstream of A)
    /// need to be updated to point to the equiv class of U
    ///
    /// Coincidentally, this means we can track transitive
    /// cyclic port dependencies:
    /// - say you have bound A -> B, then B -> C
    /// - so all three refer to the equiv class of A, whose downstream is now {B, C}
    /// - if you then try binding C -> A, then we can know
    /// that C is in the downstream of A, indicating that there is a cycle.
    downstreams: RefCell<HashMap<PortId, Rc<RefCell<Binding<T>>>>>,
}

impl<T> PortEquivClass<T> {
    fn new(initial: T) -> Self {
        PortEquivClass {
            cell: Rc::new(RefCell::new(initial)),
            downstreams: Default::default(),
        }
    }

    fn check_cycle(&self, upstream_id: &PortId, downstream_id: &PortId) -> Result<(), AssemblyError> {
        if (&*self.downstreams.borrow()).contains_key(upstream_id) {
            Err(AssemblyError::CyclicDependency(format!("Port {} is already in the downstream of port {}", upstream_id, downstream_id)))
        } else {
            Ok(())
        }
    }

    /// This updates all downstreams to point to the given equiv class instead of `self`
    fn set_upstream(&self, new_binding: &Rc<PortEquivClass<T>>) {
        for (_, cell_rc) in &*self.downstreams.borrow() {
            let cell: &RefCell<Binding<T>> = Rc::borrow(cell_rc);
            let mut ref_mut = cell.borrow_mut();
            *ref_mut.deref_mut() = (ref_mut.0, Rc::clone(new_binding));
        }
    }
}
