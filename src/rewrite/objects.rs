use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};

use super::errors::RewriteError;
use super::model::{CommitMetadata, ObjectId, RepoPath, Signature, TreeEntry};

pub(crate) type TreeSnapshot = BTreeMap<RepoPath, TreeEntry>;

pub(crate) struct CommitObject {
    pub tree: ObjectId,
    pub parents: Vec<ObjectId>,
    pub metadata: CommitMetadata,
}

pub(crate) fn read_commit(runner: &GitRunner, id: &ObjectId) -> Result<CommitObject, RewriteError> {
    let mut args = GitRunner::command_args(&["cat-file", "commit"]);
    args.push(OsString::from(id.as_str()));
    let output = runner.run(GitCommand::read(args))?;
    parse_commit(&output.stdout).map_err(RewriteError::Parse)
}

pub(crate) fn read_tree(runner: &GitRunner, id: &ObjectId) -> Result<TreeSnapshot, RewriteError> {
    let mut args = GitRunner::command_args(&["ls-tree", "-r", "-z", "--full-tree"]);
    args.push(OsString::from(id.as_str()));
    let output = runner.run(GitCommand::read(args))?;
    parse_tree(&output.stdout).map_err(RewriteError::Parse)
}

fn parse_commit(bytes: &[u8]) -> Result<CommitObject, String> {
    let separator = bytes.windows(2).position(|pair| pair == b"\n\n");
    let Some(separator) = separator else {
        return Err("commit object has no header separator".to_string());
    };
    let mut tree = None;
    let mut parents = Vec::new();
    let mut author = None;
    let mut committer = None;
    for line in bytes[..separator].split(|byte| *byte == b'\n') {
        parse_commit_header(line, &mut tree, &mut parents, &mut author, &mut committer)?;
    }
    let tree = tree.ok_or_else(|| "commit object has no tree".to_string())?;
    let author = author.ok_or_else(|| "commit object has no author".to_string())?;
    let committer = committer.ok_or_else(|| "commit object has no committer".to_string())?;
    Ok(CommitObject {
        tree,
        parents,
        metadata: CommitMetadata {
            author,
            committer,
            message: bytes[separator + 2..].to_vec(),
        },
    })
}

fn parse_commit_header(
    line: &[u8],
    tree: &mut Option<ObjectId>,
    parents: &mut Vec<ObjectId>,
    author: &mut Option<Signature>,
    committer: &mut Option<Signature>,
) -> Result<(), String> {
    if let Some(value) = line.strip_prefix(b"tree ") {
        *tree = Some(ObjectId::from_bytes(value)?);
    }
    if let Some(value) = line.strip_prefix(b"parent ") {
        parents.push(ObjectId::from_bytes(value)?);
    }
    if let Some(value) = line.strip_prefix(b"author ") {
        *author = Some(parse_signature(value)?);
    }
    if let Some(value) = line.strip_prefix(b"committer ") {
        *committer = Some(parse_signature(value)?);
    }
    Ok(())
}

fn parse_signature(bytes: &[u8]) -> Result<Signature, String> {
    let value = String::from_utf8(bytes.to_vec()).map_err(|_| "signature is not UTF-8")?;
    let open = value
        .rfind('<')
        .ok_or_else(|| "signature has no email".to_string())?;
    let close = value
        .rfind('>')
        .ok_or_else(|| "signature has no email end".to_string())?;
    let mut date = value[close + 1..].split_whitespace();
    let timestamp = date
        .next()
        .ok_or_else(|| "signature has no timestamp".to_string())?;
    let timezone = date
        .next()
        .ok_or_else(|| "signature has no timezone".to_string())?;
    Ok(Signature {
        name: value[..open].trim().to_string(),
        email: value[open + 1..close].to_string(),
        date: format!("{timestamp} {timezone}"),
    })
}

fn parse_tree(bytes: &[u8]) -> Result<TreeSnapshot, String> {
    let mut tree = BTreeMap::new();
    for record in bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let tab = record.iter().position(|byte| *byte == b'\t');
        let Some(tab) = tab else {
            return Err("tree record has no path separator".to_string());
        };
        let fields: Vec<&[u8]> = record[..tab].split(|byte| *byte == b' ').collect();
        if fields.len() != 3 {
            return Err("tree record has invalid metadata".to_string());
        }
        let path = RepoPath::from_bytes(&record[tab + 1..])?;
        let mode = String::from_utf8(fields[0].to_vec()).map_err(|_| "tree mode is not UTF-8")?;
        let kind = String::from_utf8(fields[1].to_vec()).map_err(|_| "tree kind is not UTF-8")?;
        let object = ObjectId::from_bytes(fields[2])?;
        tree.insert(path, TreeEntry { mode, kind, object });
    }
    Ok(tree)
}
