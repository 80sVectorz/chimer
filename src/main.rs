extern crate daemonize_me;
extern crate notify_rust;
extern crate chrono;
extern crate serde;
extern crate serde_yaml;

use std::process::exit;
use std::collections::HashMap;
use std::{thread, time};
pub use daemonize_me::Daemon;
use notify_rust::Notification;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::io;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::collections::BTreeMap;

fn update_timer_list(start_time: chrono::NaiveTime, duration: chrono::Duration, id:String) {
    let mut dir = match env::current_exe(){
        Ok(p) => p,
        Err(_) => {
            println!("Failed to get exe path!");
            exit(0);
        },
    };
    dir.pop();
    dir.push("timers.yaml");
    let file = match File::open(&dir){
        Ok(f) => f,
        Err(_) => {
            let mut f = File::create(&dir).expect("Error incountered while creating file.");
            f.write_all("---".as_bytes()).expect("Failed to create timers.yaml");
            File::open(&dir).expect("Failed to load created file")
        },
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).expect("Failed to read contents");

    let doc: BTreeMap<String, String> = match serde_yaml::from_str(&contents){
        Ok(r) => r,
        Err(_) => BTreeMap::new(),
    };

    let seconds = duration.num_seconds() % 60;
    let minutes = (duration.num_seconds() / 60) % 60;
    let hours = (duration.num_seconds() / 60) / 60;
    let duration_string = format!("{}:{}:{}",hours,minutes,seconds);

    let start_time_string = start_time.format("%H:%M:%S").to_string();

    let mut new_doc: BTreeMap<String,String> = BTreeMap::new();
    new_doc.insert(id, format!("{}|{}",duration_string,start_time_string));

    for (k, x) in &doc {
        let splitted = x.split('|')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let duration = splitted[0].split(':')
                                .map(|s| s.parse().expect("timers.yaml has syntax errors"))
                                .collect::<Vec<u64>>();

        if duration.len() != 3 {
            println!("timer.yaml has syntax errors");
            exit(0);
        }

        let duration: u64 = duration[0]*60*60+duration[1]*60+duration[2];

        let start_time = chrono::NaiveTime::parse_from_str(&splitted[1], "%H:%M:%S").expect("Failed to parse time.");

        let target_time = start_time + chrono::Duration::seconds(duration as i64);
        if chrono::Local::now().time() >= target_time {
            continue;
        }
        new_doc.insert(k.to_string(),x.to_string());
    }

    let new_contents = serde_yaml::to_string(&new_doc).expect("Failed to parse BTreeMap to yaml string.");
    std::fs::write(&dir, new_contents).expect("Unable to write file");
}

fn after_init_timer(duration: u64, id: String) {
    let target_time = chrono::Local::now().time() + chrono::Duration::seconds(duration as i64);

    loop {
        if chrono::Local::now().time() >= target_time {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();

            let mut dir = env::current_exe().expect("Failed to get current exe");
            dir.pop();
            dir.push("alarmSound.mp3");
            let file = BufReader::new(File::open(dir).expect("Failed to load audio file."));

            let source = Decoder::new(file).unwrap().repeat_infinite();
            stream_handle.play_raw(source.convert_samples()).expect("Failed to play audio!");
            Notification::new()
            .summary(&format!("Timer finished: {}", id))
            .timeout(0)
            .show()
            .unwrap()
            .on_close(|| exit(0));
            break;
        }
        thread::sleep(time::Duration::from_secs(1));
    }
    let mut dir = match env::current_exe(){
        Ok(p) => p,
        Err(_) => {
            println!("Failed to get exe path!");
            exit(0);
        },
    };
    dir.pop();
    dir.push("timers.yaml");
    let file = match File::open(&dir){
        Ok(f) => f,
        Err(_) => {
            let mut f = File::create(&dir).expect("Error incountered while creating file.");
            f.write_all("---".as_bytes()).expect("Failed to create timers.yaml");
            File::open(&dir).expect("Failed to load created file")
        },
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).expect("Failed to read contents");

    let doc: BTreeMap<String, String> = match serde_yaml::from_str(&contents){
        Ok(r) => r,
        Err(_) => BTreeMap::new(),
    };

    if doc.len() == 1 && doc.contains_key(&id){
        let mut f = File::create(&dir).expect("Error incountered while creating file.");
        f.write_all("---".as_bytes()).expect("Failed to create timers.yaml");
    }
}

