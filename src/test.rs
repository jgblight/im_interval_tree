use std::collections::HashSet;

use crate::*;
use quickcheck::*;

quickcheck! {
    fn test_insert(intervals : HashSet<Interval<u8>>) -> TestResult {
        let mut tree = IntervalTree::new();
        for i in &intervals {
            tree = tree.insert(i.clone());
        }

        let mut collected = HashSet::new();
        for i in tree.iter() {
            collected.insert((*i).clone());
        }
        TestResult::from_bool(collected == intervals)
    }
}

quickcheck! {
    fn test_remove(intervals : Vec<Interval<u8>>, to_remove : usize) -> TestResult {
        if intervals.len() == 0 {
            return TestResult::discard();
        }
        let interval_to_remove = intervals.get(to_remove % intervals.len()).unwrap();
        let mut tree = IntervalTree::new();
        let mut expected = HashSet::new();
        for i in &intervals {
            tree = tree.insert(i.clone());
            if &i != &interval_to_remove {
                expected.insert(i.clone());
            }
        }

        tree = tree.remove(interval_to_remove);

        let mut collected = HashSet::new();
        for i in tree.iter() {
            collected.insert((*i).clone());
        }
        TestResult::from_bool(collected == expected)
    }
}

quickcheck! {
    fn test_query_interval(intervals : HashSet<Interval<u8>>, query : Interval<u8>) -> TestResult {
        let mut tree = IntervalTree::new();
        let mut expected = HashSet::new();
        for i in &intervals {
            tree = tree.insert(i.clone());
            if i.overlaps(&query) {
                expected.insert(i.clone());
            }
        }

        let mut collected = HashSet::new();
        for i in tree.query_interval(&query) {
            collected.insert((*i).clone());
        }
        TestResult::from_bool(collected == expected)
    }
}

quickcheck! {
    fn test_query_point(intervals : HashSet<Interval<u8>>, query : u8) -> TestResult {
        let mut tree = IntervalTree::new();
        let mut expected = HashSet::new();
        for i in &intervals {
            tree = tree.insert(i.clone());

            let point_gte_low = match &*i.low {
                Included(low) => &query >= low,
                Excluded(low) => &query > low,
                Unbounded => true
            };

            let point_lte_high = match &*i.high {
                Included(high) => &query <= high,
                Excluded(high) => &query < high,
                Unbounded => true
            };

            if point_gte_low && point_lte_high {
                expected.insert(i.clone());
            }
        }

        let mut collected = HashSet::new();
        for i in tree.query_point(&query) {
            collected.insert((*i).clone());
        }
        TestResult::from_bool(collected == expected)
    }
}

quickcheck! {
    fn test_get_overlap(a: Interval<u8>, b: Interval<u8>) -> TestResult {
        let overlap = a.get_overlap(&b);
        let get_overlap_is_commutative = &overlap == &b.get_overlap(&a);

        let get_overlap_is_minimal = match &overlap {
            Some(ov) => &a.get_overlap(&ov) == &overlap && &b.get_overlap(&ov) == &overlap,
            None => true
        };
        TestResult::from_bool(get_overlap_is_commutative && get_overlap_is_minimal)
    }
}
