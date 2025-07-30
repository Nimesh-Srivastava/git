use crate::objects::{Kind, Object};
use anyhow::{Context, Ok};
use std::fmt::Write;
use std::io::Cursor;

pub(crate) fn invoke(
    message: String,
    parent_hash: Option<String>,
    tree_hash: String,
) -> anyhow::Result<()> {
    let mut commit = String::new();

    writeln!(commit, "tree {tree_hash}")?;

    if let Some(parent_hash) = parent_hash {
        writeln!(commit, "parent_hash {parent_hash}")?;
    }

    writeln!(
        commit,
        "author test_user <test@user_mail.com> 1123764321 +0100"
    )?;

    writeln!(
        commit,
        "commiter test_user <test@user_mail.com> 1123764321 +0100"
    )?;

    writeln!(commit, "")?;
    writeln!(commit, "{message}")?;

    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit.len() as u64,
        reader: Cursor::new(commit),
    }
    .write_to_objects()
    .context("write commit tree object")?;

    println!("{}", hex::encode(hash));

    Ok(())
}
