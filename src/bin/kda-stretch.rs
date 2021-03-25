//use std::collections::HashMap;
use std::io::Write;
use std::io::{stdin,stdout};
use std::io::BufRead;
//use std::fs::File;
//use std::path::Path;
use std::string::String;

fn main() {
    let mut line_count = 0;
    for input_line in stdin().lock().lines()
    {
        let line:String = match input_line
        {
            Ok(line)=>line,
            Err(_)=>continue,
        };
        let mut tok_iter = line.split_whitespace();
        let indexer = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>continue,
        };
        for tok in tok_iter{
            writeln!(stdout(),"{} {} {}",line_count,indexer,tok).unwrap_or(());
        }
        line_count+=1;
    }
}
