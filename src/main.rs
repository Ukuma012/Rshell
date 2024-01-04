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

                    let rawstring;
                    let mut filename = "";
                    let mut arg_vecs = Vec::new();
                    while let Some(arg) = args.next_if(|s| !s.contains('>')) {
                        arg_vecs.push(arg);
                    }

                    println!("Printing args");
                    for arg in args.clone() {
                        println!("{}", arg);
                    }

                    println!("Printing arg_vecs");
                    for arg_vec in arg_vecs.clone() {
                        println!("{}", arg_vec);
                    }

                    if let Some(redir) = args.peek() {
                        rawstring = args.collect::<Vec<&str>>().concat();
                    }

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
