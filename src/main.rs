use clap::Parser;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Child;
use std::process::Stdio;

use signal_child::Signalable;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use std::thread::JoinHandle;

use std::{fs, path::PathBuf, process::Command};

use knuffel::Decode;

#[derive(Parser)]
struct Options {
    path: PathBuf,
}

#[derive(Decode, Debug)]
struct Run {
    #[knuffel(argument)]
    name: String,

    #[knuffel(child, unwrap(argument))]
    path: String,

    #[knuffel(child, unwrap(argument))]
    command: String,

    #[knuffel(child, unwrap(arguments))]
    args: Vec<String>,
}

use miette::{Context, Diagnostic, IntoDiagnostic};

use knuffel::Error;
use knuffel::{span::Span, DecodeChildren};

fn parse_config<T>(path: &str) -> miette::Result<T>
where
    T: DecodeChildren<Span>,
{
    let text = fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("cannot read {:?}", path))?;
    Ok(knuffel::parse(path, &text)?)
}

pub fn main() -> miette::Result<()> {
    let mut opts = Options::parse();
    opts.path = opts.path.canonicalize().unwrap();

    std::env::set_current_dir(std::path::Path::new(&opts.path)).unwrap();

    opts.path.push("runner.kdl");

    let runnables: Vec<Run> = parse_config(opts.path.to_str().unwrap())?;

    let mut runners: HashMap<String, (Child, JoinHandle<()>, JoinHandle<()>)> = HashMap::new();

    for runnable in runnables {
        let mut command = Command::new(runnable.command);
        let name = runnable.name.clone();
        let name2 = name.clone();
        command.current_dir(runnable.path);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        for arg in runnable.args {
            command.arg(arg);
        }

        let mut cmd = command.spawn().unwrap();

        let stdout = cmd.stdout.take().unwrap();
        let stderr = cmd.stderr.take().unwrap();

        let stdout_handle = std::thread::spawn(move || {
            let buf_reader = BufReader::new(stdout);
            for line in buf_reader.lines() {
                match line {
                    Ok(l) => {
                        println!("[{name}][stdout] {l}");
                    }
                    _ => return,
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            let buf_reader = BufReader::new(stderr);
            for line in buf_reader.lines() {
                match line {
                    Ok(l) => {
                        println!("[{name2}][stderr] {l}");
                    }
                    _ => return,
                }
            }
        });

        runners.insert(runnable.name, (cmd, stdout_handle, stderr_handle));
    }

    let mut signals = Signals::new(&[SIGTERM]).unwrap();

    let _ = signals.forever().next();

    for (name, (mut running, stdout, stderr)) in runners {
        match running.term() {
            Ok(()) => {
                println!("Terminated {name}");
            }
            _ => running
                .kill()
                .expect(format!("Couldn't kill {name}").as_ref()),
        }

        match stdout.join() {
            Ok(_) => {}
            Err(_) => println!("couldn't end stdout thread for {name}"),
        }

        match stderr.join() {
            Ok(_) => {}
            Err(_) => println!("couldn't end stderr thread for {name}"),
        }
    }

    Ok(())
}
