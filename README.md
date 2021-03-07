# im_interval_tree
An immutable data structure for storing and querying a collection of intervals

```rust
use std::ops::Bound::*;
use im_interval_tree::{IntervalTree, Interval};

// Construct a tree of intervals
let tree : IntervalTree<u8> = IntervalTree::new();
let tree = tree.insert(Interval::new(Included(1), Excluded(3)));
let tree = tree.insert(Interval::new(Included(2), Excluded(4)));
let tree = tree.insert(Interval::new(Included(5), Unbounded));
let tree = tree.insert(Interval::new(Excluded(7), Included(8)));

// Query for overlapping intervals
let query = tree.query_interval(&Interval::new(Included(3), Included(6)));
assert_eq!(
    query.collect::<Vec<Interval<u8>>>(),
    vec![
        Interval::new(Included(2), Excluded(4)),
        Interval::new(Included(5), Unbounded)
    ]
);

// Query for a specific point
let query = tree.query_point(&2);
assert_eq!(
    query.collect::<Vec<Interval<u8>>>(),
    vec![
        Interval::new(Included(2), Excluded(4)),
        Interval::new(Included(1), Excluded(3))
    ]
);
```