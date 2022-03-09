use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::IntoIterator;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Id(usize);

#[derive(Copy, Clone, PartialEq, Debug)]
struct Node {
    size: usize,
    parent: Id,
}

impl Node {
    fn new(parent: Id) -> Self {
        Self { size: 1, parent }
    }
}

#[derive(Debug)]
pub struct DisjointHashSet<T: Hash + Eq> {
    map: HashMap<T, Id>,
    data: Vec<Node>,
}

impl<T: Hash + Eq> DisjointHashSet<T> {
    /// Create an empty `DisjointHashSet`.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            data: Vec::new(),
        }
    }

    /// Create an empty `DisjointHashSet` with specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: HashMap::with_capacity(cap),
            data: Vec::with_capacity(cap),
        }
    }

    /// Create a `DisjointHashSet` with specified values in unique sets.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::with_values(vec!["this", "that", "other"]);
    /// assert!(set.find(&"this").is_some());
    /// assert!(set.find(&"that").is_some());
    /// assert!(set.find(&"other").is_some());
    /// ```
    pub fn with_values<S>(set: S) -> Self
    where
        S: IntoIterator<Item = T>,
    {
        let map: HashMap<T, Id> = set
            .into_iter()
            .enumerate()
            .map(|(a, b)| (b, Id(a)))
            .collect();
        let data = (0..map.len()).map(|i| Node::new(Id(i))).collect();
        Self { map, data }
    }

    /// Returns the number of elements in the disjoint set.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// assert_eq!(set.size(), 0);
    /// set.insert("this");
    /// assert_eq!(set.size(), 1);
    /// ```
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the disjoint set contains the specified value.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// set.insert("this");
    /// assert!(set.contains(&"this"));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    /// Returns `true` if the two values are in the same set.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::with_values(vec!["this", "that"]);
    /// assert!(!set.connected(&"this", &"that"));
    /// set.union("this", "that");
    /// assert!(set.connected(&"this", &"that"));
    /// ```
    pub fn connected(&mut self, value: &T, other: &T) -> bool {
        self.find(value) == self.find(other)
    }

    /// Insert a new value into the disjoint set.
    ///
    /// If the disjoint set already had this value present, returns `false`.
    /// If not returns `true`.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// assert!(!set.contains(&"this"));
    /// set.insert("this");
    /// assert!(set.contains(&"this"));
    /// ```
    pub fn insert(&mut self, value: T) -> bool {
        self.insert_inner(value) != Id(self.size() - 1)
    }

    fn insert_inner(&mut self, value: T) -> Id {
        self.map
            .entry(value)
            .or_insert_with(|| {
                let new_id = Id(self.data.len());
                self.data.push(Node::new(new_id));
                new_id
            })
            .clone()
    }

    /// Insert a connected set into the disjoint set.
    ///
    /// If a value is already present the set is unionized with it.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// set.insert_set(vec!["this", "that"]);
    /// assert!(set.connected(&"this", &"that"));
    /// ```
    pub fn insert_set<S>(&mut self, set: S)
    where
        S: IntoIterator<Item = T>,
    {
        let mut set = set.into_iter();
        let mut k = match set.next() {
            Some(k) => self.find_or_insert(k),
            None => return,
        };
        set.for_each(|v| {
            let v = self.find_or_insert(v);
            k = self.compress_path(k);
            self.union_inner(k, v)
        });
    }

    fn get(&self, id: Id) -> Node {
        self.data[id.0]
    }

    fn get_mut(&mut self, id: Id) -> &mut Node {
        &mut self.data[id.0]
    }

    /// Find the set a value is in.
    ///
    /// Returns `None` if the value is not present.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// assert!(set.find(&"this").is_none());
    /// set.insert("this");
    /// assert!(set.find(&"this").is_some());
    /// ```
    pub fn find(&mut self, value: &T) -> Option<Id> {
        let id = self.map.get(value)?.clone();
        Some(self.compress_path(id))
    }

    /// Find the set a value is in, inserting it if not present.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// let set_id = set.find_or_insert("this");
    /// assert_eq!(set.find_or_insert("this"), set_id);
    /// ```
    pub fn find_or_insert(&mut self, value: T) -> Id {
        let id = self.insert_inner(value);
        self.compress_path(id)
    }

    fn compress_path(&mut self, mut id: Id) -> Id {
        // path halving
        let mut parent = self.get(id).parent;
        while parent != id {
            self.get_mut(id).parent = self.get(parent).parent;
            id = parent;
            parent = self.get(id).parent;
        }
        id
    }

    /// Unions two sets together specified by values.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::with_values(vec!["this", "that"]);
    /// set.union(&"this", &"that");
    /// assert!(set.connected(&"this", &"that"));
    /// ```
    pub fn union(&mut self, value: T, other: T) {
        let value = self.find_or_insert(value);
        let other = self.find_or_insert(other);

        self.union_inner(value, other);
    }

    /// Union two sets together by their set id's.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// let id_1 = set.find_or_insert("this");
    /// let id_2 = set.find_or_insert("that");
    /// set.union_sets(id_1, id_2);
    /// assert!(set.connected(&"this", &"that"));
    /// ```
    pub fn union_sets(&mut self, value: Id, other: Id) {
        let value = self.compress_path(value);
        let other = self.compress_path(other);

        self.union_inner(value, other);
    }

    /// value and other are assumed to be the root
    fn union_inner(&mut self, value_id: Id, other_id: Id) {
        let value = self.get(value_id);
        let other = self.get(other_id);
        if value == other {
            return;
        }

        if value.size < other.size {
            self.get_mut(other_id).parent = value.parent;
            self.get_mut(value_id).size += other.size;
        } else {
            self.get_mut(value_id).parent = other.parent;
            self.get_mut(other_id).size += value.size;
        }
    }

    /// Split a value from it's set, creating it's own unique set.
    ///
    /// Inserts the value if not present.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// set.insert_set(vec!["this", "that"]);
    /// set.split(&"this");
    /// assert!(!set.connected(&"this", &"that"));
    /// ```
    pub fn split(&mut self, value: T) {
        let id = self.split_inner(value);
        self.get_mut(id).parent = id;
    }

    /// Split a value into the set of another.
    ///
    /// Inserts the value if not present.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// set.insert_set(vec!["this", "that"]);
    /// set.insert("other");
    /// set.split_into("this", "other");
    /// assert!(!set.connected(&"this", &"that"));
    /// assert!(set.connected(&"this", &"other"));
    /// ```
    pub fn split_into(&mut self, value: T, into: T) {
        let id = self.split_inner(value);
        let into = self.find_or_insert(into);
        self.union_inner(id, into);
    }

    /// Split a value into the set specified by it's id.
    ///
    /// Inserts the value if not present.
    ///
    /// # Examples
    /// ```
    /// # use disjoint_hash_set::DisjointHashSet;
    /// let mut set = DisjointHashSet::<&str>::new();
    /// set.insert_set(vec!["this", "that"]);
    /// let id = set.find_or_insert("other");
    /// set.split_into_set("this", id);
    /// assert!(!set.connected(&"this", &"that"));
    /// assert!(set.connected(&"this", &"other"));
    /// ```
    pub fn split_into_set(&mut self, value: T, into: Id) {
        let id = self.split_inner(value);
        let into = self.compress_path(into);
        self.union_inner(id, into);
    }

    fn split_inner(&mut self, value: T) -> Id {
        let id = self.insert_inner(value);
        let value = self.get(id);

        if value.size == 1 {
            return id;
        }
        let mut data_iter = self.data.iter_mut();
        let new_parent = {
            if id == value.parent {
                Id(data_iter.position(|v| v.parent == id).unwrap())
            } else {
                value.parent
            }
        };
        for mut v in data_iter.filter(|v| v.parent == id) {
            v.parent = new_parent;
        }
        self.get_mut(id).size = 1;
        id
    }
}
