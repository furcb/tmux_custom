extern crate clap;
extern crate regex;
extern crate subprocess;
extern crate uuid;
#[macro_use]
extern crate lazy_static;
extern crate chrono;

use chrono::NaiveDateTime;
use clap::{App, Arg};
use regex::{Match, Regex};
use std::cmp;
use subprocess::{Exec, Redirection};
use uuid::Uuid;

fn main() {
    // get command line arguments
    let matches = App::new("Tmux custom launcher")
        .version("0")
        .about("Custom interface for tmux, and a terminal")
        .arg(
            Arg::with_name("new")
                .help("create a new session")
                .long("new"),
        )
        .arg(
            Arg::with_name("PREFIX")
                .help("Prefix for tmux session name. i.e. \"prefix_ck2h8c\"")
                .index(1),
        )
        .get_matches();

    // tmux values
    let prefix = matches.value_of("PREFIX").unwrap_or("general");
    let new_session = matches.is_present("new");
    // shell commands
    let tmux_new = "tmux new-session -s ".to_string();
    let tmux_attach = "tmux attach -t ".to_string();

    //get tmux sessions list
    let tmux_sessions = Exec::cmd("tmux")
        .arg("list-sessions")
        .stdout(Redirection::Pipe)
        .capture()
        .expect("Failed to obtain tmux session list")
        .stdout_str();

    //create a session id or use newest session (LIFO)
    if new_session || tmux_sessions.len() == 0 {
        let session_id = format!("{}_{}", prefix, session_suffix()).to_string();

        Exec::shell(tmux_new + &session_id)
            .join()
            .expect("Failed to create a new tmux session");
    } else {
        let session_id = extract_session(&prefix, &tmux_sessions);

        Exec::shell(tmux_attach + &session_id)
            .join()
            .expect("Failed to attach to tmux session");
    }
}

// Read session list and match lines against prefix provided
fn extract_session(prefix: &str, lines: &str) -> String {
    //set up regex string
    let r_prefix = format!("{}_[a-z0-9]+", prefix);
    let session_regex = Regex::new(&r_prefix).expect("Regex build failed");

    let lines = lines.to_string();
    let mut v_t = lines.trim().split('\n').collect::<Vec<&str>>();
    let mut v_t = v_t
        .iter()
        .map(|a| a.to_string())
        .filter(|a| match session_regex.find(&a) {
            Some(_) => true,
            None => false,
        })
        .collect::<Vec<String>>();
    v_t.sort_by(|a, b| time_sort(a, b));

    let v_t = v_t
        .pop()
        .expect("There are no sessions that match the input given.")
        .to_string();
    session_regex.find(&v_t).unwrap().as_str().to_string()
}

// Sort strings based on time provided
fn time_sort(a: &str, b: &str) -> cmp::Ordering {
    lazy_static! {
        static ref date_regex: Regex = Regex::new(r"[A-z]{3}\s[A-z]{3}\s+\d+\s+\d+:\d+:\d+\s+\d+")
            .expect("Regex build failed");
    }

    let a = date_regex.find(a).expect("no matches").as_str();
    let b = date_regex.find(b).expect("no matches").as_str();

    let a = NaiveDateTime::parse_from_str(a, "%a %b %e %T %Y").unwrap();
    let b = NaiveDateTime::parse_from_str(b, "%a %b %e %T %Y").unwrap();

    a.timestamp_millis().cmp(&b.timestamp_millis())
}

// Generate a session suffix using part of a UUID string
fn session_suffix() -> String {
    Uuid::new_v4().to_string()[0..7].to_string()
}