fn duration(a:Vec<String>) {
    if a.len() > 1 {
        if a.len() == 3 {
            let timer_name = &a[2];

            let mut dir = match env::current_exe(){
                Ok(p) => p,
                Err(_) => {
                    println!("Failed to get exe path!");
                    exit(0);
                },
            };
            dir.pop();
            dir.push("timers.yaml");
            let file = match File::open(&dir){
                Ok(f) => f,
                Err(_) => {
                    let mut f = File::create(&dir).expect("Error incountered while creating file.");
                    f.write_all("---".as_bytes()).expect("Failed to create timers.yaml");
                    File::open(&dir).expect("Failed to load created file")
                },
            };
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents).expect("Failed to read contents");

            let doc: BTreeMap<String, String> = match serde_yaml::from_str(&contents){
                Ok(r) => r,
                Err(_) => BTreeMap::new(),
            };

            if doc.contains_key(timer_name) {
                let splitted = doc.get(timer_name).expect("Failed to get timer values")
                    .to_string()
                    .split('|')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                let duration = splitted[0].split(':')
                                        .map(|s| s.parse().expect("timers.yaml has syntax errors"))
                                        .collect::<Vec<u64>>();

                if duration.len() != 3 {
                    println!("timer.yaml has syntax errors");
                    exit(0);
                }

                let duration: u64 = duration[0]*60*60+duration[1]*60+duration[2];

                let start_time = chrono::NaiveTime::parse_from_str(&splitted[1], "%H:%M:%S").expect("Failed to parse time.");

                let target_time = start_time + chrono::Duration::seconds(duration as i64);

                let time_left = target_time-chrono::Local::now().time();

                let seconds = time_left.num_seconds() % 60;
                let minutes = (time_left.num_seconds() / 60) % 60;
                let hours = (time_left.num_seconds() / 60) / 60;

                println!("Press enter to stop viewing this timer...");
                println!("Time left on timer \"{}\":  {}:{}:{}", timer_name, hours, minutes, seconds);

                let stdin_channel = spawn_stdin_channel();

                loop {
                    let time_left = target_time-chrono::Local::now().time();
                    let seconds = time_left.num_seconds() % 60;
                    let minutes = (time_left.num_seconds() / 60) % 60;
                    let hours = (time_left.num_seconds() / 60) / 60;

                    println!("\x1b[1ATime left on timer \"{}\":  {}:{}:{}", timer_name, hours, minutes, seconds);

                   if stdin_channel.try_recv().is_ok() || target_time < chrono::Local::now().time(){
                       break;
                   }
                    thread::sleep(time::Duration::from_millis(1));
                }
            } else {
                println!("Timer does not exist: {}", timer_name);
                exit(0);
            }
        } else if a.len() > 3 {
             println!("Please be sure to put names with spaces in quotes. example:
chimer -d \"Go for a walk\" ");
            exit(0);
        }
    } else {
        println!("Please specify which timer you want to view. example:
chimer -d \"Go for a walk\" ");
    }
}

fn list(_a:Vec<String>){
     let mut dir = match env::current_exe(){
        Ok(p) => p,
        Err(_) => {
            println!("Failed to get exe path!");
            exit(0);
        },
    };
    dir.pop();
    dir.push("timers.yaml");
    let file = match File::open(&dir){
        Ok(f) => f,
        Err(_) => File::create(&dir).expect("Error incountered while creating file."),
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).expect("Failed to read contents");

    let doc: BTreeMap<String, String> = serde_yaml::from_str(&contents).expect("Failed to convert file contents to rust type");

    for (k, x) in &doc {
        let splitted = x.split('|')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let duration = splitted[0].split(':')
                                .map(|s| s.parse().expect("timers.yaml has syntax errors"))
                                .collect::<Vec<u64>>();

        if duration.len() != 3 {
            println!("timer.yaml has syntax errors");
            exit(0);
        }

        let duration: u64 = duration[0]*60*60+duration[1]*60+duration[2];

        let start_time = chrono::NaiveTime::parse_from_str(&splitted[1], "%H:%M:%S").expect("Failed to parse time.");

        let target_time = start_time + chrono::Duration::seconds(duration as i64);

        let time_left = target_time-chrono::Local::now().time();

        let seconds = time_left.num_seconds() % 60;
        let minutes = (time_left.num_seconds() / 60) % 60;
        let hours = (time_left.num_seconds() / 60) / 60;

        println!("{} | {}:{}:{}", k, hours,minutes,seconds);
    }
}

fn help(_a:Vec<String>) {
    println!("Chimer is a pure Rust cli timer & stopwatch application.
----------------------------------------------------------------------------------------------------------
Usage:
    - chimer -t/--timer DURATION \"TIMER NAME\" | Starts a timer that chimes when the duration has passed.
                                   The duration has a format of H:M:S .
    - chimer -d/--duration \"TIMER NAME\" | Checks and shows the time left on the given timer until enter is pressed.
    - chimer -l/--list | Shows a list of all running timers.
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

        update_timer_list(chrono::Local::now().time(), chrono::Duration::seconds(duration as i64), id.to_string());

        let daemon = Daemon::new()
            .umask(0o000)
            .work_dir(".")
            .start();

        match daemon {
            Ok(_) => println!("Daemonized with success"),
            Err(e) => {
                eprintln!("Error, {}", e);
                exit(-1);
            },
        }
        after_init_timer(duration,id.to_string());
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

fn stopwatch(_a: Vec<String>) {
    let start_time = chrono::Local::now().time();

    let duration = chrono::NaiveTime::signed_duration_since(chrono::Local::now().time(),start_time);
    let mseconds = duration.num_milliseconds() % 1000;
    let seconds = duration.num_seconds() % 60;
    let minutes = (duration.num_seconds() / 60) % 60;
    let hours = (duration.num_seconds() / 60) / 60;

    println!("Press enter to stop the timer...");
    println!("{}:{}:{}:{}", hours, minutes, seconds, mseconds);

    let stdin_channel = spawn_stdin_channel();

    loop {
        let duration = chrono::NaiveTime::signed_duration_since(chrono::Local::now().time(),start_time);
        let mseconds = duration.num_milliseconds() % 1000;
        let seconds = duration.num_seconds() % 60;
        let minutes = (duration.num_seconds() / 60) % 60;
        let hours = (duration.num_seconds() / 60) / 60;
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
    let duration: Argop = duration;
    let list: Argop = list;

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
        "-d".to_string(),
        duration,
    );
    valid_args.insert(
        "--duration".to_string(),
        duration,
    );
    valid_args.insert(
        "-l".to_string(),
        list,
    );
    valid_args.insert(
        "--list".to_string(),
        list,
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
