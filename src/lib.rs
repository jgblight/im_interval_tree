/*!
 * An immutable data structure for storing and querying a collection of intervals
 *
 * ```
 * use std::ops::Bound::*;
 * use im_interval_tree::{IntervalTree, Interval};
 *
 * // Construct a tree of intervals
 * let tree : IntervalTree<u8> = IntervalTree::new();
 * let tree = tree.insert(Interval::new(Included(1), Excluded(3)));
 * let tree = tree.insert(Interval::new(Included(2), Excluded(4)));
 * let tree = tree.insert(Interval::new(Included(5), Unbounded));
 * let tree = tree.insert(Interval::new(Excluded(7), Included(8)));
 *
 * // Query for overlapping intervals
 * let query = tree.query_interval(&Interval::new(Included(3), Included(6)));
 * assert_eq!(
 *     query.collect::<Vec<Interval<u8>>>(),
 *     vec![
 *         Interval::new(Included(2), Excluded(4)),
 *         Interval::new(Included(5), Unbounded)
 *     ]
 * );
 *
 * // Query for a specific point
 * let query = tree.query_point(&2);
 * assert_eq!(
 *     query.collect::<Vec<Interval<u8>>>(),
 *     vec![
 *         Interval::new(Included(2), Excluded(4)),
 *         Interval::new(Included(1), Excluded(3))
 *     ]
 * );
 * ```
*/
#[cfg(test)]
mod test;

use std::cmp::*;
use std::ops::Bound;
use std::ops::Bound::*;

mod interval;
mod shared;

pub use crate::interval::Interval;
use crate::interval::*;
use crate::shared::Shared;

#[derive(Clone, Hash)]
struct Node<T: Ord + Clone> {
    interval: Interval<T>,
    left: Option<Shared<Node<T>>>,
    right: Option<Shared<Node<T>>>,
    height: usize,
    max: Shared<Bound<T>>,
    min: Shared<Bound<T>>,
}

impl<T: Ord + Clone> Node<T> {
    fn new(
        interval: Interval<T>,
        left: Option<Shared<Node<T>>>,
        right: Option<Shared<Node<T>>>,
    ) -> Node<T> {
        let height = usize::max(Self::height(&left), Self::height(&right)) + 1;
        let max = Self::get_max(&interval, &left, &right);
        let min = Self::get_min(&interval, &left, &right);
        Node {
            interval,
            left,
            right,
            height,
            max,
            min,
        }
    }

    fn leaf(interval: Interval<T>) -> Node<T> {
        Node::new(interval, None, None)
    }

    fn height(node: &Option<Shared<Node<T>>>) -> usize {
        match node {
            None => 0,
            Some(n) => n.height,
        }
    }

    fn get_max(
        interval: &Interval<T>,
        left: &Option<Shared<Node<T>>>,
        right: &Option<Shared<Node<T>>>,
    ) -> Shared<Bound<T>> {
        let mid = &interval.high;
        match (left, right) {
            (None, None) => mid.clone(),
            (None, Some(r)) => high_bound_max(mid, &r.max),
            (Some(l), None) => high_bound_max(mid, &l.max),
            (Some(l), Some(r)) => high_bound_max(mid, &high_bound_max(&l.max, &r.max)),
        }
    }

    fn get_min(
        interval: &Interval<T>,
        left: &Option<Shared<Node<T>>>,
        right: &Option<Shared<Node<T>>>,
    ) -> Shared<Bound<T>> {
        let mid = &interval.low;
        match (left, right) {
            (None, None) => mid.clone(),
            (None, Some(r)) => low_bound_min(mid, &r.min),
            (Some(l), None) => low_bound_min(mid, &l.min),
            (Some(l), Some(r)) => low_bound_min(mid, &low_bound_min(&l.min, &r.min)),
        }
    }

    fn balance_factor(&self) -> isize {
        (Self::height(&self.left) as isize) - (Self::height(&self.right) as isize)
    }

    fn insert(&self, interval: Interval<T>) -> Self {
        let res = match interval.cmp(&self.interval) {
            Ordering::Less => {
                let insert_left = match &self.left {
                    None => Node::leaf(interval),
                    Some(left_tree) => left_tree.insert(interval),
                };
                Node::new(
                    self.interval.clone(),
                    Some(Shared::new(insert_left)),
                    self.right.clone(),
                )
            }
            Ordering::Greater => {
                let insert_right = match &self.right {
                    None => Node::leaf(interval),
                    Some(right_tree) => right_tree.insert(interval),
                };
                Node::new(
                    self.interval.clone(),
                    self.left.clone(),
                    Some(Shared::new(insert_right)),
                )
            }
            Ordering::Equal => self.clone(),
        };
        res.balance()
    }

    fn get_minimum(&self) -> Interval<T> {
        match &self.left {
            None => self.interval.clone(),
            Some(left_tree) => left_tree.get_minimum(),
        }
    }

