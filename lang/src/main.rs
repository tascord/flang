use {
    clap::Parser,
    flang::*,
    miette::GraphicalReportHandler,
    project::{pack, Package, PACKAGE},
    repl::repl,
    std::{path::PathBuf, process},
};

mod repl;

#[derive(Parser, Debug)]
#[command(version = "1.0.0", about = "Interpreter for flang")]
struct Args {
    #[arg(help = "Path to project root (contains manifold)")]
    project: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(target) = args.project {
        let target = PathBuf::from(target);
        let package = if target.is_dir() {
            Package::from_folder(target)?
        } else {
            Package::from_folder(target.parent().unwrap().to_path_buf())?
        };
        PACKAGE.set((package, None).into()).unwrap();

        return process();
    }

    repl();
    Ok(())
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
