mod dbg;
mod entry;
mod rust_analyzer_actions;
mod shared;
mod stage1;
mod stage2;
mod stage3;
mod util;

fn main() {
    let path = std::env::args().skip(1).next().unwrap_or(String::from("."));
    println!("Reading source from {}", path);

    dbg::enter("stage1and2", &());
    println!("Doing stages 1 and 2");
    let (mut s2_environment, s2_root) = entry::start_from_root(&path).unwrap();
    dbg::leave("stage1and2");
    // println!("{:#?}", s2_environment);
    // println!("root: {:?}", s2_root);
    // println!(
    //     "vomited root:\n{}",
    //     stage2::completely_vomit_item(&s2_environment, s2_root)
    // );

    dbg::enter("stage3", &());
    println!("Doing stage 3");
    let (mut s3_environment, s3_root) = stage3::ingest(&s2_environment, s2_root);
    dbg::leave("stage3");
    // println!("{:#?}", s3_environment);
    // println!("root {:#?}", s3_root);

    dbg::enter("reduce", &());
    println!("Doing reduction");
    s3_environment.reduce_all();
    dbg::leave("reduce");
    // println!("{:#?}", s3_environment);
    // println!("root {:#?}", s3_root);

    println!("\nDisplays:");
    let displays = s3_environment.display_all(&mut s2_environment, s2_root);
    for display in displays {
        println!("\n{} is", display.value_name);
        let value = stage2::completely_vomit_item(&s2_environment, display.vomited_root);
        println!("{}", value);
    }
}
