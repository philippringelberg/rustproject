use runner::*;
// Note, We use le (little-endian) byte order
fn main() {
    let mut session = open_session();
    reset_and_halt(&mut session);
    run_to_halt(&mut session); // our first breakpoint
    cycnt_enable(&mut session);
    cycnt_reset(&mut session);

    run_to_halt(&mut session); // our second breakpoint
    let cycle_count = cycnt_read(&mut session);
    println!("cycles {}", cycle_count);
}
