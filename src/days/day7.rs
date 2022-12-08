use crate::{
    parsing::{integer, line_separated},
    Solution,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, multispace1, not_line_ending},
    combinator::{all_consuming, map, not, peek},
    multi::separated_list0,
    sequence::{preceded, separated_pair},
    IResult,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct Day7 {}

impl Solution for Day7 {
    type Part1Result = usize;
    type Part2Result = Self::Part1Result;

    type Input = Vec<Command>;

    fn parse<'a>(
        &mut self,
        input: &'a str,
    ) -> Result<Self::Input, nom::Err<nom::error::Error<&'a str>>> {
        all_consuming(line_separated(command))(input).map(|x| x.1)
    }

    fn run_part_1(&mut self, data: &Self::Input) -> Self::Part1Result {
        const SIZE_LIMIT: usize = 100_000;
        let tree = Tree::build_from_commands(data);
        let result = walk_dir_sizes(&tree);

        result
            .dir_sizes
            .into_iter()
            .filter(|&x| x <= SIZE_LIMIT)
            .sum()
    }

    fn run_part_2(&mut self, data: &Self::Input) -> Self::Part2Result {
        const TOTAL_SIZE: usize = 70_000_000;
        const SIZE_REQUIREMENT: usize = 30_000_000;
        
        let tree = Tree::build_from_commands(data);
        let result = walk_dir_sizes(&tree);
        let available_space = TOTAL_SIZE - result.root_size;
        let size_to_free = SIZE_REQUIREMENT - available_space;

        result
            .dir_sizes
            .into_iter()
            .filter(|&x| x >= size_to_free)
            .min()
            .unwrap()
    }
}

type NodeRef<'a> = Rc<RefCell<Node<'a>>>;

struct Tree<'a> {
    root: NodeRef<'a>,
}

#[derive(Debug, Default)]
struct Node<'a> {
    size: usize,
    children: HashMap<&'a str, NodeRef<'a>>,
    parent: Option<NodeRef<'a>>,
    entry_type: EntryType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EntryType {
    #[default]
    Directory,
    File,
}

impl<'a> Tree<'a> {
    fn build_from_commands(cmds: &'a [Command]) -> Self {
        let root = Rc::new(RefCell::new(Node::default()));
        let mut current_node = root.clone();

        for cmd in cmds {
            match cmd {
                Command::ChangeDirectory(dir) => match dir {
                    Directory::Root => current_node = root.clone(),
                    Directory::Child(name) => {
                        if !current_node.borrow().children.contains_key(name as &str) {
                            Self::add_child_dir(&current_node, name);
                        }
                        let next_node = current_node.borrow().children[name as &str].clone();
                        current_node = next_node;
                    }
                    Directory::Parent => {
                        let next_node = current_node.borrow().parent();
                        current_node = next_node;
                    }
                },
                Command::List(entries) => {
                    for entry in entries {
                        match entry {
                            DirectoryEntry::File(size, name) => {
                                Self::add_child_file(&current_node, name, *size);
                            }
                            DirectoryEntry::Directory(name) => {
                                Self::add_child_dir(&current_node, name);
                            }
                        }
                    }
                }
            }
        }

        Self { root }
    }

    fn add_child_dir(parent: &NodeRef<'a>, name: &'a str) {
        let dir_node = Node::new_dir(parent.clone());
        parent
            .borrow_mut()
            .children
            .insert(name, Rc::new(RefCell::new(dir_node)));
    }

    fn add_child_file(parent: &NodeRef<'a>, name: &'a str, size: usize) {
        let file_node = Node::new_file(parent.clone(), size);
        parent
            .borrow_mut()
            .children
            .insert(name, Rc::new(RefCell::new(file_node)));
    }
}

impl<'a> Node<'a> {
    fn new_file(parent: NodeRef<'a>, size: usize) -> Self {
        Self {
            size,
            children: HashMap::new(),
            parent: Some(parent),
            entry_type: EntryType::File,
        }
    }

    fn new_dir(parent: NodeRef<'a>) -> Self {
        Self {
            size: 0,
            children: HashMap::new(),
            parent: Some(parent),
            entry_type: EntryType::Directory,
        }
    }

    fn parent(&self) -> NodeRef<'a> {
        self.parent.as_ref().cloned().unwrap()
    }
}

struct TreeWalkResult {
    root_size: usize,
    dir_sizes: Vec<usize>,
}

fn walk_dir_sizes(tree: &Tree) -> TreeWalkResult {
    let mut state = TreeWalkResult {
        root_size: 0,
        dir_sizes: Vec::new(),
    };
    state.root_size = walk_rec(&tree.root.borrow(), &mut state);
    return state;

    fn walk_rec(node: &Node, state: &mut TreeWalkResult) -> usize {
        let mut total_size = node.size;

        for child in node.children.values() {
            let child_size = walk_rec(&child.borrow(), state);
            total_size += child_size;
        }

        if node.entry_type == EntryType::Directory {
            state.dir_sizes.push(total_size);
        }

        total_size
    }
}

#[derive(Debug)]
pub enum Command {
    ChangeDirectory(Directory),
    List(Vec<DirectoryEntry>),
}

#[derive(Debug)]
pub enum Directory {
    Root,
    Child(String),
    Parent,
}

#[derive(Debug)]
pub enum DirectoryEntry {
    File(usize, String),
    Directory(String),
}

fn command(input: &str) -> IResult<&str, Command> {
    map(
        separated_pair(
            char('$'),
            multispace1,
            alt((change_directory_command, list_command)),
        ),
        |x| x.1,
    )(input)
}

fn change_directory_command(input: &str) -> IResult<&str, Command> {
    map(separated_pair(tag("cd"), multispace1, directory), |x| {
        Command::ChangeDirectory(x.1)
    })(input)
}

fn list_command(input: &str) -> IResult<&str, Command> {
    map(
        separated_pair(
            tag("ls"),
            line_ending,
            separated_list0(line_ending, preceded(not(peek(char('$'))), entry)),
        ),
        |x| Command::List(x.1),
    )(input)
}

fn directory(input: &str) -> IResult<&str, Directory> {
    alt((root_directory, parent_directory, child_directory))(input)
}

fn root_directory(input: &str) -> IResult<&str, Directory> {
    map(char('/'), |_| Directory::Root)(input)
}

fn child_directory(input: &str) -> IResult<&str, Directory> {
    map(not_line_ending, |x: &str| Directory::Child(x.to_owned()))(input)
}

fn parent_directory(input: &str) -> IResult<&str, Directory> {
    map(tag(".."), |_| Directory::Parent)(input)
}

fn entry(input: &str) -> IResult<&str, DirectoryEntry> {
    alt((file_entry, directory_entry))(input)
}

fn file_entry(input: &str) -> IResult<&str, DirectoryEntry> {
    map(
        separated_pair(integer, multispace1, not_line_ending),
        |(size, name)| DirectoryEntry::File(size, name.to_owned()),
    )(input)
}

fn directory_entry(input: &str) -> IResult<&str, DirectoryEntry> {
    map(
        separated_pair(tag("dir"), multispace1, not_line_ending),
        |(_, dir): (&str, &str)| DirectoryEntry::Directory(dir.to_owned()),
    )(input)
}
