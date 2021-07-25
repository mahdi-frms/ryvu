use std::{env::args, fs, io::{self, Read, Write}, mem, process::exit};
use compile::{LexerError, ParserError, compile};
use module::Module;
use network::Network;

mod network;

fn main() {
    let address = get_path();
    let content = read_file(&address);
    let mut module = compile_file(&content);

    let inputs = mem::take(&mut module.inputs);
    let outputs = mem::take(&mut module.outputs);
    let network = Network::new(module);
    exec_loop(network,inputs,outputs);
}

fn get_path() -> String {
    let mut args : Vec<String>= args().collect();
    if args.len() < 2 {
        exit(1);
    }
    std::mem::take(&mut args[1])
}

fn compile_file(source:&String) -> Module {

    let cr = compile(source.as_str(),false,true);
    if let Some(module) = cr.module {
        module
    }
    else {
        for err in cr.lerrors.iter() {
            print_lerror(err);
        }
        for err in cr.perrors.iter() {
            print_perror(err);
        }
        exit(1);
    }
}   

fn print_lerror(err:&LexerError){
    eprintln!("{:?}",err);
}
fn print_perror(err:&ParserError){
    eprintln!("{:?}",err);
}

fn read_file(path:&String)-> String{
    match fs::read_to_string(&path) {
        Err(_)=>{
            eprintln!("could not open file '{}'",path);
            exit(1);
        },
        Ok(content)=>{
            content
        }
    }
}

fn read_bits(mut count:usize)->Vec<bool> {
    let mut buffer = [0u8;1];
    let mut input = vec![];
    let zero = '0' as u8;
    let one = '1' as u8;
    let quit = 'q' as u8;
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
        else if buffer[0] == quit {
            exit(0);
        }
    }
    input
}

fn write_bits(bits:Vec<bool>){
    for i in 0..bits.len() {
        if bits[i] {
            print!("1");
        }
        else {
            print!("0");
        }
    }
    io::stdout().flush().unwrap();
}

fn get_input(network: &mut Network,inputs:&Vec<usize>){
    let input_data = read_bits(inputs.len());
    for i in 0..inputs.len() {
        let index = inputs[i];
        if input_data[i] {
            network.charge(index);
        }
    }
}

fn set_output(network: &mut Network,outputs:&Vec<usize>){
    let mut bits = vec![];
    for i in 0..outputs.len() {
        let index = outputs[i];
        bits.push(network.seek(index));
    }
    write_bits(bits);
}

fn exec_loop(mut network:Network,inputs:Vec<usize>,outputs:Vec<usize>){
    loop {
        get_input(&mut network, &inputs);
        network.next();
        set_output(&mut network, &outputs);
    }
}