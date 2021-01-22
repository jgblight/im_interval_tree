#[cfg(test)]
use quickcheck::*;

use std::cmp::Ord;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Bound;
use std::ops::Bound::*;
use std::rc::Rc;

pub fn low_bound_cmp<T: Ord>(a: &Bound<T>, b: &Bound<T>) -> Ordering {
    match (a, b) {
        (Included(low1), Included(low2)) => low1.cmp(low2),
        (Included(low1), Excluded(low2)) => {
            if low1 <= low2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(low1), Included(low2)) => {
            if low1 < low2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(low1), Excluded(low2)) => low1.cmp(low2),
        (Unbounded, Unbounded) => Ordering::Equal,
        (Unbounded, _) => Ordering::Less,
        (_, Unbounded) => Ordering::Greater,
    }
}

pub fn low_bound_min<T: Ord + Clone>(a: &Rc<Bound<T>>, b: &Rc<Bound<T>>) -> Rc<Bound<T>> {
    match low_bound_cmp(&*a, &*b) {
        Ordering::Less => a.clone(),
        _ => b.clone(),
    }
}

pub fn high_bound_cmp<T: Ord + Clone>(a: &Bound<T>, b: &Bound<T>) -> Ordering {
    match (a, b) {
        (Included(high1), Included(high2)) => high1.cmp(high2),
        (Included(high1), Excluded(high2)) => {
            if high1 < high2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(high1), Included(high2)) => {
            if high1 <= high2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(high1), Excluded(high2)) => high1.cmp(high2),
        (Unbounded, Unbounded) => Ordering::Equal,
        (Unbounded, _) => Ordering::Greater,
        (_, Unbounded) => Ordering::Less,
    }
}

pub fn high_bound_max<T: Ord + Clone>(a: &Rc<Bound<T>>, b: &Rc<Bound<T>>) -> Rc<Bound<T>> {
    match high_bound_cmp(&*a, &*b) {
        Ordering::Less => b.clone(),
        _ => a.clone(),
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Interval<T: Ord + Clone> {
    pub low: Rc<Bound<T>>,
    pub high: Rc<Bound<T>>,
}

impl<T: Ord + Clone> Interval<T> {
    pub fn new(low: Bound<T>, high: Bound<T>) -> Interval<T> {
        Interval {
            low: Rc::new(low),
            high: Rc::new(high),
        }
    }

    fn valid(interval: &Interval<T>) -> bool {
        match (&*interval.low, &*interval.high) {
            (Included(low), Included(high)) => low <= high,

            (Included(low), Excluded(high))
            | (Excluded(low), Included(high))
            | (Excluded(low), Excluded(high)) => low < high,

            _ => true,
        }
    }

    pub fn get_overlap(&self, other: &Self) -> Option<Self> {
        let low = match low_bound_cmp(&*self.low, &*other.low) {
            Ordering::Less => other.low.clone(),
            _ => self.low.clone(),
        };
        let high = match high_bound_cmp(&*self.high, &*other.high) {
            Ordering::Less => self.high.clone(),
            _ => other.high.clone(),
        };
        let interval = Interval {
            low: low,
            high: high,
        };
        if Self::valid(&interval) {
            Some(interval)
        } else {
            None
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.get_overlap(other).is_some()
    }

    pub fn contains(&self, other: &Self) -> bool {
        let left_side_lte = match low_bound_cmp(self.low(), other.low()) {
            Ordering::Greater => false,
            _ => true,
        };
        let right_side_gte = match high_bound_cmp(self.high(), other.high()) {
            Ordering::Less => false,
            _ => true
        };
        left_side_lte && right_side_gte
    }

    pub fn low(&self) -> &Bound<T> {
        &*self.low
    }

    pub fn high(&self) -> &Bound<T> {
        &*self.high
    }
}

impl<T: Ord + Clone> PartialEq for Interval<T> {
    fn eq(&self, other: &Self) -> bool {
        self.low == other.low && self.high == other.high
    }
}

impl<T: Ord + Clone> Eq for Interval<T> {}

impl<T: Ord + Clone> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Clone> Ord for Interval<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let low_bound_cmp = low_bound_cmp(&*self.low, &*other.low);
        if low_bound_cmp == Ordering::Equal {
            high_bound_cmp(&*self.high, &*other.high)
        } else {
            low_bound_cmp
        }
    }
}

#[cfg(test)]
impl<T: Arbitrary + Clone + Ord + Debug> Arbitrary for Interval<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        loop {
            let a = Bound::<T>::arbitrary(g);
            let b = Bound::<T>::arbitrary(g);
            let interval = Interval::new(a.clone(), b.clone());
            if Interval::valid(&interval) {
                return interval;
            }

            let interval = Interval::new(b.clone(), a.clone());
            if Interval::valid(&interval) {
                return interval;
            }
        }
    }
}
