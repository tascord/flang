use {
    flang::*,
    std::{fs::File, io::Read},
};

fn main() {
    let mut i = String::new();
    File::open("./input.fl").unwrap().read_to_string(&mut i).unwrap();

    let tree = parser::parse(i.leak()).unwrap();
    let result = runtime::process(tree, None).unwrap();

    println!("Program returns -> {result:?}");
}
