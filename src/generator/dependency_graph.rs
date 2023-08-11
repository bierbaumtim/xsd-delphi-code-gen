use core::hash::Hash;
use std::{
    cmp::{Eq, PartialEq},
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

// Dependency Graph for Types

// CustomNumber
// Alias1 -> Alias2 -> CustomNumber
// Alias3 -> CustomNumber
// Alias4 -> Alias1
// Alias5 -> Alias1
// => Tree
//                Alias4   Alias5
//                   \      /
//                    Alias1
//                  /       \
//      Alias3   Alias2   Alias6
//          \   /
//           CustomNumber
// => List
// CustomNumber, Alias3, Alias2, Alias1, Alias4, Alias5

pub(crate) struct DependencyGraph<K, T, F>
where
    K: Sized,
    K: Eq,
    K: PartialEq,
    K: Hash,
    K: Clone,
    K: Debug,
    K: Display,
    T: Clone,
    T: Debug,
    F: Fn(&T) -> (K, Option<K>),
{
    dependencies: HashMap<K, Node<K, T>>,
    keys_fn: F,
}

#[derive(Debug)]
struct Node<K, T> {
    item: T,
    parents: Vec<K>,
    children: Vec<K>,
}

impl<K, T> Node<K, T> {
    fn empty(item: T) -> Self {
        Node {
            item,
            parents: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl<K, T, F> DependencyGraph<K, T, F>
where
    K: Sized,
    K: Eq,
    K: PartialEq,
    K: Hash,
    K: Clone,
    K: Debug,
    K: Display,
    T: Clone,
    T: Debug,
    F: Fn(&T) -> (K, Option<K>),
{
    pub(crate) fn new(key: F) -> Self {
        DependencyGraph {
            dependencies: HashMap::new(),
            keys_fn: key,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        let (item_key, dep_key) = (self.keys_fn)(&item);

        let mut node = Node::empty(item);

        if let Some(dep_key) = dep_key {
            if let Some(dependency) = self.dependencies.get_mut(&dep_key) {
                dependency.children.push(item_key.clone());
            }

            node.parents.push(dep_key.clone());
        }

        for value in self.dependencies.values_mut() {
            if value.parents.contains(&item_key) {
                let (value_key, _) = (self.keys_fn)(&value.item);

                if !node.children.contains(&value_key) {
                    node.children.push(value_key);
                }
            }
        }

        self.dependencies.insert(item_key, node);
    }

    pub(crate) fn get_sorted_elements(&self) -> Vec<T> {
        let mut unique = HashSet::new();

        self.dependencies
            .values()
            .filter(|i| i.children.is_empty())
            .flat_map(|node| self.get_node_creation_order(node))
            .filter(|i| unique.insert((self.keys_fn)(i).0))
            .collect::<Vec<T>>()
    }

    fn get_node_creation_order(&self, node: &Node<K, T>) -> Vec<T> {
        node.parents
            .iter()
            .map(|i| self.dependencies.get(i))
            .map(|v| v.map(|i| self.get_node_creation_order(i)))
            .flat_map(|v| v.unwrap_or(Vec::new()))
            .chain(vec![node.item.clone()])
            .collect::<Vec<T>>()
    }
}

// TODO: Add test