    fn remove(&self, interval: &Interval<T>) -> Option<Shared<Self>> {
        let res = match interval.cmp(&self.interval) {
            Ordering::Equal => match (&self.left, &self.right) {
                (None, None) => None,
                (Some(left_tree), None) => Some(left_tree.clone()),
                (None, Some(right_tree)) => Some(right_tree.clone()),
                (Some(_), Some(right_tree)) => {
                    let successor = right_tree.get_minimum();
                    let new_node = Node::new(
                        successor.clone(),
                        self.left.clone(),
                        right_tree.remove(&successor),
                    );
                    Some(Shared::new(new_node))
                }
            },
            Ordering::Less => match &self.left {
                None => Some(Shared::new(self.clone())),
                Some(left_tree) => Some(Shared::new(self.replace_left(left_tree.remove(interval)))),
            },
            Ordering::Greater => match &self.right {
                None => Some(Shared::new(self.clone())),
                Some(right_tree) => {
                    Some(Shared::new(self.replace_right(right_tree.remove(interval))))
                }
            },
        };
        res.map(|r| Shared::new(r.balance()))
    }

    fn replace_left(&self, new_left: Option<Shared<Node<T>>>) -> Node<T> {
        Self::new(self.interval.clone(), new_left, self.right.clone())
    }

    fn replace_right(&self, new_right: Option<Shared<Node<T>>>) -> Node<T> {
        Self::new(self.interval.clone(), self.left.clone(), new_right)
    }

    fn rotate_right(&self) -> Self {
        let pivot = self.left.as_ref().unwrap();
        let new_right = self.replace_left(pivot.right.clone());
        pivot.replace_right(Some(Shared::new(new_right)))
    }

    fn rotate_left(&self) -> Self {
        let pivot = self.right.as_ref().unwrap();
        let new_left = self.replace_right(pivot.left.clone());
        pivot.replace_left(Some(Shared::new(new_left)))
    }

    fn balance(&self) -> Self {
        let balance_factor = self.balance_factor();
        if balance_factor < -1 {
            let right = self.right.as_ref().unwrap();
            if right.balance_factor() > 0 {
                self.replace_right(Some(Shared::new(right.rotate_right())))
                    .rotate_left()
            } else {
                self.rotate_left()
            }
        } else if balance_factor > 1 {
            let left = self.left.as_ref().unwrap();
            if left.balance_factor() < 0 {
                self.replace_left(Some(Shared::new(left.rotate_left())))
                    .rotate_right()
            } else {
                self.rotate_right()
            }
        } else {
            self.clone()
        }
    }
}

/// An Iterator over Intervals matching some query
pub struct Iter<T: Ord + Clone> {
    stack: Vec<Shared<Node<T>>>,
    query: Interval<T>,
}

impl<T: Ord + Clone> Iterator for Iter<T> {
    type Item = Interval<T>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if let Some(left_tree) = &node.left {
                let max_is_gte = match (&*left_tree.max, self.query.low()) {
                    (Included(max), Included(low)) => max >= low,
                    (Included(max), Excluded(low))
                    | (Excluded(max), Included(low))
                    | (Excluded(max), Excluded(low)) => max > low,
                    _ => true,
                };
                if max_is_gte {
                    self.stack.push(left_tree.clone())
                }
            }
            if let Some(right_tree) = &node.right {
                let min_is_lte = match (&*right_tree.min, self.query.high()) {
                    (Included(min), Included(high)) => min <= high,
                    (Included(min), Excluded(high))
                    | (Excluded(min), Included(high))
                    | (Excluded(min), Excluded(high)) => min < high,
                    _ => true,
                };
                if min_is_lte {
                    self.stack.push(right_tree.clone())
                }
            }
            if self.query.overlaps(&node.interval) {
                return Some(node.interval.clone());
            }
        }
        None
    }
}

/// An immutable data structure for storing and querying a collection of intervals
///
/// # Example
/// ```
/// use std::ops::Bound::*;
/// use im_interval_tree::{IntervalTree, Interval};
///
/// // Construct a tree of intervals
/// let tree : IntervalTree<u8> = IntervalTree::new();
/// let tree = tree.insert(Interval::new(Included(1), Excluded(3)));
/// let tree = tree.insert(Interval::new(Included(2), Excluded(4)));
/// let tree = tree.insert(Interval::new(Included(5), Unbounded));
/// let tree = tree.insert(Interval::new(Excluded(7), Included(8)));
///
/// // Query for overlapping intervals
/// let query = tree.query_interval(&Interval::new(Included(3), Included(6)));
/// assert_eq!(
///     query.collect::<Vec<Interval<u8>>>(),
///     vec![
///         Interval::new(Included(2), Excluded(4)),
///         Interval::new(Included(5), Unbounded)
///     ]
/// );
///
/// // Query for a specific point
/// let query = tree.query_point(&2);
/// assert_eq!(
///     query.collect::<Vec<Interval<u8>>>(),
///     vec![
///         Interval::new(Included(2), Excluded(4)),
///         Interval::new(Included(1), Excluded(3))
///     ]
/// );
/// ```
#[derive(Clone, Hash)]
pub struct IntervalTree<T: Ord + Clone> {
    root: Option<Shared<Node<T>>>,
}

