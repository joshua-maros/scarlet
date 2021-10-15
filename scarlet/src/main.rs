#![feature(try_trait_v2)]

mod rust_analyzer_actions;
mod shared;
mod util;
mod entry;
mod stage1;

fn main() {
    let path = std::env::args().skip(1).next().unwrap_or(String::from("."));
    println!("Reading source from {}", path);

    let root = entry::read_root(&path).unwrap();
    println!("{:#?}", root);

    // println!("Doing reduction");
    // s3_environment.reduce_all();
    // println!("{:#?}", s3_environment);
    // println!("root {:#?}", s3_root);

    // println!("\nDisplays:");
    // let displays = s3_environment.display_all(&mut s2_environment, s2_root);
    // for display in displays {
    //     println!("\n{} is", display.value_name);
    //     let value = stage2::completely_vomit_item(&s2_environment,
    // display.vomited_root);     println!("{}", value);
    //     let value = stage2::completely_vomit_item(&s2_environment,
    // display.vomited_type);     println!(":{}", value);
    // }
}
