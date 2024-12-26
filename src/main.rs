use {
    flang::*,
    project::{Package, SemverPackage},
    std::{
        env::{args, current_dir},
        fs::{File, OpenOptions},
        io::Read,
        path::{Path, PathBuf},
        process,
    },
};

fn main() -> anyhow::Result<()> {
    let target = PathBuf::from(args().skip(1).next().unwrap_or(current_dir().unwrap().display().to_string()));
    let package = if target.is_dir() {
        Package::from_folder(target)?
    } else {
        Package::from_folder(target.parent().unwrap().to_path_buf())?
    };

    println!("{:?}", package);

    let mut input = String::new();
    OpenOptions::new()
        .read(true)
        .open(Path::new(&package.disk_path).join(package.main))?
        .read_to_string(&mut input)
        .unwrap();

    let tree = parser::parse(input.leak()).unwrap();
    let result = runtime::process(tree, None).unwrap();
    println!("Program returns -> {result:?}");

    Ok(())
}
