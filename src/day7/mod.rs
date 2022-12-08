use anyhow::Result;
use dendron::{traverse::DftEvent::Close, tree::HierarchyEditGrantError, tree_node, Node};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, digit1, line_ending, space1},
    combinator::{eof, map, map_res, recognize},
    error::{ErrorKind, FromExternalError, ParseError},
    multi::fold_many1,
    sequence::{delimited, terminated},
    IResult,
};
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
enum TreeBuildCommand {
    MoveToParent,
    CreateDir(String),
    CreateFile(String, usize),
    MoveTo(String),
    DoNothing, // for ls
}

#[derive(Clone, Debug, PartialEq)]
enum FsNode {
    FsDirectory(FsNodeInfo),
    FsFile(FsNodeInfo),
}

impl FsNode {
    fn new_file(name: &str, size: usize) -> Self {
        FsNode::FsFile(FsNodeInfo::new(name, size))
    }

    fn new_dir(name: &str) -> Self {
        FsNode::FsDirectory(FsNodeInfo::new(name, 0))
    }

    fn name(self: &Self) -> &String {
        match &self {
            FsNode::FsDirectory(info) => &info.name,
            FsNode::FsFile(info) => &info.name,
        }
    }

    fn size(self: &Self) -> usize {
        match &self {
            FsNode::FsDirectory(info) => info.size,
            FsNode::FsFile(info) => info.size,
        }
    }

    fn increase_size(self: &mut Self, sz: usize) {
        match self {
            FsNode::FsDirectory(info) => {
                info.size += sz;
            }
            _ => (),
        }
    }

    fn is_directory(&self) -> bool {
        match self {
            FsNode::FsDirectory(_) => true,
            FsNode::FsFile(_) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FsNodeInfo {
    name: String,
    size: usize,
}

impl FsNodeInfo {
    fn new(name: &str, size: usize) -> Self {
        FsNodeInfo {
            name: name.to_owned(),
            size,
        }
    }
}

fn file_name<'a, E>(i: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    alt((
        recognize(delimited(alphanumeric1, char('.'), alphanumeric1)),
        alphanumeric1,
    ))(i)
}

fn size<'a, E>(i: &'a str) -> IResult<&'a str, usize, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map_res(digit1, |s: &str| s.parse::<usize>())(i)
}

fn file_statement<'a, E>(i: &'a str) -> IResult<&'a str, TreeBuildCommand, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let (rest, sz) = size(i)?;
    let (rest, _) = space1(rest)?;
    let (rest, file) = file_name(rest)?;
    let (rest, _) = alt((line_ending, eof))(rest)?;
    Ok((rest, TreeBuildCommand::CreateFile(file.to_owned(), sz)))
}

fn dir_name<'a, E>(i: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    alt((alphanumeric1, tag("/")))(i)
}

fn dir_statement<'a, E>(i: &'a str) -> IResult<&'a str, TreeBuildCommand, E>
where
    E: ParseError<&'a str>,
{
    map(
        delimited(tag("dir "), dir_name, alt((line_ending, eof))),
        |dir| TreeBuildCommand::CreateDir(dir.to_owned()),
    )(i)
}

fn ls_statement<'a, E>(i: &'a str) -> IResult<&'a str, TreeBuildCommand, E>
where
    E: ParseError<&'a str>,
{
    map(terminated(tag("$ ls"), alt((line_ending, eof))), |_| {
        TreeBuildCommand::DoNothing
    })(i)
}

fn cd_statement<'a, E>(i: &'a str) -> IResult<&'a str, TreeBuildCommand, E>
where
    E: ParseError<&'a str>,
{
    map(
        delimited(
            tag("$ cd "),
            alt((dir_name, tag(".."))),
            alt((line_ending, eof)),
        ),
        |dir| {
            if dir == ".." {
                TreeBuildCommand::MoveToParent
            } else {
                TreeBuildCommand::MoveTo(dir.to_owned())
            }
        },
    )(i)
}

fn file_system<'a, E>(i: &'a str) -> IResult<&'a str, Node<FsNode>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map(
        fold_many1(
            alt((file_statement, dir_statement, ls_statement, cd_statement)),
            || Node::new_tree(FsNode::new_dir("/")),
            |node, cmd| match cmd {
                TreeBuildCommand::MoveToParent => node.parent().unwrap_or(node),
                TreeBuildCommand::MoveTo(child) => node
                    .children()
                    .find(|e| *e.borrow_data().name() == child)
                    .unwrap_or(node),
                TreeBuildCommand::CreateDir(dir) => {
                    let grant = node.tree().grant_hierarchy_edit().unwrap();
                    node.create_as_last_child(&grant, FsNode::new_dir(&dir));
                    node
                }
                TreeBuildCommand::CreateFile(file, sz) => {
                    let grant = node.tree().grant_hierarchy_edit().unwrap();
                    node.create_as_last_child(&grant, FsNode::new_file(&file, sz));
                    node.ancestors_or_self()
                        .for_each(|n| n.borrow_data_mut().increase_size(sz));
                    node
                }
                TreeBuildCommand::DoNothing => node,
            },
        ),
        |res| res.root(),
    )(i)
}

