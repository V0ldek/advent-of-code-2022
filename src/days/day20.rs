use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::combinator::all_consuming;
use std::{
    cell::{Ref, RefCell, RefMut},
    cmp,
    fmt::{self, Debug, Display},
    rc::Rc,
};

#[derive(Default)]
pub struct Day20 {}

const MAGIC_CONSTANT: Int = 811_589_153;
const MAGIC_INDICES: [usize; 3] = [1000, 2000, 3000];
const REPETITIONS: usize = 10;

impl Solution for Day20 {
    type Part1Result = Int;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Int>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(integer))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        mix(data, 1, 1)
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        mix(data, MAGIC_CONSTANT, REPETITIONS)
    }
}

fn mix(numbers: &[Int], multiplier: Int, times: usize) -> Int {
    let modulo = numbers.len() as Int - 1;
    let zero_initial_idx = numbers.iter().position(|&x| x == 0).expect("no 0 in input");
    let mut tree = None;
    let mut node_map = Vec::with_capacity(numbers.len());
    for (i, x) in numbers.iter().map(|x| x * multiplier).enumerate() {
        let node = insert(&mut tree, x, i);
        node_map.push(node);
    }

    for _ in 0..times {
        for (i, x) in numbers.iter().map(|x| x * multiplier).enumerate() {
            let idx = node_map[i].index();
            remove(&mut tree, idx);

            let base = idx as Int + x;
            let target_idx = ((base % modulo) + modulo) % modulo;
            node_map[i] = insert(&mut tree, x, usize::try_from(target_idx).unwrap());
        }
    }

    let zero_idx = node_map[zero_initial_idx].index();
    let final_list: Vec<_> = tree.unwrap().collect();

    let mut result = 0;

    for idx in MAGIC_INDICES {
        let actual_idx = (zero_idx + idx) % numbers.len();
        result += final_list[actual_idx];
    }

    result
}

fn insert<T>(root: &mut Option<NodeRef<T>>, element: T, idx: usize) -> NodeRef<T> {
    match root.as_mut() {
        Some(node) => {
            let (new_node, new_root) = node.insert(element, idx);

            if new_root.is_some() {
                *root = new_root;
            }

            new_node
        }
        None => {
            let node = NodeRef::new(element, None);
            *root = Some(node.clone());
            node
        }
    }
}

fn remove<T: Debug>(root: &mut Option<NodeRef<T>>, idx: usize) {
    let root_node = root.as_mut().expect("cannot remove from empty tree");
    let new_root = root_node.remove(idx);
    *root = new_root;
}

type Int = i64;

struct NodeRef<T>(Rc<RefCell<Node<T>>>);

impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<T> Eq for NodeRef<T> {}

struct Node<T> {
    element: T,
    size: usize,
    height: usize,
    left: Option<NodeRef<T>>,
    right: Option<NodeRef<T>>,
    parent: Option<NodeRef<T>>,
}

impl<T> NodeRef<T> {
    fn new(element: T, parent: Option<NodeRef<T>>) -> Self {
        NodeRef(Rc::new(RefCell::new(Node {
            element,
            size: 1,
            height: 1,
            left: None,
            right: None,
            parent,
        })))
    }

    fn insert(&mut self, element: T, idx: usize) -> (NodeRef<T>, Option<NodeRef<T>>) {
        let mut me = self.0.borrow_mut();

        if idx > me.size {
            panic!("cannot insert at index {idx} to tree of size {}", me.size);
        }

        let left_size = me.left.as_ref().map_or(0, |n| n.size());
        let (insert_at_node, insert_at_idx) = if left_size >= idx {
            (&mut me.left, idx)
        } else {
            (&mut me.right, idx - left_size - 1)
        };

        let new_node = match insert_at_node {
            Some(node) => {
                let (new_node, new_child) = node.insert(element, insert_at_idx);
                if new_child.is_some() {
                    *insert_at_node = new_child;
                }
                new_node
            }
            None => {
                let node = NodeRef::new(element, Some(self.clone()));
                *insert_at_node = Some(node.clone());
                node
            }
        };

        drop(me);

        self.update_props();

        if self.balance_factor().abs() >= 2 {
            let rebalance_result = self.rebalance();
            (new_node, Some(rebalance_result))
        } else {
            (new_node, None)
        }
    }

    fn remove(&mut self, idx: usize) -> Option<NodeRef<T>> {
        let mut me = self.0.borrow_mut();

        if idx > me.size {
            panic!("cannot remove at index {idx} from tree of size {}", me.size);
        }

        let left_size = me.left.as_ref().map_or(0, |n| n.size());
        if left_size == idx {
            drop(me);
            return self.remove_node();
        }

        let (remove_at_ref, remove_at_idx) = if left_size > idx {
            (&mut me.left, idx)
        } else {
            (&mut me.right, idx - left_size - 1)
        };

        let mut remove_at_node = remove_at_ref
            .clone()
            .expect("empty child but remove search did not conclude");
        let new_child = remove_at_node.remove(remove_at_idx);
        *remove_at_ref = new_child;

        drop(me);

        self.update_props();

        if self.balance_factor().abs() >= 2 {
            let rebalance_result = self.rebalance();
            Some(rebalance_result)
        } else {
            Some(self.clone())
        }
    }

