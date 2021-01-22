#[cfg(test)]
mod test;

use std::cmp::*;
use std::ops::Bound;
use std::ops::Bound::*;
use std::rc::Rc;

mod interval;
pub use crate::interval::Interval;
use crate::interval::*;

#[derive(Clone, Hash)]
struct Node<T: Ord + Clone> {
    interval: Rc<Interval<T>>,
    left: Option<Rc<Node<T>>>,
    right: Option<Rc<Node<T>>>,
    height: usize,
    max: Rc<Bound<T>>,
    min: Rc<Bound<T>>,
}

impl<T: Ord + Clone> Node<T> {
    fn new(
        interval: Rc<Interval<T>>,
        left: Option<Rc<Node<T>>>,
        right: Option<Rc<Node<T>>>,
    ) -> Node<T> {
        let height = usize::max(Self::height(&left), Self::height(&right)) + 1;
        let max = Self::get_max(&interval, &left, &right);
        let min = Self::get_min(&interval, &left, &right);
        Node {
            interval: interval,
            left: left,
            right: right,
            height: height,
            max: max,
            min: min,
        }
    }

    fn leaf(interval: Rc<Interval<T>>) -> Node<T> {
        Node::new(interval, None, None)
    }

    fn height(node: &Option<Rc<Node<T>>>) -> usize {
        match node {
            None => 0,
            Some(n) => n.height,
        }
    }

    fn get_max(
        interval: &Rc<Interval<T>>,
        left: &Option<Rc<Node<T>>>,
        right: &Option<Rc<Node<T>>>,
    ) -> Rc<Bound<T>> {
        let mid = &interval.high;
        match (left, right) {
            (None, None) => mid.clone(),
            (None, Some(r)) => high_bound_max(mid, &r.max),
            (Some(l), None) => high_bound_max(mid, &l.max),
            (Some(l), Some(r)) => high_bound_max(mid, &high_bound_max(&l.max, &r.max)),
        }
    }

    fn get_min(
        interval: &Rc<Interval<T>>,
        left: &Option<Rc<Node<T>>>,
        right: &Option<Rc<Node<T>>>,
    ) -> Rc<Bound<T>> {
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
        let res = if interval < *self.interval {
            let insert_left = match &self.left {
                None => Node::leaf(Rc::new(interval)),
                Some(left_tree) => left_tree.insert(interval),
            };
            Node::new(
                self.interval.clone(),
                Some(Rc::new(insert_left)),
                self.right.clone(),
            )
        } else if interval > *self.interval {
            let insert_right = match &self.right {
                None => Node::new(Rc::new(interval), None, None),
                Some(right_tree) => right_tree.insert(interval),
            };
            Node::new(
                self.interval.clone(),
                self.left.clone(),
                Some(Rc::new(insert_right)),
            )
        } else {
            self.clone()
        };
        res.balance()
    }

    fn get_minimum(&self) -> Rc<Interval<T>> {
        match &self.left {
            None => self.interval.clone(),
            Some(left_tree) => left_tree.get_minimum(),
        }
    }

    fn remove(&self, interval: &Interval<T>) -> Option<Rc<Self>> {
        let res = if interval == &*self.interval {
            match (&self.left, &self.right) {
                (None, None) => None,
                (Some(left_tree), None) => Some(left_tree.clone()),
                (None, Some(right_tree)) => Some(right_tree.clone()),
                (Some(_), Some(right_tree)) => {
                    let successor = right_tree.get_minimum();
                    let new_node = Node::new(
                        successor.clone(),
                        self.left.clone(),
                        right_tree.remove(&*successor),
                    );
                    Some(Rc::new(new_node))
                }
            }
        } else if interval < &self.interval {
            match &self.left {
                None => Some(Rc::new(self.clone())),
                Some(left_tree) => Some(Rc::new(self.replace_left(left_tree.remove(interval)))),
            }
        } else {
            match &self.right {
                None => Some(Rc::new(self.clone())),
                Some(right_tree) => Some(Rc::new(self.replace_right(right_tree.remove(interval)))),
            }
        };
        match res {
            None => None,
            Some(r) => Some(Rc::new(r.balance())),
        }
    }

    fn replace_left(&self, new_left: Option<Rc<Node<T>>>) -> Node<T> {
        Self::new(self.interval.clone(), new_left, self.right.clone())
    }

    fn replace_right(&self, new_right: Option<Rc<Node<T>>>) -> Node<T> {
        Self::new(self.interval.clone(), self.left.clone(), new_right)
    }

    fn rotate_right(&self) -> Self {
        let pivot = self.left.as_ref().unwrap();
        let new_right = self.replace_left(pivot.right.clone());
        pivot.replace_right(Some(Rc::new(new_right)))
    }

    fn rotate_left(&self) -> Self {
        let pivot = self.right.as_ref().unwrap();
        let new_left = self.replace_right(pivot.left.clone());
        pivot.replace_left(Some(Rc::new(new_left)))
    }

    fn balance(&self) -> Self {
        let balance_factor = self.balance_factor();
        if balance_factor < -1 {
            let right = self.right.as_ref().unwrap();
            if right.balance_factor() > 0 {
                self.replace_right(Some(Rc::new(right.rotate_right())))
                    .rotate_left()
            } else {
                self.rotate_left()
            }
        } else if balance_factor > 1 {
            let left = self.left.as_ref().unwrap();
            if left.balance_factor() < 0 {
                self.replace_left(Some(Rc::new(left.rotate_left())))
                    .rotate_right()
            } else {
                self.rotate_right()
            }
        } else {
            self.clone()
        }
    }
}

pub struct Iter<T: Ord + Clone> {
    stack: Vec<Rc<Node<T>>>,
    query: Interval<T>,
}

impl<T: Ord + Clone> Iterator for Iter<T> {
    type Item = Rc<Interval<T>>;
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
            if self.query.overlaps(&*node.interval) {
                return Some(node.interval.clone());
            }
        }
        None
    }
}

#[derive(Clone, Hash)]
pub struct IntervalTree<T: Ord + Clone> {
    root: Option<Rc<Node<T>>>,
}

impl<T: Ord + Clone> IntervalTree<T> {
    pub fn new() -> IntervalTree<T> {
        IntervalTree { root: None }
    }

    pub fn insert(&self, interval: Interval<T>) -> IntervalTree<T> {
        let new_root = match &self.root {
            None => Node::leaf(Rc::new(interval)),
            Some(node) => node.insert(interval),
        };
        IntervalTree {
            root: Some(Rc::new(new_root)),
        }
    }

    pub fn remove(&self, interval: &Interval<T>) -> IntervalTree<T> {
        match &self.root {
            None => IntervalTree::new(),
            Some(node) => IntervalTree {
                root: node.remove(interval),
            },
        }
    }

    pub fn query_interval(
        &self,
        interval: &Interval<T>,
    ) -> impl Iterator<Item = Rc<Interval<T>>> + '_ {
        let mut stack = Vec::new();
        if let Some(node) = &self.root {
            stack.push(node.clone())
        }
        Iter {
            stack: stack,
            query: interval.clone(),
        }
    }

    pub fn query_point(&self, point: &T) -> impl Iterator<Item = Rc<Interval<T>>> + '_ {
        let interval = Interval::new(Included(point.clone()), Included(point.clone()));
        self.query_interval(&interval)
    }

    pub fn iter(&self) -> impl Iterator<Item = Rc<Interval<T>>> + '_ {
        self.query_interval(&Interval::new(Unbounded, Unbounded))
    }
}
