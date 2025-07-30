use crate::objects::{Kind, Object};
use anyhow::{Context, Ok};
use is_executable::IsExecutable;
use std::cmp::Ordering;
use std::fs;
use std::io::Cursor;
use std::path::Path;

fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("read directory {}", path.display()))?;

    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("invalid directory in {}", path.display()))?;
        let file_name = entry.file_name();
        let meta = entry
            .metadata()
            .context("extract metadata before comparison")?;
        entries.push((entry, file_name, meta));
    }

    entries.sort_unstable_by(|a, b| {
        let x = &a.1;
        let y = &b.1;

        let x = x.as_encoded_bytes();
        let y = y.as_encoded_bytes();

        let common_len = std::cmp::min(x.len(), y.len());

        match x[..common_len].cmp(&y[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }
        if x.len() == y.len() {
            return Ordering::Equal;
        }

        let x_char = if let Some(c) = x.get(common_len).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        let y_char = if let Some(c) = y.get(common_len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        x_char.cmp(&y_char)
    });

    let mut tree_object = Vec::new();

    for (entry, file_name, meta) in entries {
        // remove the .git directory from current commit
        if file_name == ".git" {
            continue;
        }

        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if entry.path().is_executable() {
            "100755"
        } else {
            "100644"
        };

        let path = entry.path();

        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&entry.path())? else {
                continue;
            };

            hash
        } else {
            let tmp = "temporary";
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write(std::fs::File::create(tmp).context("construct temporary file for tree")?)
                .context("write out tree object")?;

            let hash_hex = hex::encode(hash);
            fs::create_dir_all(format!(".git/objects/{}", &hash_hex[..2]))
                .context("create subdir of .git/objects")?;

            std::fs::rename(
                tmp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
            )
            .context("move tree file into .git/objects")?;

            hash
        };

        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }

    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("write out tree object")?,
        ))
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("construct root tree object")? else {
        anyhow::bail!("cannot make tree object for empty tree");
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
