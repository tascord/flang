use {
    flang::*,
    project::{pack, Package, PACKAGE},
    std::{
        env::{args, current_dir},
        path::PathBuf,
    },
};

fn main() -> anyhow::Result<()> {
    let target = PathBuf::from(args().skip(1).next().unwrap_or(current_dir().unwrap().display().to_string()));
    let package = if target.is_dir() {
        Package::from_folder(target)?
    } else {
        Package::from_folder(target.parent().unwrap().to_path_buf())?
    };

    PACKAGE.set((package, None).into()).unwrap();
    pack().process()?;
    Ok(())
}
