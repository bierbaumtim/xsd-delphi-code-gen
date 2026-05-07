use core::hash::Hash;
use std::{
    cmp::{Eq, PartialEq},
    collections::{HashMap, HashSet},
    fmt::Debug,
};

pub trait Dependable<K>
where
    K: Eq + PartialEq + Hash,
{
    fn key(&self) -> &K;
    fn key_and_deps(&self) -> (&K, Option<Vec<K>>);
}

/// Dependency Graph for Types
///
/// # Example
///
/// ```text
/// CustomNumber
/// Alias1 -> Alias2 -> CustomNumber
/// Alias3 -> CustomNumber
/// Alias4 -> Alias1
/// Alias5 -> Alias1
/// => Tree
///            Alias4   Alias5
///               \      /
///                Alias1
///              /       \
///  Alias3   Alias2   Alias6
///      \   /
///       CustomNumber
/// => List
/// CustomNumber, Alias3, Alias2, Alias1, Alias4, Alias5
/// ```
pub struct DependencyGraph<K, T>
where
    K: Eq + PartialEq + Hash + Clone,
    T: Clone + Dependable<K>,
{
    dependencies: HashMap<K, Node<K, T>>,
}

#[derive(Debug)]
struct Node<K, T> {
    item: T,
    parents: Vec<K>,
    children: Vec<K>,
}

impl<K, T> Node<K, T> {
    fn empty(item: T) -> Self {
        Self {
            item,
            parents: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl<K, T> DependencyGraph<K, T>
where
    // K: Sized,
    K: Eq + PartialEq + Hash + Clone,
    T: Clone + Dependable<K>,
{
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Adds an item to the dependency graph
    pub fn push(&mut self, item: T) {
        let mut node = Node::empty(item);

        let (item_key, dep_keys) = node.item.key_and_deps();

        if let Some(dep_keys) = dep_keys {
            for dep_key in dep_keys {
                if let Some(dependency) = self.dependencies.get_mut(&dep_key) {
                    dependency.children.push(item_key.clone());
                }

                node.parents.push(dep_key.clone());
            }
        }

        for value in self.dependencies.values() {
            if value.parents.contains(item_key) {
                let value_key = value.item.key();

                if !node.children.contains(value_key) {
                    node.children.push(value_key.clone());
                }
            }
        }

        self.dependencies.insert(item_key.clone(), node);
    }

    /// Returns a list of the items of the dependency graph sorted by their dependencies
    pub fn get_sorted_elements(&self) -> Vec<T> {
        let mut unique = HashSet::new();

        self.dependencies
            .values()
            .filter(|i| i.children.is_empty())
            .flat_map(|node| self.get_node_creation_order(node))
            .filter(|i| unique.insert(i.key().clone()))
            .collect::<Vec<T>>()
    }

    fn get_node_creation_order(&self, node: &Node<K, T>) -> Vec<T> {
        node.parents
            .iter()
            .map(|i| self.dependencies.get(i))
            .filter_map(|v| v.map(|i| self.get_node_creation_order(i)))
            .flatten()
            .chain(vec![node.item.clone()])
            .collect::<Vec<T>>()
    }
}

impl<K, T> Default for DependencyGraph<K, T>
where
    // K: Sized,
    K: Eq + PartialEq + Hash + Clone,
    T: Clone + Dependable<K>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct GraphItem {
        key: String,
        dep: Option<String>,
    }

    impl Dependable<String> for GraphItem {
        fn key(&self) -> &String {
            &self.key
        }

        fn key_and_deps(&self) -> (&String, Option<Vec<String>>) {
            (&self.key, self.dep.clone().map(|d| vec![d]))
        }
    }

    #[test]
    fn get_sorted_elements_with_empty_graph() {
        let graph = DependencyGraph::<String, GraphItem>::new();

        let items = graph.get_sorted_elements();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn get_sorted_elements_with_duplicates() {
        let mut graph = DependencyGraph::<String, GraphItem>::new();

        graph.push(GraphItem {
            key: "Alias3".to_owned(),
            dep: Some("CustomNumber".to_owned()),
        });
        graph.push(GraphItem {
            key: "Alias4".to_owned(),
            dep: Some("Alias1".to_owned()),
        });
        graph.push(GraphItem {
            key: "Alias2".to_owned(),
            dep: Some("CustomNumber".to_owned()),
        });
        graph.push(GraphItem {
            key: "Alias6".to_owned(),
            dep: None,
        });
        graph.push(GraphItem {
            key: "Alias1".to_owned(),
            dep: Some("Alias2".to_owned()),
        });
        graph.push(GraphItem {
            key: "Alias5".to_owned(),
            dep: Some("Alias1".to_owned()),
        });
        graph.push(GraphItem {
            key: "CustomNumber".to_owned(),
            dep: None,
        });

        let items = graph.get_sorted_elements();

        let cni = items.iter().position(|i| i.key == "CustomNumber").unwrap();
        let a3i = items.iter().position(|i| i.key == "Alias3").unwrap();
        let a2i = items.iter().position(|i| i.key == "Alias2").unwrap();
        let a1i = items.iter().position(|i| i.key == "Alias1").unwrap();
        let a4i = items.iter().position(|i| i.key == "Alias4").unwrap();
        let a5i = items.iter().position(|i| i.key == "Alias5").unwrap();

        assert!(cni < a3i);
        assert!(cni < a2i);
        assert!(a2i < a1i);
        assert!(a1i < a4i);
        assert!(a1i < a5i);
    }
}
