use std::collections::HashMap;
use std::io::BufRead;
use std::io;
//use std::fs::File;
//use std::path::Path;
//use std::io::BufRead;
use std::string::String;

/**
 * 
 */
fn main() {
    let mut line_count = 0;
    let mut item_counts: HashMap<String,u32> = HashMap::new();

    for input_line in io::stdin().lock().lines()
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
            eprintln!("{} {} {}",line_count,indexer,tok);
        }
        line_count+=1;
    }
}
