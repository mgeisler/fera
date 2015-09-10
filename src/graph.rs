use iter::{IteratorExt, Map1};

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::IndexMut;

use rand::Rng;


pub trait Graph: Basic + BasicProps { }

impl<G> Graph for G
    where G: Basic + BasicProps { }


pub trait Types<G: Graph>: IterTypes<G> { }

impl<G, T> Types<G> for T
    where G: Graph,
          T: IterTypes<G> { }


// Aliases

pub type Vertex<G> = <G as Basic>::Vertex;
pub type Edge<G> = <G as Basic>::Edge;

pub type IterVertex<'a, G> = <&'a G as IterTypes<G>>::Vertex;
pub type IterEdge<'a, G> = <&'a G as IterTypes<G>>::Edge;
pub type IterInc<'a, G> = <&'a G as IterTypes<G>>::Inc;

pub type PropVertex<G, T> = <G as WithProps<T>>::Vertex;
pub type PropEdge<G, T> = <G as WithProps<T>>::Edge;

pub type VecVertex<G> = Vec<Vertex<G>>;
pub type VecEdge<G> = Vec<Edge<G>>;

pub type OptionVertex<G> = Option<Vertex<G>>;
pub type OptionEdge<G> = Option<Edge<G>>;


// Basic

// We are implementing lifetime polymorphism using the idea described in
// https://github.com/rust-lang/rfcs/blob/master/text/0195-associated-items.md#encoding-higher-kinded-types
//
// To understand the problem, see
// https://github.com/rust-lang/rfcs/blob/master/text/0195-associated-items.md#limitations
//
// When declaring and implementing methods that return types with lifetime polymorphism some care
// must be taken.
//
// 1 - On declaration, put bounds on methods, not on traits
//
//     // This generates an ICE (see https://github.com/rust-lang/rust/issues/23958)
//     trait Basic where for<'a> &'a Self: IterTypes<G> {
//           fn vertices<'a>(&'a self) -> IterVertex<Self>;
//     }
//
//     // This works (with item #2)
//     trait Basic {
//           fn vertices<'a>(&'a self) -> IterVertex<Self> where &'a Self: IterTypes<G>;
//     }
//
// 2 - On impl do not repeat the bound, use &'a (): Sized instead
//
//     // This do not compile (see https://github.com/rust-lang/rust/issues/28046)
//     impl Basic for StaticGraph {
//           fn vertices<'a>(&'a self) -> IterVertex<Self> where &'a Self: IterTypes<G>;
//     }
//
//     // This works
//     impl Basic for StaticGraph {
//           fn vertices<'a>(&'a self) -> IterVertex<Self> where &'a (): Sized;
//     }

// To be implemented on &'a G
pub trait IterTypes<G: Basic> {
    type Vertex: Iterator<Item=Vertex<G>>;
    type Edge: Iterator<Item=Edge<G>>;
    type Inc: Iterator<Item=Edge<G>>;
}

pub trait Basic: Sized {
    type Vertex: Copy + Eq + Hash + Debug;
    type Edge: Copy + Eq + Hash + Debug;

    // Vertices

    fn num_vertices(&self) -> usize;

    fn vertices<'a>(&'a self) -> IterVertex<Self>
        where &'a Self: IterTypes<Self>;

    fn choose_vertex<R: Rng>(&self, rng: &mut R) -> Vertex<Self>;


    // Edges

    fn num_edges(&self) -> usize;

    fn edges<'a>(&'a self) -> IterEdge<Self>
        where &'a Self: IterTypes<Self>;

    fn source(&self, e: Edge<Self>) -> Vertex<Self>;

    fn target(&self, e: Edge<Self>) -> Vertex<Self>;

    fn endvertices(&self, e: Edge<Self>) -> (Vertex<Self>, Vertex<Self>) {
        (self.source(e), self.target(e))
    }

    fn reverse(&self, e: Edge<Self>) -> Edge<Self>;

    fn opposite(&self, u: Vertex<Self>, e: Edge<Self>) -> Vertex<Self> {
        let (s, t) = self.endvertices(e);
        if u == s {
            t
        } else if u == t {
            s
        } else {
            panic!("u is not an endvertex of e");
        }
    }

    fn choose_edge<R: Rng>(&self, rng: &mut R) -> Edge<Self>;


    // Incidence

    fn degree(&self, v: Vertex<Self>) -> usize;

    fn inc_edges<'a>(&'a self, v: Vertex<Self>) -> IterInc<Self>
        where &'a Self: IterTypes<Self>;

    fn choose_inc_edge<R: Rng>(&self, rng: &mut R, v: Vertex<Self>) -> Edge<Self>;
}


// Properties

pub trait WithProps<T: Clone>: Basic {
    type Vertex: IndexMut<Vertex<Self>, Output=T>;
    type Edge: IndexMut<Edge<Self>, Output=T>;

    fn vertex_prop(&self, value: T) -> PropVertex<Self, T>;

    fn edge_prop(&self, value: T) -> PropEdge<Self, T>;
}

macro_rules! basic_props1 {
    ($($t1:ty),* ; $($t2:ty),* ) => (
        items! {
            pub trait BasicProps:
                $(WithProps<$t1> +)* { }

            impl<G> BasicProps for G where G:
                $(WithProps<$t2> +)* { }
        }
    )
}

macro_rules! basic_props2 {
    ($($t1:ty),* ; $($t2:ty),* ) => (
        basic_props1!{
            $($t1),+ , $(Vec<$t1>),+ , $(Option<$t1>),+ ;
            $($t2),+ , $(Vec<$t2>),+ , $(Option<$t2>),+
        }
    )
}

macro_rules! basic_props {
    ($($ty:ty),*) => (
        basic_props2!{
            Vertex<Self>, Edge<Self>, $($ty),+ ;
            Vertex<G>, Edge<G>, $($ty),+
        }
    )
}

basic_props! {
    bool,
    char,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    String
}


// Adjacency

pub trait Adj: Basic {
    fn neighbors<'a>(&'a self,
                     v: Vertex<Self>)
                     -> Map1<'a,
                             IterInc<'a, Self>,
                             Self,
                             fn(&'a Self, Edge<Self>) -> Vertex<Self>>
        where &'a Self: IterTypes<Self>
    {
        self.inc_edges(v).map1(self, Self::target)
    }
}

impl<G> Adj for G
    where G: Basic { }
