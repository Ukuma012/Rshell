use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main() {
    loop {
        print!("shell> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let mut args = parts.peekable();

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "exit" => {
                    println!("bye!");
                    return;
                }
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let mut output_file = "";
                    let mut arg_vecs = Vec::new();
                    while let Some(arg) = args.next_if(|s| !s.contains('>')) {
                        arg_vecs.push(arg);
                    }

                    if let Some(redir) = args.peek() {
                        if redir.contains('>') {
                            args.next();
                            if let Some(file) = args.next() {
                                output_file = file;
                            }
                        }
                    }

                    let flags = OFlag::O_RDWR | OFlag::O_CREAT;
                    let fd = open(output_file, flags, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();

                    let output = Command::new(command)
                        .args(arg_vecs)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(_e) => {
                            previous_command = None;
                            eprintln!("Command \"{}\" not found", command);
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            final_command.wait().unwrap();
        }
    }
}
