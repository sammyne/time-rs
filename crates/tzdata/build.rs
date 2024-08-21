use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io::Write,
    iter,
    path::{self, Path, PathBuf},
};

use anyhow::Context;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

fn main() -> anyhow::Result<()> {
    let root = env::var("CARGO_MANIFEST_DIR").context("read env 'CARGO_MANIFEST_DIR'")?;
    let out_dir = env::var("OUT_DIR").context("read env 'OUT_DIR'")?;

    println!("out-dir = {}", out_dir);

    let zoneinfo_dir = PathBuf::from(root).join("static").join("zoneinfo");

    let f = {
        let p = PathBuf::from(out_dir).join("zoneinfo.zip");
        File::create(&p).with_context(|| format!("create {p:?}"))?
    };

    let mut zw = ZipWriter::new(f);

    let mut seen = HashSet::<String>::default();
    for p in walk_dir(&zoneinfo_dir).context("walk dir")? {
        let data = fs::read(&p).with_context(|| format!("read {p:?}"))?;
        if p.ends_with(".zip") {
            anyhow::bail!("unexpected file during walk: {p:?}");
        }

        let name = p
            .strip_prefix(&zoneinfo_dir)
            .context("strip path prefix")?
            .to_str()
            .with_context(|| format!("path as str: {p:?}"))?
            .replace(path::MAIN_SEPARATOR, "/");

        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zw.start_file(&name, opts)
            .with_context(|| format!("append {name}"))?;
        zw.write_all(&data)
            .with_context(|| format!("write data for {name}"))?;
        let _ = seen.insert(name);
    }

    if seen.is_empty() {
        anyhow::bail!("did not find any files to add");
    }
    if !seen.contains("US/Eastern") {
        anyhow::bail!("did not find US/Eastern to add");
    }

    Ok(())
}

fn walk_dir(p: &Path) -> anyhow::Result<Box<dyn Iterator<Item = PathBuf>>> {
    let mut out: Box<dyn Iterator<Item = PathBuf>> = Box::new(iter::once(PathBuf::default()));
    out = Box::new(out.skip(1));

    for v in fs::read_dir(p).context("read dir")? {
        let e = v.map_err(|err| anyhow::anyhow!("{err}"))?;
        let t = e
            .file_type()
            .with_context(|| format!("get file type: {:?}", e.path()))?;
        let i: Box<dyn Iterator<Item = PathBuf>> = if t.is_file() {
            Box::new(iter::once(e.path()))
        } else if t.is_dir() {
            walk_dir(&e.path()).with_context(|| format!("walk into dir {:?}", e.path()))?
        } else {
            anyhow::bail!("unexpected file: {:?}", e.path());
        };
        out = Box::new(out.chain(i));
    }

    Ok(out)
}
