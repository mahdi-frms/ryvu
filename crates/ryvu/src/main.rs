
use std::{env::args, fs, io::{self, Read, Write}, mem, process::exit};
use compile::{LexerError, ParserError, compile};
use module::Module;
use network::Network;

mod network;

fn main() {
    let args : Vec<String>= args().collect();
    if args.len() < 2 {
        exit(1);
    }
    let address = &args[1];
    let mut module = match read_file(address) {
        None => {
            eprintln!("could not open file '{}'",address);
            exit(1);
        },
        Some(content) => {
            println!("compiling {}...",address);
            compile_file(content)
        }
    };
    let inputs = mem::take(&mut module.inputs);
    let outputs = mem::take(&mut module.outputs);
    let network = Network::new(module);
    exec_loop(network,inputs,outputs);
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

fn get_input(network: &mut Network,inputs:&Vec<usize>){
    let input_data = get_bools(inputs.len());
    for i in 0..inputs.len() {
        let index = inputs[i];
        if input_data[i] {
            network.charge(index);
        }
    }
}

fn set_output(network: &mut Network,outputs:&Vec<usize>){
    for i in 0..outputs.len() {
        let index = outputs[i];
        if network.seek(index) {
            print!("1");
        }
        else {
            print!("0");
        }
    }
    io::stdout().flush().unwrap();
}

fn exec_loop(mut network:Network,inputs:Vec<usize>,outputs:Vec<usize>){
    loop {
        get_input(&mut network, &inputs);
        network.next();
        set_output(&mut network, &outputs);
    }
}