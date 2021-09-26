#![feature(try_trait_v2)]

use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
};

use shared::{Definitions, ItemId};
use stage2::structure::Environment;

mod entry;
mod shared;
mod stage1;
mod stage2;
mod stage3;
mod stage4;
mod util;

fn main() {
    println!("Doing stages 1 and 2");
    let environment = entry::start_from_root(".").unwrap();
    println!("{:#?}", environment);

    println!("Doing stage 3");
    let environment = stage3::ingest(&environment).unwrap();
    println!("{:#?}", environment);

    println!("Doing stage 4");
    let mut environment = stage4::ingest(environment).unwrap();
    println!("{:#?}", environment);

    println!("Doing type check");
    stage4::type_check(&environment).unwrap();

    println!("Doing reduce");
    stage4::reduce(&mut environment);
    println!("{:#?}", environment);

    println!("Infos:");
    environment.display_infos();
}
