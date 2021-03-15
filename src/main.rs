use std::{io, env, process, thread};
use std::io::{Write,Read};
use std::fs::File;
use std::ffi::CString;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use nix::unistd::{fork, ForkResult, execvp};
use nix::sys::wait::waitpid;
use std::path::{Path};

fn main() -> io::Result<()> {
    loop {
        let path = env::current_dir()?;
        println!("{}", path.display());
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("not valid input.");
        if input.contains("webserver"){
            let listener = TcpListener::bind("localhost:8888")?;
            let total_requests = Arc::new(Mutex::new(0));
            let valid_requests = Arc::new(Mutex::new(0));
            for stream in listener.incoming(){
                let total_requests = total_requests.clone();
                let valid_requests = valid_requests.clone();
                thread::spawn(move || {
                    handle(stream.unwrap(), valid_requests.clone());
                    let mut total_requests = total_requests.lock().unwrap();
                    *total_requests += 1;
                    let valid_requests = valid_requests.lock().unwrap();
                    println!("Total: {} Valid: {}",*total_requests ,*valid_requests );
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

fn handle(mut stream: TcpStream, num: Arc<Mutex<i32>>) {
    let ip = stream.peer_addr().unwrap();
    println!("{}",ip);
    let mut message = String::new();
    loop {
        let mut buffer = [0; 500];
        let bytes_read = stream.read(&mut buffer).unwrap();
        if bytes_read == 0 { break }
        let buff = from_utf8(&buffer[0..bytes_read]).unwrap();
        message.push_str(buff);
        if message.ends_with("\r\n\r\n") { break }
        if message.ends_with("\n\n") { break }
    }
    println!("Message Recieved: {}",message);
    let file: String =  message.split("GET ").collect();
    let filename: String = file.split(" HTTP").take(1).collect();
    let path = Path::new(filename.trim());

    if path.parent().unwrap() == Path::new("/") && path.is_file() {
        let mut that = num.lock().unwrap();
        *that += 1;
        let mut file_contents = String::new();
        let new_filename: String = filename.split("/").collect();
        let mut file = File::open(format!("{}", new_filename)).unwrap();
        loop {
            let mut new_buffer = [0; 500];
            let new_bytes_read = file.read(&mut new_buffer).unwrap();
            let new_buff = from_utf8(&new_buffer[0..new_bytes_read]).unwrap();
            file_contents.push_str(new_buff);
            if new_bytes_read == 0 { break }
        }
        let reply_message = format!("<html>
<body>
<h1>Message received</h1>
Requested file: {} <br>
File contents: {} <br>
</body>
</html>", filename, file_contents);
        let reply = format!("HTTP/1.1 200 OK
Content-Type: text/html; charset=UTF-8
Content-Length: {}

{}
",reply_message.len(), reply_message);
        stream.write(reply.as_bytes()).unwrap();
    }

    if !(path.parent().unwrap() == Path::new("/")){
        let reply = format!("HTTP/1.1 403 Forbidden");
        stream.write(reply.as_bytes()).unwrap();
    }

    if !path.is_file() {
        let reply = format!("HTTP/1.1 404 Not Found");
        stream.write(reply.as_bytes()).unwrap();
    }

    else {
        let reply = format!("HTTP/1.1 404 Not Found");
        stream.write(reply.as_bytes()).unwrap();
    }
}
