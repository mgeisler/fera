use prelude::*;

use std::fmt::Debug;
use std::hash::Hash;

pub type Vertex<G> = <G as WithVertex>::Vertex;
pub type OptionVertex<G> = <G as WithVertex>::OptionVertex;
pub type VertexIndexProp<G> = <G as VertexIndex>::VertexIndexProp;
pub type VertexIter<'a, G> = <G as VertexTypes<'a, G>>::VertexIter;
pub type OutNeighborIter<'a, G> =
    <G as VertexTypes<'a, G>>::OutNeighborIter;
pub type DefaultVertexPropMut<G, T> =
    <G as WithVertexProp<T>>::VertexProp;
pub type VecVertex<G> = Vec<Vertex<G>>;

pub type Edge<G> = <G as WithEdge>::Edge;
pub type OptionEdge<G> = <G as WithEdge>::OptionEdge;
pub type EdgeIndexProp<G> = <G as EdgeIndex>::EdgeIndexProp;
pub type EdgeIter<'a, G> = <G as EdgeTypes<'a, G>>::EdgeIter;
pub type OutEdgeIter<'a, G> = <G as EdgeTypes<'a, G>>::OutEdgeIter;
pub type DefaultEdgePropMut<G, T> = <G as WithEdgeProp<T>>::EdgeProp;
pub type VecEdge<G> = Vec<Edge<G>>;

macro_rules! items {
    ($($item:item)*) => ($($item)*);
}

macro_rules! trait_alias {
    ($name:ident = $($base:tt)+) => {
        items! {
            pub trait $name: $($base)+ { }
            impl<T: $($base)+> $name for T { }
        }
    };
}

trait_alias!(Graph = VertexList + EdgeList<Kind = Undirected> + BasicProps);
trait_alias!(AdjacencyGraph = Graph + Adjacency);
trait_alias!(IncidenceGraph = AdjacencyGraph + Incidence);

trait_alias!(DiGraph = VertexList + EdgeList<Kind = Directed> + BasicProps);
trait_alias!(AdjacencyDiGraph = DiGraph + Adjacency);
trait_alias!(IncidenceDiGraph = AdjacencyDiGraph + Incidence);

trait_alias!(GraphItem = Copy + Eq + Hash + Debug);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Orientation {
    Directed,
    Undirected,
}

impl Orientation {
    fn is_directed(&self) -> bool {
        *self == Orientation::Directed
    }

    fn is_undirected(&self) -> bool {
        *self == Orientation::Undirected
    }
}

pub trait EdgeKind {}

pub trait UniformEdgeKind: EdgeKind {
    fn orientation() -> Orientation;

    fn is_directed() -> bool {
        Self::orientation().is_directed()
    }

    fn is_undirected() -> bool {
        Self::orientation().is_undirected()
    }
}


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Directed {}

impl EdgeKind for Directed {}

impl UniformEdgeKind for Directed {
    fn orientation() -> Orientation {
        Orientation::Directed
    }
}


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Undirected {}

impl EdgeKind for Undirected {}

impl UniformEdgeKind for Undirected {
    fn orientation() -> Orientation {
        Orientation::Undirected
    }
}


// TODO: write a graph with mixed edges and test it
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Mixed {}

impl EdgeKind for Mixed {}


pub trait VertexTypes<'a, G: WithVertex> {
    type VertexIter: Iterator<Item = Vertex<G>>;
    type OutNeighborIter: Iterator<Item = Vertex<G>>;
}

pub trait WithVertex: Sized + for<'a> VertexTypes<'a, Self> {
    type Vertex: 'static + GraphItem;
    type OptionVertex: 'static + GraphItem + Optional<Vertex<Self>> + From<Option<Vertex<Self>>>;

    // TODO: is this necessary?
    fn vertex_none() -> OptionVertex<Self> {
        Default::default()
    }

    // TODO: is this necessary?
    fn vertex_some(v: Vertex<Self>) -> OptionVertex<Self> {
        From::from(v)
    }
}

pub trait EdgeTypes<'a, G: WithEdge> {
    type EdgeIter: Iterator<Item = Edge<G>>;
    type OutEdgeIter: Iterator<Item = Edge<G>>;
}

pub trait WithEdge: Sized + WithVertex + for<'a> EdgeTypes<'a, Self> {
    type Kind: EdgeKind;
    type Edge: 'static + GraphItem;
    type OptionEdge: 'static + GraphItem + Optional<Edge<Self>> + From<Option<Edge<Self>>>;

    fn source(&self, e: Edge<Self>) -> Vertex<Self>;

    fn target(&self, e: Edge<Self>) -> Vertex<Self>;

    fn orientation(&self, _e: Edge<Self>) -> Orientation;

    fn is_directed_edge(&self, e: Edge<Self>) -> bool {
        self.orientation(e) == Orientation::Directed
    }

    fn is_undirected_edge(&self, e: Edge<Self>) -> bool {
        self.orientation(e) == Orientation::Undirected
    }

    fn ends(&self, e: Edge<Self>) -> (Vertex<Self>, Vertex<Self>) {
        (self.source(e), self.target(e))
    }

    fn opposite(&self, u: Vertex<Self>, e: Edge<Self>) -> Vertex<Self> {
        let (s, t) = self.ends(e);
        if u == s {
            t
        } else if u == t {
            s
        } else {
            panic!("u is not an end of e");
        }
    }

    fn reverse(&self, e: Edge<Self>) -> Edge<Self>
        where Self: WithEdge<Kind = Undirected>
    {
        self.get_reverse(e)
            .expect("the reverse of an edge (all undirected graphs must implement reverse)")
    }

    fn get_reverse(&self, _e: Edge<Self>) -> Option<Edge<Self>> {
        None
    }

    // TODO: is this necessary?
    fn edge_none() -> OptionEdge<Self> {
        Default::default()
    }

    // TODO: is this necessary?
    fn edge_some(e: Edge<Self>) -> OptionEdge<Self> {
        From::from(e)
    }
}

pub trait VertexList: Sized + WithVertex {
    fn vertices(&self) -> VertexIter<Self>;

    fn num_vertices(&self) -> usize {
        self.vertices().count()
    }
}

pub trait EdgeList: Sized + WithEdge {
    fn edges(&self) -> EdgeIter<Self>;

    fn num_edges(&self) -> usize {
        self.edges().count()
    }
}

pub trait Adjacency: WithVertex {
    fn out_neighbors(&self, v: Vertex<Self>) -> OutNeighborIter<Self>;

    fn out_degree(&self, v: Vertex<Self>) -> usize {
        self.out_neighbors(v).count()
    }
}

pub trait Incidence: WithEdge + Adjacency {
    fn out_edges(&self, v: Vertex<Self>) -> OutEdgeIter<Self>;
}

pub trait EdgeByEnds: WithEdge + WithVertex {
    // TODO: Move to EdgeList? What if there is more than one edge?
    // TODO: rename to get_edge?
    fn edge_by_ends(&self, u: Vertex<Self>, v: Vertex<Self>) -> Option<Edge<Self>>;
}


// Index

pub trait VertexIndex: WithVertex {
    type VertexIndexProp: VertexPropGet<Self, usize>;

    fn vertex_index(&self) -> VertexIndexProp<Self>;
}

pub trait EdgeIndex: WithEdge {
    type EdgeIndexProp: EdgePropGet<Self, usize>;

    fn edge_index(&self) -> EdgeIndexProp<Self>;
}