    fn remove_node(&mut self) -> Option<NodeRef<T>> {
        let left = self.left().clone();
        let right = self.right().clone();

        match (left, right) {
            (None, None) => None,
            (None, Some(mut right)) => {
                *right.parent_mut() = self.parent().clone();
                right.update_props();
                Some(right)
            }
            (Some(mut left), None) => {
                *left.parent_mut() = self.parent().clone();
                left.update_props();
                Some(left)
            }
            (Some(_), Some(right)) => {
                let mut successor = right;

                while successor.left().is_some() {
                    let next = successor.left().clone().unwrap();
                    successor = next;
                }

                *successor.left_mut() = self.left().clone();
                if let Some(left) = self.left_mut().as_mut() {
                    *left.parent_mut() = Some(successor.clone());
                }

                let successor_parent = successor.parent().clone().unwrap();

                if successor_parent.ne(self) {
                    if successor_parent.is_left(&successor) {
                        *successor_parent.left_mut() = successor.right().clone();
                    } else {
                        *successor_parent.right_mut() = successor.right().clone();
                    }
                    if let Some(right) = successor.right_mut().as_mut() {
                        *right.parent_mut() = Some(successor_parent.clone());
                    }

                    *successor.right_mut() = self.right().clone();
                    if let Some(right) = self.right_mut().as_mut() {
                        *right.parent_mut() = Some(successor.clone());
                    }

                    let mut node = successor_parent;

                    while node.ne(&successor) {
                        node.update_props();
                        let next_node = node.parent().clone().unwrap();
                        node = next_node;
                    }
                }

                *successor.parent_mut() = self.parent().clone();
                successor.update_props();

                Some(successor)
            }
        }
    }

    fn index(&self) -> usize {
        let mut idx = self.left().as_ref().map_or(0, |n| n.size());
        let mut node = self.clone();

        loop {
            let parent = node.parent().clone();

            if let Some(parent) = parent {
                if parent.is_right(&node) {
                    idx += parent.left().as_ref().map_or(0, |n| n.size()) + 1;
                }
                node = parent;
            } else {
                break;
            }
        }

        idx
    }

    fn update_props(&mut self) {
        let mut me = self.0.borrow_mut();

        me.height = cmp::max(
            me.left.as_ref().map_or(0, |n| n.height()),
            me.right.as_ref().map_or(0, |n| n.height()),
        ) + 1;
        me.size = me.left.as_ref().map_or(0, |n| n.size())
            + me.right.as_ref().map_or(0, |n| n.size())
            + 1;
    }

    fn rebalance(&mut self) -> NodeRef<T> {
        let is_positive = self.balance_factor() > 0;
        let is_right = self.right_height() > self.left_height();

        match (is_right, is_positive) {
            (true, true) => self.rotate_left(),
            (false, false) => self.rotate_right(),
            (true, false) => self.rotate_right_left(),
            (false, true) => self.rotate_left_right(),
        }
    }

    fn rotate_left(&mut self) -> NodeRef<T> {
        let mut right = self.right().clone().expect("right is None in rotate_left");
        let right_left = right.left().clone();

        if let Some(right_left) = right_left {
            *right_left.parent_mut() = Some(self.clone());
            *self.right_mut() = Some(right_left);
        } else {
            *self.right_mut() = None;
        }

        *right.parent_mut() = self.parent().clone();
        *right.left_mut() = Some(self.clone());
        *self.parent_mut() = Some(right.clone());

        self.update_props();
        right.update_props();

        right
    }

    fn rotate_right(&mut self) -> NodeRef<T> {
        let mut left = self.left().clone().expect("left is None in rotate_right");
        let left_right = left.right().clone();

        if let Some(left_right) = left_right {
            *left_right.parent_mut() = Some(self.clone());
            *self.left_mut() = Some(left_right);
        } else {
            *self.left_mut() = None;
        }

        *left.parent_mut() = self.parent().clone();
        *left.right_mut() = Some(self.clone());
        *self.parent_mut() = Some(left.clone());

        self.update_props();
        left.update_props();

        left
    }

    fn rotate_left_right(&mut self) -> NodeRef<T> {
        let mut left = self
            .left()
            .clone()
            .expect("left is None in rotate_left_right");
        let mut left_right = left
            .right()
            .clone()
            .expect("left_right is None in rotate_left_right");

        let left_right_left = left_right.left().clone();
        let left_right_right = left_right.right().clone();

        if let Some(left_right_left) = left_right_left {
            *left_right_left.parent_mut() = Some(left.clone());
            *left.left_mut() = Some(left_right_left);
        } else {
            *left.left_mut() = None;
        }
        if let Some(left_right_right) = left_right_right {
            *left_right_right.parent_mut() = Some(self.clone());
            *self.right_mut() = Some(left_right_right);
        } else {
            *self.right_mut() = None;
        }

        *left_right.parent_mut() = self.parent().clone();
        *self.parent_mut() = Some(left_right.clone());
        *left.parent_mut() = Some(left_right.clone());

        *left_right.left_mut() = Some(left.clone());
        *left_right.right_mut() = Some(self.clone());

        self.update_props();
        left.update_props();
        left_right.update_props();

        left_right
    }

