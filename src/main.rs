extern crate daemonize_me;
extern crate notify_rust;
extern crate chrono;

use std::any::Any;
use std::fs::File;
use std::process::exit;
use std::env;
use std::collections::HashMap;
use std::{thread, time};

pub use daemonize_me::Daemon;
use notify_rust::Notification;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source};




fn after_init_timer(a: Option<&dyn Any>) {
    let a = a.unwrap().downcast_ref::<(u64,String)>().unwrap();
    let target_time = chrono::Local::now().time() + chrono::Duration::from_std(time::Duration::from_secs(a.0)).unwrap();

    loop {
        if chrono::Local::now().time() >= target_time {
            // Get a output stream handle to the default physical sound device
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            // Load a sound from a file, using a path relative to Cargo.toml
            let file = BufReader::new(File::open("alarmSound.mp3").unwrap());
            // Decode that sound file into a source
            let source = Decoder::new(file).unwrap().repeat_infinite();
            // Play the sound directly on the device
            stream_handle.play_raw(source.convert_samples());
            Notification::new()
            .summary(&format!("Timer finished: {}", a.1))
            .timeout(0)
            .show()
            .unwrap()
            .on_close(|| exit(0));
            break;
        }
        thread::sleep(time::Duration::from_secs(1));
    }

}
fn help(a:Vec<String>) {
    println!("Chimer is a pure Rust cli timer & stopwatch application.
--------------------------------------------------------
Usage:
    - chimer -t/--timer DURATION \"TIMER NAME\" | Starts a timer that chimes when the duration has passed.
                                   The duration has a format of H:M:S .
");
}

fn timer(a:Vec<String>) {

    if a.len() > 2 {
        let duration = a[2].split(':')
                           .map(|s| s.parse().expect("Please use the following format: Hours:Minutes:Seconds ."))
                           .collect::<Vec<u64>>();

        if duration.len() != 3 {
            println!("Please use the following format: Hours:Minutes:Seconds .");
            exit(0);
        }

        let duration: u64 = duration[0]*60*60+duration[1]*60+duration[2];

        let mut id = "Timer";

        if a.len() == 4 {
            id = &a[3];
        } else if a.len() > 4 {
            println!("Please be sure to put names with spaces in quotes. example:
chimer -t 0:10:0 \"Go for a walk\" ");
            exit(0);
        }

        let stdout = File::create("info.log").unwrap();
        let stderr = File::create("err.log").unwrap();
        let daemon = Daemon::new()
            .pid_file("example.pid", Some(false))
            .umask(0o000)
            .work_dir(".")
            .stdout(stdout)
            .stderr(stderr)
            // Hooks are optional
            .setup_post_init_hook(after_init_timer, Some(&(duration,id.to_string())))
            .start();

        match daemon {
            Ok(_) => println!("Daemonized with success"),
            Err(e) => {
                eprintln!("Error, {}", e);
                exit(-1);
            },
        }
    }
}

fn main() {

    type Argop = fn(Vec<String>);

    let help: Argop = help;
    let timer: Argop = timer;

    let mut valid_args: HashMap<String, Argop> = HashMap::new();
    valid_args.insert(
        "-h".to_string(),
        help,
    );
    valid_args.insert(
        "--help".to_string(),
        help,
    );
    valid_args.insert(
        "-t".to_string(),
        timer,
    );
    valid_args.insert(
        "--timer".to_string(),
        timer,
    );


    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if valid_args.contains_key(&args[1]){
            valid_args.get(&args[1]).unwrap()(args);
        }
    }

}
