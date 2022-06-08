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
use std::io::{stdin, stdout, Write};
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;

fn after_init_timer(a: Option<&dyn Any>) {
    let a = a.unwrap().downcast_ref::<(u64,String)>().unwrap();
    let target_time = chrono::Local::now().time() + chrono::Duration::seconds(a.0 as i64);

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
----------------------------------------------------------------------------------------------------------
Usage:
    - chimer -t/--timer DURATION \"TIMER NAME\" | Starts a timer that chimes when the duration has passed.
                                   The duration has a format of H:M:S .
    - chimer -s/--stopwatch | Starts a stopwatch that stops when any key is pressed.
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

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}

fn stopwatch(a: Vec<String>) {
    let start_time = chrono::Local::now().time();

    let duration = chrono::NaiveTime::signed_duration_since(chrono::Local::now().time(),start_time);
    let mseconds = duration.num_milliseconds() % 1000;
    let seconds = duration.num_seconds() % 60;
    let minutes = (duration.num_minutes() / 60) % 60;
    let hours = (duration.num_hours() / 60) / 60;

    println!("Press enter to stop the timer...");
    println!("{}:{}:{}:{}", hours, minutes, seconds, mseconds);

    let stdin_channel = spawn_stdin_channel();

    loop {
        let duration = chrono::NaiveTime::signed_duration_since(chrono::Local::now().time(),start_time);
        let mseconds = duration.num_milliseconds() % 1000;
        let seconds = duration.num_seconds() % 60;
        let minutes = (duration.num_minutes() / 60) % 60;
        let hours = (duration.num_hours() / 60) / 60;
        println!("\x1b[1A{}:{}:{}:{}", hours, minutes, seconds, mseconds);
        if stdin_channel.try_recv().is_ok(){
            break;
        }
        thread::sleep(time::Duration::from_millis(1));
    }
}

fn main() {

    type Argop = fn(Vec<String>);

    let help: Argop = help;
    let timer: Argop = timer;
    let stopwatch: Argop = stopwatch;

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
    valid_args.insert(
        "-s".to_string(),
        stopwatch,
    );
    valid_args.insert(
        "--stopwatch".to_string(),
        stopwatch,
    );




    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if valid_args.contains_key(&args[1]){
            valid_args.get(&args[1]).unwrap()(args);
        } else {
            help(args);
        }
    } else {
        help(args);
    }

}
