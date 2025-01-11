use {
    flang::*,
    miette::GraphicalReportHandler,
    project::{pack, Package, PACKAGE},
    std::{
        env::{args, current_dir},
        path::PathBuf,
        process,
    },
};

fn main() -> anyhow::Result<()> {
    let target =
        PathBuf::from(args().skip(1).find(|a| !a.starts_with("--")).unwrap_or(current_dir().unwrap().display().to_string()));
    let package = if target.is_dir() {
        Package::from_folder(target)?
    } else {
        Package::from_folder(target.parent().unwrap().to_path_buf())?
    };

    PACKAGE.set((package, None).into()).unwrap();
    process()
}

fn process() -> anyhow::Result<()> {
    let (_, errors) = pack().process()?;
    if !errors.is_empty() {
        errors.iter().for_each(|e| {
            let mut out = String::new();
            let _ = GraphicalReportHandler::default().render_report(&mut out, &e.clone().as_error("Parsing error"));
            println!("{}", out);
        });

        process::exit(1);
    }

    Ok(())
}