    fn rotate_right_left(&mut self) -> NodeRef<T> {
        let mut right = self
            .right()
            .clone()
            .expect("right is None in rotate_right_left");
        let mut right_left = right
            .left()
            .clone()
            .expect("right_left is None in rotate_right_left");

        let right_left_left = right_left.left().clone();
        let right_left_right = right_left.right().clone();

        if let Some(right_left_left) = right_left_left {
            *right_left_left.parent_mut() = Some(self.clone());
            *self.right_mut() = Some(right_left_left);
        } else {
            *self.right_mut() = None;
        }
        if let Some(right_left_right) = right_left_right {
            *right_left_right.parent_mut() = Some(right.clone());
            *right.left_mut() = Some(right_left_right);
        } else {
            *right.left_mut() = None;
        }

        *right_left.parent_mut() = self.parent().clone();
        *self.parent_mut() = Some(right_left.clone());
        *right.parent_mut() = Some(right_left.clone());

        *right_left.left_mut() = Some(self.clone());
        *right_left.right_mut() = Some(right.clone());

        self.update_props();
        right.update_props();
        right_left.update_props();

        right_left
    }

    fn balance_factor(&self) -> isize {
        self.right_height() as isize - self.left_height() as isize
    }

    fn left_height(&self) -> usize {
        self.left().as_ref().map_or(0, |n| n.height())
    }

    fn right_height(&self) -> usize {
        self.right().as_ref().map_or(0, |n| n.height())
    }

    fn left(&self) -> Ref<Option<NodeRef<T>>> {
        Ref::map(self.0.borrow(), |x| &x.left)
    }

    fn right(&self) -> Ref<Option<NodeRef<T>>> {
        Ref::map(self.0.borrow(), |x| &x.right)
    }

    fn parent(&self) -> Ref<Option<NodeRef<T>>> {
        Ref::map(self.0.borrow(), |x| &x.parent)
    }

    fn left_mut(&self) -> RefMut<Option<NodeRef<T>>> {
        RefMut::map(self.0.borrow_mut(), |x| &mut x.left)
    }

    fn right_mut(&self) -> RefMut<Option<NodeRef<T>>> {
        RefMut::map(self.0.borrow_mut(), |x| &mut x.right)
    }

    fn parent_mut(&self) -> RefMut<Option<NodeRef<T>>> {
        RefMut::map(self.0.borrow_mut(), |x| &mut x.parent)
    }

    fn is_left(&self, node: &NodeRef<T>) -> bool {
        self.left().as_ref().map_or(false, |n| n.eq(node))
    }

    fn is_right(&self, node: &NodeRef<T>) -> bool {
        self.right().as_ref().map_or(false, |n| n.eq(node))
    }

    fn height(&self) -> usize {
        self.0.borrow().height
    }

    fn size(&self) -> usize {
        self.0.borrow().size
    }
}

impl<T: Clone> NodeRef<T> {
    fn collect(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.size());

        go(self, &mut result);

        return result;

        fn go<T: Clone>(node: &NodeRef<T>, vec: &mut Vec<T>) {
            let node = node.0.borrow();

            if let Some(left) = node.left.as_ref() {
                go(left, vec);
            }

            vec.push(node.element.clone());

            if let Some(right) = node.right.as_ref() {
                go(right, vec);
            }
        }
    }
}

impl<T: Debug> Debug for NodeRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_tree(self, f, 0)
    }
}

impl<T: Display> Display for NodeRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        print_tree(self, f)
    }
}

fn debug_tree<T: Debug>(
    node: &NodeRef<T>,
    f: &mut std::fmt::Formatter<'_>,
    depth: usize,
) -> fmt::Result {
    for _ in 0..depth {
        write!(f, "  ")?;
    }

    let node = node.0.borrow();
    writeln!(f, "-({:?})[H{}S{}]", node.element, node.height, node.size)?;

    if let Some(left) = node.left.as_ref() {
        debug_tree(left, f, depth + 1)?;
    }

    if let Some(right) = node.right.as_ref() {
        debug_tree(right, f, depth + 1)?;
    }

    Ok(())
}

fn print_tree<T: Display>(node: &NodeRef<T>, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    let node = node.0.borrow();

    if let Some(left) = node.left.as_ref() {
        print_tree(left, f)?;
    }

    write!(f, "{} ", node.element)?;

    if let Some(right) = node.right.as_ref() {
        print_tree(right, f)?;
    }

    Ok(())
}
