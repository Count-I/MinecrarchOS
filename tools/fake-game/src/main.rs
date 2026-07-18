use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let crash = args.iter().any(|a| a == "--crash");
    let hang = args.iter().any(|a| a == "--hang");

    println!("[fake-game] started pid={}", process::id());

    if hang {
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    } else if crash {
        thread::sleep(Duration::from_secs(3));
        println!("[fake-game] exiting");
        process::exit(1);
    } else {
        thread::sleep(Duration::from_secs(30));
        println!("[fake-game] exiting");
    }
}
