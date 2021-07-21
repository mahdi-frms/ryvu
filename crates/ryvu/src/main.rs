use std::{env::args, fs, io::{self, Read}, process::exit};
use compile::{LexerError, ParserError, compile};
use module::Module;
use network::Network;

mod network;

fn main() {
    let args : Vec<String>= args().collect();
    let address = &args[0];
    let module = match read_file(address) {
        None => {
            eprintln!("could not open file '{}'",address);
            exit(1);
        },
        Some(content) => {
            println!("compiling {}...",address);
            compile_file(content)
        }
    };
    let network = Network::new(module);
    exec_loop(network);    
}

fn compile_file(source:String) -> Module {
    match compile(source.as_str()) {
        Ok(module)=>{
            module
        },
        Err((lerr,perr)) => {
            for err in lerr.iter() {
                print_lerror(err);
            }
            for err in perr.iter() {
                print_perror(err);
            }
            exit(1);
        }
    }
}   

fn print_lerror(err:&LexerError){
    eprintln!("{:?}",err);
}
fn print_perror(err:&ParserError){
    eprintln!("{:?}",err);
}

fn read_file(path:&String)->Option<String>{
    match fs::read_to_string(&path) {
        Err(_)=>None,
        Ok(content)=>Some(content)
    }
}

fn get_bools(mut count:usize)->Vec<bool> {
    
    let mut buffer = [0u8;1];
    let mut input = vec![];

    let zero = '0' as u8;
    let one = '1' as u8;

    while count > 0 {
        let _ = io::stdin().read(&mut buffer);
        if buffer[0] == zero {
            input.push(false);
            count -= 1;
        }
        else if buffer[0] == one {
            input.push(true);
            count -= 1;
        }
    }

    input
}

fn exec_loop(network:Network){
       
}