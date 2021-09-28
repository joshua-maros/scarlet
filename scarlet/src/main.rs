#![feature(try_trait_v2)]

mod entry;
mod shared;
mod stage1;
mod stage2;
mod stage3;
mod stage4;
mod util;

fn main() {
    let path = std::env::args().skip(1).next().unwrap_or(String::from("."));
    println!("Reading source from {}", path);

    println!("Doing stages 1 and 2");
    let environment = entry::start_from_root(&path).unwrap();
    println!("{:#?}", environment);

    println!("Doing stage 3");
    let environment = stage3::ingest(&environment).unwrap();
    println!("{:#?}", environment);

    println!("Doing stage 4");
    let mut environment = stage4::ingest(environment).unwrap();
    println!("{:#?}", environment);

    println!("Infos:");
    environment.display_infos();

    // println!("Doing reduce");
    // stage4::reduce(&mut environment);
    // println!("{:#?}", environment);

    println!("Doing type check");
    stage4::type_check(&environment).unwrap();

    println!("Infos:");
    environment.display_infos();
}
