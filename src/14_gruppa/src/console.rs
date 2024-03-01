use spin::Mutex;
use alloc::string::String;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROMPT: Mutex<String> = {
        Mutex::new(String::new())
    };
}

pub fn pop() {
    let mut prompt = PROMPT.lock();
    prompt.pop();
}

pub fn push(character: char) {
    match character as u8 {
        0x20..=0x7e => {
            let mut prompt = PROMPT.lock();
            prompt.push(character);
        }
        _ => {}
    }
}

pub fn eval() {
    pub const COMMANDS: [&str; 2] = ["pause", "commands"];

    let command = {
        let mut prompt = PROMPT.lock();
        let out = prompt.clone();
        prompt.clear();
        out
    };
    
    if COMMANDS.iter().any(|&p| p == command) {
        match command.as_str() {
            "pause" => {
                println!("Pause");
            },
            "commands" => {
                println!("\nAvaliable commands:\n{:?}", COMMANDS);
            },
            _ => println!("Unknown command"),
        }
    } else {
        println!("\nYou just wrote:\n  {}", command);
        print_prompt();
    }
}

pub fn print_prompt() {
    let mut vga = crate::vga::VGA_WRITER.lock();
    vga.write_prompt();
}

pub fn init() {
    print_prompt();
}
