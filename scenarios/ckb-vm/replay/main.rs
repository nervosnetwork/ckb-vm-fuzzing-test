#[path = "../../ckb-vm/fuzz/fuzz_targets/cases.rs"]
mod cases;

use std::{env, fs, path::PathBuf};

fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    assert_eq!(args.len(), 2, "expected target and seed file or directory");
    let target = args[0].to_str().unwrap();
    let run: fn(&[u8]) = match target {
        "backend_steps" => cases::backend_steps,
        "snapshot2_continue" => cases::snapshot2_continue,
        "elf_page_edges" => cases::elf_page_edges,
        _ => panic!("no native replay for {target}"),
    };
    let path = PathBuf::from(&args[1]);
    let mut files = if path.is_dir() {
        fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|p| p.is_file())
            .collect::<Vec<_>>()
    } else {
        vec![path]
    };
    files.sort();
    assert!(!files.is_empty(), "empty seed directory for {target}");
    for file in &files {
        eprintln!("{target} {}", file.display());
        run(&fs::read(file).unwrap());
    }
    println!("{target}: {} cases passed", files.len());
}
