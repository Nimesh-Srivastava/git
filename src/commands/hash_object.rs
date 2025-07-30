use crate::objects::Object;
use anyhow::Context;
use std::path::Path;

pub(crate) fn invoke(write: bool, file: &Path) -> anyhow::Result<()> {
    let object = Object::blob_from_file(file).context("open blob input file")?;
    let hash = if write {
        object
            .write_to_objects()
            .context("write data into blob file")?
    } else {
        object
            .write(std::io::sink())
            .context("write data into blob file")?
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
