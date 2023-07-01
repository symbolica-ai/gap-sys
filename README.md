# gap-sys

This crate contains bindings to GAP - Groups, Algorithms, Programming - a System for Computational Discrete Algebra. It is currently in a very early state, and contribution is encouraged.

#### Example showing how to create a Group
```
let mut gap = Gap::init();
let gap_element = gap.eval("Group((1,2,3),(1,2));").unwrap();
assert_eq!(gap.elem_string(&gap_element), "Group( [ (1,2,3), (1,2) ] )");
```

#### Example showing how to access elements of a list
```
let mut gap = Gap::init();
let outer_list = gap.eval("[[1, 2, 3], [4, 5, 6]];;").unwrap();
let inner_list = gap.get_list_elem(&outer_list, 1).unwrap();
let element = gap.get_list_elem(&inner_list, 1).unwrap();
let string = gap.elem_string(&element);
assert_eq!(string, "5");
```