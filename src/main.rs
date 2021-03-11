use std::{io, env, process, thread};
use std::io::{Write,Read};
use std::ffi::CString;
use std::str::from_utf8;
use nix::unistd::{fork, ForkResult, execvp};
use nix::sys::wait::waitpid;
use std::net::{TcpListener, TcpStream};

fn main() -> io::Result<()> {
    loop {
        let path = env::current_dir()?;
        println!("{}", path.display());
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("not valid input.");
        if input.contains("webserver"){
            let listener = TcpListener::bind("localhost:8888")?;
            for stream in listener.incoming(){
                thread::spawn(move || {
                    handle(stream.unwrap());
                });
            }
        }
        if input.trim() == "exit"{
            process::exit(0x0100);
        }
        else {
            match unsafe { fork() }.unwrap() {
                ForkResult::Parent { child } => {
                    waitpid(child, None).expect("incorrect input");
                }
                ForkResult::Child => {
                    let input = input.trim();
                    let c_input = CString::new(input).unwrap();
                    let externalized =  externalize(input);
                    execvp(c_input.as_c_str(), &externalized).unwrap();
                }
            }
        }
    }
}

fn externalize(command: &str) -> Box<[CString]> {
    let converted = command.split_whitespace()
        .map(|s| CString::new(s).unwrap())
        .collect::<Vec<_>>();
    converted.into_boxed_slice()
}

fn handle(mut stream: TcpStream) {
    let ip = stream.peer_addr().unwrap();
    println!("{}",ip);
    let mut message = String::new();
    loop {
        let mut buffer = [0; 500];
        stream.read(&mut buffer).unwrap();
        let buff = from_utf8(&buffer).unwrap();
        message.push_str(buff);
        if message.ends_with("\r\n\r\n") {
            break
        }
        if message.ends_with("\n\n") {
            break
        }
    }
    println!("{}",message);
    let file: String =  message.split("GET").collect();
    let filename: String = file.split("HTTP").take(1).collect();
    println!("filename: {}",filename);
    let reply_message = format!("<html>
<body>
<h1>Message received</h1>
Requested file: {} <br>
</body>
</html>", filename);

    let reply = format!("HTTP/1.1 200 OK
Content-Type: text/html; charset=UTF-8
Content-Length: {}

{}
",reply_message.len(), reply_message);

    stream.write(reply.as_bytes()).unwrap();
}