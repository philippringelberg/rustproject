use ktest::{read_ktest, KTEST};

fn main() -> std::io::Result<()> {
    let ktest = read_ktest("test000001.ktest");
    println!("{:?}", ktest);
    Ok(())
}
