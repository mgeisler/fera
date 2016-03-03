use graph::*;
use ds::{IteratorExt, VecExt};
use builder::{Builder, WithBuilder};
use choose::Choose;
use vecprop::*;

use std::iter::{Cloned, Map};
use std::ops::{Index, Range};
use std::slice::Iter;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::fmt::Debug;

use rand::Rng;

// TODO: Rename to FastGraph
pub type StaticGraph = StaticGraphGeneric<u32, usize>;

pub trait Num: 'static + Eq + Copy + Clone + Debug + Hash +
               OptionItem<StaticVertex<Self>> +
               OptionItem<StaticEdge<Self>> {
    type Range: Iterator<Item = Self>;
    fn range(a: usize, b: usize) -> Self::Range;
    fn to_usize(self) -> usize;
    fn from_usize(v: usize) -> Self;
    fn is_valid(v: usize) -> bool;
    fn max() -> Self;
}

macro_rules! impl_num {
    ($t: ident) => (
        impl Num for $t {
            type Range = Range<$t>;

            #[inline(always)]
            fn range(a: usize, b: usize) -> Self::Range {
                Self::from_usize(a) .. Self::from_usize(b)
            }

            #[inline(always)]
            fn to_usize(self) -> usize {
                self as usize
            }

            #[inline(always)]
            fn from_usize(v: usize) -> Self {
                v as Self
            }

            #[inline(always)]
            fn is_valid(v: usize) -> bool {
                (v as u64) < (Self::max() as u64)
            }

            #[inline(always)]
            fn max() -> Self {
                use std;
                std::$t::MAX
            }
        }

        impl OptionItem<StaticVertex<$t>> for $t {
            #[inline(always)]
            fn new_none() -> Self {
                $t::max()
            }

            #[inline(always)]
            fn new_some(x: StaticVertex<$t>) -> Self {
                x
            }

            #[inline(always)]
            fn to_option(&self) -> Option<Self> {
                if <$t as OptionItem<StaticVertex<$t>>>::is_none(self) {
                    None
                } else {
                    Some(*self)
                }
            }
        }

        impl OptionItem<StaticEdge<$t>> for $t {
            #[inline(always)]
            fn new_none() -> Self {
                $t::max()
            }

            #[inline(always)]
            fn new_some(x: StaticEdge<$t>) -> Self {
                x.0
            }

            #[inline(always)]
            fn to_option(&self) -> Option<StaticEdge<$t>> {
                if <$t as OptionItem<StaticEdge<$t>>>::is_none(self) {
                    None
                } else {
                    Some(StaticEdge(*self))
                }
            }
        }
    )
}

impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(usize);


// StaticEdge

#[derive(Copy, Clone, Debug, Eq)]
pub struct StaticEdge<N: Num>(N);

impl<N: Num> Item for StaticEdge<N> {}

// TODO: Document the representation of StaticEdge
impl<N: Num> StaticEdge<N> {
    #[inline(always)]
    fn new(e: usize) -> Self {
        StaticEdge(Num::from_usize(2 * e + 1))
    }

    #[inline(always)]
    fn new_reverse(e: usize) -> Self {
        StaticEdge(Num::from_usize(2 * e))
    }

    #[inline(always)]
    fn reverse(self) -> Self {
        StaticEdge(Num::from_usize(Num::to_usize(self.0) ^ 1))
    }

    #[inline(always)]
    fn to_index(self) -> usize {
        Num::to_usize(self.0) / 2
    }
}

impl<N: Num> PartialEq for StaticEdge<N> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.to_index() == other.to_index()
    }
}

impl<N: Num> PartialOrd for StaticEdge<N> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_index().partial_cmp(&other.to_index())
    }
}

impl<N: Num> Ord for StaticEdge<N> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_index().cmp(&other.to_index())
    }
}

impl<N: Num> Hash for StaticEdge<N> {
    #[inline(always)]
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        self.to_index().hash(state)
    }
}


// Vertex

pub type StaticVertex<N> = N;

impl<N: Num> Item for StaticVertex<N> {}


// StaticGraphGeneric

#[derive(Clone)]
pub struct StaticGraphGeneric<V: Num, E: Num> {
    num_vertices: usize,
    endvertices: Vec<StaticVertex<V>>,
    inc: Vec<Vec<StaticEdge<E>>>,
}

impl<V: Num, E: Num> StaticGraphGeneric<V, E> {
    fn add_edge(&mut self, u: Vertex<Self>, v: Vertex<Self>) {
        self.endvertices.push(u);
        self.endvertices.push(v);
        let e = (self.endvertices.len() - 2) / 2;
        self.inc[Num::to_usize(u)].push(StaticEdge::new(e));
        self.inc[Num::to_usize(v)].push(StaticEdge::new_reverse(e));
    }

    fn inc(&self, v: Vertex<Self>) -> &Vec<StaticEdge<E>> {
        self.inc.index(Num::to_usize(v))
    }
}

impl<V: Num, E: Num> WithBuilder for StaticGraphGeneric<V, E> {
    type Builder = StaticGraphGenericBuilder<V, E>;
}

pub struct StaticGraphGenericBuilder<V: Num, E: Num> {
    g: StaticGraphGeneric<V, E>,
}

impl<V: Num, E: Num> Builder for StaticGraphGenericBuilder<V, E> {
    type Graph = StaticGraphGeneric<V, E>;