impl<T: Ord + Clone> IntervalTree<T> {
    /// Construct an empty IntervalTree
    pub fn new() -> IntervalTree<T> {
        IntervalTree { root: None }
    }

    /// Construct a new IntervalTree with the given Interval added
    ///
    /// # Example
    /// ```
    /// # use std::ops::Bound::*;
    /// # use im_interval_tree::{IntervalTree, Interval};
    /// let tree : IntervalTree<u8> = IntervalTree::new();
    /// let tree = tree.insert(Interval::new(Included(1), Included(2)));
    /// assert_eq!(
    ///     tree.iter().collect::<Vec<Interval<u8>>>(),
    ///     vec![Interval::new(Included(1), Included(2))]
    /// );
    /// ```
    pub fn insert(&self, interval: Interval<T>) -> IntervalTree<T> {
        let new_root = match &self.root {
            None => Node::leaf(interval),
            Some(node) => node.insert(interval),
        };
        IntervalTree {
            root: Some(Shared::new(new_root)),
        }
    }

    /// Construct a new IntervalTree minus the given Interval, if present
    ///
    /// # Example
    /// ```
    /// # use std::ops::Bound::*;
    /// # use im_interval_tree::{IntervalTree, Interval};
    /// let tree : IntervalTree<u8> = IntervalTree::new();
    /// let tree = tree.insert(Interval::new(Included(1), Included(2)));
    /// let tree = tree.insert(Interval::new(Included(1), Included(3)));
    ///
    /// let tree = tree.remove(&Interval::new(Included(1), Included(2)));
    /// assert_eq!(
    ///     tree.iter().collect::<Vec<Interval<u8>>>(),
    ///     vec![Interval::new(Included(1), Included(3))]
    /// );
    /// ```
    pub fn remove(&self, interval: &Interval<T>) -> IntervalTree<T> {
        match &self.root {
            None => IntervalTree::new(),
            Some(node) => IntervalTree {
                root: node.remove(interval),
            },
        }
    }

    /// Return an Iterator over all the intervals in the tree that overlap
    /// with the given interval
    ///
    /// # Example
    /// ```
    /// # use std::ops::Bound::*;
    /// # use im_interval_tree::{IntervalTree, Interval};
    /// let tree : IntervalTree<u8> = IntervalTree::new();
    /// let tree = tree.insert(Interval::new(Included(1), Excluded(3)));
    /// let tree = tree.insert(Interval::new(Included(5), Unbounded));
    ///
    /// let query = tree.query_interval(&Interval::new(Included(3), Included(6)));
    /// assert_eq!(
    ///     query.collect::<Vec<Interval<u8>>>(),
    ///     vec![Interval::new(Included(5), Unbounded)]
    /// );
    /// ```
    pub fn query_interval(&self, interval: &Interval<T>) -> impl Iterator<Item = Interval<T>> + '_ {
        let mut stack = Vec::new();
        if let Some(node) = &self.root {
            stack.push(node.clone())
        }
        Iter {
            stack,
            query: interval.clone(),
        }
    }

    /// Return an Iterator over all the intervals in the tree that contain
    /// the given point
    ///
    /// This is equivalent to `tree.query_interval(Interval::new(Included(point), Included(point)))`
    ///
    /// # Example
    /// ```
    /// # use std::ops::Bound::*;
    /// # use im_interval_tree::{IntervalTree, Interval};
    /// let tree : IntervalTree<u8> = IntervalTree::new();
    /// let tree = tree.insert(Interval::new(Included(1), Excluded(3)));
    /// let tree = tree.insert(Interval::new(Included(5), Unbounded));
    ///
    /// let query = tree.query_point(&2);
    /// assert_eq!(
    ///     query.collect::<Vec<Interval<u8>>>(),
    ///     vec![Interval::new(Included(1), Excluded(3))]
    /// );
    /// ```
    pub fn query_point(&self, point: &T) -> impl Iterator<Item = Interval<T>> + '_ {
        let interval = Interval::new(Included(point.clone()), Included(point.clone()));
        self.query_interval(&interval)
    }

    /// Return an Iterator over all the intervals in the tree
    ///
    /// This is equivalent to `tree.query_interval(Unbounded, Unbounded)`
    ///
    /// # Example
    /// ```
    /// # use std::ops::Bound::*;
    /// # use im_interval_tree::{IntervalTree, Interval};
    /// let tree : IntervalTree<u8> = IntervalTree::new();
    /// let tree = tree.insert(Interval::new(Included(2), Excluded(4)));
    /// let tree = tree.insert(Interval::new(Included(5), Unbounded));
    ///
    /// let iter = tree.iter();
    /// assert_eq!(
    ///     iter.collect::<Vec<Interval<u8>>>(),
    ///     vec![
    ///         Interval::new(Included(2), Excluded(4)),
    ///         Interval::new(Included(5), Unbounded),
    ///     ]
    /// );
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = Interval<T>> + '_ {
        self.query_interval(&Interval::new(Unbounded, Unbounded))
    }
}

impl<T: Ord + Clone> Default for IntervalTree<T> {
    fn default() -> Self {
        Self::new()
    }
}