fn total_size_of_directories_up_to(fs: &Node<FsNode>, max_size: usize) -> usize {
    fs.depth_first_traverse()
        .filter_map(|e| match e {
            Close(e) => {
                let node: &FsNode = &e.borrow_data();
                match node {
                    FsNode::FsDirectory(info) if info.size < max_size => Some(info.size),
                    _ => None,
                }
            }
            _ => None,
        })
        .sum()
}

fn smallest_directory_to_delete_size(fs: &Node<FsNode>, min_size: usize) -> usize {
    fs.depth_first_traverse()
        .filter_map(|e| match e {
            Close(e) => {
                let node: &FsNode = &e.borrow_data();
                match node {
                    FsNode::FsDirectory(info) if info.size >= min_size => Some(info.size),
                    FsNode::FsDirectory(info) => None,
                    _ => None,
                }
            }
            _ => None,
        })
        .min()
        .unwrap_or(0)
}

pub fn total_size_of_small_directories_and_smallest_to_delete(
    input: &PathBuf,
) -> Result<(usize, usize)> {
    let data = read_to_string(input)?;
    let (rest, fs) = file_system::<()>(&data)?;
    let total_size = total_size_of_directories_up_to(&fs, 100000);
    let fs_size = fs.borrow_data().size();
    let space_to_clear = fs_size - (70_000_000 - 30_000_000);
    let smallest = smallest_directory_to_delete_size(&fs, space_to_clear);
    Ok((total_size, smallest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dir_name() {
        let d = dir_name::<()>("adirectory");
        assert_eq!(d, Ok(("", "adirectory")));
    }

    #[test]
    fn parse_root() {
        let root = dir_name::<()>("/");
        assert_eq!(root, Ok(("", "/")));
    }

    #[test]
    fn parse_dir_statement() {
        let dir_stmt = dir_statement::<()>("dir aeisnieuianst\n");
        assert_eq!(
            dir_stmt,
            Ok(("", TreeBuildCommand::CreateDir("aeisnieuianst".to_string())))
        );
    }

    #[test]
    fn parse_file_name() {
        let f = file_name::<()>("toto.txt");
        assert_eq!(f, Ok(("", "toto.txt")));
    }

    #[test]
    fn parse_size() {
        let s = size::<()>("123457");
        assert_eq!(s, Ok(("", 123457)));
    }

    #[test]
    fn parse_file_statement() {
        let file_stmt = file_statement::<()>("123242 totobubu.sql");
        assert_eq!(
            file_stmt,
            Ok((
                "",
                TreeBuildCommand::CreateFile("totobubu.sql".to_string(), 123242usize)
            ))
        );
    }

    #[test]
    fn parse_cd_up() {
        let cd_up = cd_statement::<()>("$ cd ..");
        assert_eq!(cd_up, Ok(("", TreeBuildCommand::MoveToParent)));
    }

    #[test]
    fn parse_cd_dir() {
        let cd_dir = cd_statement::<()>("$ cd toto");
        assert_eq!(
            cd_dir,
            Ok(("", TreeBuildCommand::MoveTo("toto".to_string())))
        );
    }

    #[test]
    fn parse_tree() {
        let data = r#"$ cd /
$ ls
dir abc
dir cde
12345 a.c
$ cd abc
$ ls
432 b.rs
324 c.cpp
$ cd ..
$ cd cde
$ ls
48730 x.java"#;

        let tree = file_system::<()>(&data);

        assert!(tree.is_ok());
        let tree = tree.unwrap().1;

        let expected = tree_node! {
            FsNode::FsDirectory(FsNodeInfo::new("/", 61831)), [
                /(FsNode::FsDirectory(FsNodeInfo::new("abc", 756)), [
                    FsNode::new_file("b.rs", 432),
                    FsNode::new_file("c.cpp", 324)
                ]),
                /(FsNode::FsDirectory(FsNodeInfo::new("cde", 48730)), [
                    FsNode::new_file("x.java", 48730)
                ]),
                FsNode::new_file("a.c", 12345)
            ]
        };

        assert_eq!(tree.tree(), expected.tree());
    }

    #[test]
    fn find_small_directories() {
        let fs = tree_node! {
            FsNode::FsDirectory(FsNodeInfo::new("/", 61831)), [
                /(FsNode::FsDirectory(FsNodeInfo::new("abc", 756)), [
                    FsNode::new_file("b.rs", 432),
                    FsNode::new_file("c.cpp", 324)
                ]),
                /(FsNode::FsDirectory(FsNodeInfo::new("cde", 48730)), [
                    FsNode::new_file("x.java", 48730)
                ]),
                FsNode::new_file("a.c", 12345)
            ]
        };

        assert_eq!(total_size_of_directories_up_to(&fs, 100000), 111317);
    }
}