    fn new(num_vertices: usize, num_edges: usize) -> Self {
        // TODO: test this assert
        assert!(V::is_valid(num_vertices));
        StaticGraphGenericBuilder {
            g: StaticGraphGeneric {
                num_vertices: num_vertices,
                endvertices: Vec::with_capacity(2 * num_edges),
                inc: vec![vec![]; num_vertices],
            },
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.g.add_edge(Num::from_usize(u), Num::from_usize(v));
    }

    fn finalize(self) -> Self::Graph {
        // TODO: test this assert
        assert!(E::is_valid(self.g.endvertices.len()));
        self.g
    }

    fn finalize_(self) -> (Self::Graph, VecVertex<Self::Graph>, VecEdge<Self::Graph>) {
        // TODO: test this assert
        assert!(E::is_valid(self.g.endvertices.len()));
        let v = self.g.vertices().into_vec();
        let e = self.g.edges().into_vec();
        (self.g, v, e)
    }
}

impl<'a, V: Num, E: Num> Iterators<'a, StaticGraphGeneric<V, E>> for StaticGraphGeneric<V, E> {
    type Vertex = V::Range;
    type Edge = Map<Range<usize>, fn(usize) -> StaticEdge<E>>;
    type Inc = Cloned<Iter<'a, StaticEdge<E>>>;
}

impl<V: Num, E: Num> Basic for StaticGraphGeneric<V, E> {
    type Vertex = StaticVertex<V>;
    type OptionVertex = V;

    type Edge = StaticEdge<E>;
    type OptionEdge = E;

    fn num_vertices(&self) -> usize {
        self.num_vertices
    }

    fn vertices(&self) -> IterVertex<Self> {
        V::range(0, self.num_vertices)
    }

    #[inline(always)]
    fn source(&self, e: Edge<Self>) -> Vertex<Self> {
        self.endvertices[Num::to_usize(e.0) ^ 1]
    }

    #[inline(always)]
    fn target(&self, e: Edge<Self>) -> Vertex<Self> {
        self.endvertices[Num::to_usize(e.0)]
    }

    fn num_edges(&self) -> usize {
        self.endvertices.len() / 2
    }

    fn edges(&self) -> IterEdge<Self> {
        // TODO: iterate over 1, 3, 5, ...
        (0..self.num_edges()).map(StaticEdge::new)
    }

    #[inline(always)]
    fn reverse(&self, e: Edge<Self>) -> Edge<Self> {
        e.reverse()
    }

    // Inc

    #[inline(always)]
    fn degree(&self, v: Vertex<Self>) -> usize {
        self.inc[Num::to_usize(v)].len()
    }

    fn inc_edges(&self, v: Vertex<Self>) -> IterInc<Self> {
        self.inc(v).iter().cloned()
    }
}

impl<T: Clone, V: Num, E: Num> WithProps<T> for StaticGraphGeneric<V, E> {
    type Vertex = VecProp<fn (Vertex<Self>) -> usize, T>;
    type Edge = VecProp<fn (Edge<Self>) -> usize, T>;

    fn vertex_prop(&self, value: T) -> DefaultPropMutVertex<Self, T> {
        VecProp::new(StaticVertex::<V>::to_usize, Vec::with_value(value, self.num_vertices()))
    }

    fn edge_prop(&self, value: T) -> DefaultPropMutEdge<Self, T> {
        VecProp::new(StaticEdge::<E>::to_index, Vec::with_value(value, self.num_edges()))
    }
}

impl<V: Num, E: Num> Choose for StaticGraphGeneric<V, E> {
    fn choose_vertex<R: Rng>(&self, rng: &mut R) -> Vertex<Self> {
        Num::from_usize(rng.gen_range(0, self.num_vertices()))
    }

    fn choose_edge<R: Rng>(&self, rng: &mut R) -> Edge<Self> {
        StaticEdge::new(rng.gen_range(0, self.num_edges()))
    }

    fn choose_inc_edge<R: Rng>(&self, rng: &mut R, v: Vertex<Self>) -> Edge<Self> {
        self.inc(v)[rng.gen_range(0, self.degree(v))]
    }
}


// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use ds::IteratorExt;
    use graph::*;
    use builder::*;
    use tests::*;

    #[test]
    fn builder() {
        let mut builder = StaticGraph::builder(3, 1);

        builder.add_edge(0, 1);
        builder.add_edge(1, 2);

        let g = builder.finalize();
        assert_eq!(3, g.num_vertices);
        assert_eq!(vec![0, 1, 1, 2], g.endvertices);
        assert_eq!(vec![vec![StaticEdge::new(0)],
                        vec![StaticEdge::new_reverse(0), StaticEdge::new(1)],
                        vec![StaticEdge::new_reverse(1)]],
                   g.inc);
    }

    struct Test;

    impl GraphTests for Test {
        type G = StaticGraph;

        fn new() -> (Self::G, VecVertex<Self::G>, VecEdge<Self::G>) {
            Self::new_with_builder()
        }
    }

    graph_basic_tests!{Test}
    graph_prop_tests!{Test}
    graph_adj_tests!{Test}

    mod with_builder {
        use builder::BuilderTests;
        use static_::*;

        impl BuilderTests for StaticGraph {
            type G = Self;
        }

        graph_builder_tests!{StaticGraph}
    }
}
