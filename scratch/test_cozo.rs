use cozo::*;
use std::path::PathBuf;

fn main() {
    let db = DbInstance::new("mem", &PathBuf::from(""), Default::default()).unwrap();
    println!("Testing ::fts");
    match db.run_script("::fts_describe node:fts_idx", Default::default(), ScriptMutability::Immutable) {
        Ok(res) => println!("::fts_describe result: {:?}", res),
        Err(e) => println!("::fts_describe error: {:?}", e),
    }
    match db.run_script("::fts_all", Default::default(), ScriptMutability::Immutable) {
        Ok(res) => println!("::fts_all result: {:?}", res),
        Err(e) => println!("::fts_all error: {:?}", e),
    }
}
