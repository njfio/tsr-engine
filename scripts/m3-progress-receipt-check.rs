//! Offline reader only: no Runner, candidate execution, network, or provider integration.
use std::path::Path;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 6, "out manifest taskfile mode pre_oracle_root");
    let tasks = tsr_batch::read_tasks(Path::new(&args[3])).expect("trusted exported task manifest");
    assert_eq!(tasks.len(), 1);
    let mode = match args[4].as_str() { "record" => tsr_batch::Mode::Record, "rules" => tsr_batch::Mode::Rules, _ => panic!("mode") };
    match tsr_batch::oracle_receipts::verify(Path::new(&args[1]), &args[2], &tasks[0], mode, &args[5]) {
        Ok(success) => println!("{{\"receipt_check_completed\":true,\"verified_success\":{success}}}"),
        Err(error) => { eprintln!("receipt verification failed: {error}"); std::process::exit(1); }
    }
}
