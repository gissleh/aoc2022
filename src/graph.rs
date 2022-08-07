/// A basic node graph meant for small graphs for path traversal. It does not contain the data in
/// a map, and is most useful for traversal rather than lookup.
///
/// ```rust
/// use common::graph::Graph;
///
/// #[derive(Eq, PartialEq, Debug)]
/// struct Portal (bool, [u8; 2]);
///
/// let mut g: Graph<Portal, i32> = Graph::new();
/// let aa = g.insert(Portal(true, [b'A', b'A']));
/// let zz = g.insert(Portal(true, [b'Z', b'Z']));
/// g.connect_mutual(aa, zz, 64);
///
/// assert_eq!(aa, 0);
/// assert_eq!(zz, 1);
/// assert_eq!(g.node(aa), Some(&Portal(true, [b'A', b'A'])));
/// assert_eq!(g.node(zz), Some(&Portal(true, [b'Z', b'Z'])));
/// assert_eq!(g.edges(aa).unwrap().cloned().next(), Some((zz, 64)));
/// assert_eq!(g.edges(zz).unwrap().cloned().next(), Some((aa, 64)));
/// assert_eq!(g.edge(aa, zz).cloned(), Some(64))
/// ```
pub struct Graph<N, E> {
    nodes: Vec<Node<N, E>>,
}

impl<N, E> Graph<N, E> {
    pub fn find_by<F>(&self, pred: F) -> Option<usize> where F: Fn(&N) -> bool {
        self.nodes.iter().enumerate().find_map(|(i, n)| if pred(&n.data) {
            Some(i)
        } else {
            None
        })
    }

    pub fn insert(&mut self, value: N) -> usize {
        self.nodes.push(Node {
            edges: Vec::with_capacity(8),
            data: value,
        });

        self.nodes.len() - 1
    }

    pub fn connect(&mut self, a: usize, b: usize, e: E) {
        if a < self.nodes.len() && b < self.nodes.len() {
            self.nodes[a].edges.push((b, e))
        }
    }

    pub fn node(&self, index: usize) -> Option<&N> {
        self.nodes.get(index).map(|n| &n.data)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes(&self) -> impl Iterator<Item=(usize, &N)> {
        return self.nodes.iter().enumerate().map(|(i, n)| (i, &n.data))
    }

    pub fn edges(&self, index: usize) -> Option<impl Iterator<Item=&(usize, E)>> {
        self.nodes.get(index).map(|n| n.edges.iter())
    }

    pub fn edge(&self, from_index: usize, to_index: usize) -> Option<&E> {
        if from_index < self.nodes.len() && to_index < self.nodes.len() {
            self.nodes[from_index].edges.iter().find_map(|(i, e)| if *i == to_index {
                Some(e)
            } else {
                None
            })
        } else {
            None
        }
    }

    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(16),
        }
    }
}

impl<N, E> Graph<N, E> where N: Eq {
    pub fn find(&self, needle: &N) -> Option<usize> {
        self.nodes.iter().enumerate().find_map(|(i, n)| if n.data.eq(needle) {
            Some(i)
        } else {
            None
        })
    }

    pub fn find_or_insert(&mut self, value: N) -> usize {
        if let Some(index) = self.find(&value) {
            index
        } else {
            self.insert(value)
        }
    }
}


impl<N, E> Graph<N, E> where E: Copy {
    pub fn connect_mutual(&mut self, a: usize, b: usize, e: E) {
        if a < self.nodes.len() && b < self.nodes.len() {
            self.nodes[a].edges.push((b, e));
            self.nodes[b].edges.push((a, e));
        }
    }
}

struct Node<N, E> {
    data: N,
    edges: Vec<(usize, E)>,
}

