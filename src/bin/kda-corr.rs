use std::collections::HashMap;
use std::io::BufRead;
use std::io;
//use std::fs::File;
//use std::path::Path;
//use std::io::BufRead;
use std::string::String;
use nalgebra::base::DMatrix;

/**
 * Takes a long sequence of kda stats (see kda-stretch), and computes correlations
 */
fn main() {
    let mut line_count = 0;
    let mut processed_lines =0;
    let mut item_columns: HashMap<String,usize> = HashMap::new();
    let mut data_entries: HashMap<(usize,usize),i32> = HashMap::new();

    for input_line in io::stdin().lock().lines()
    {
        line_count+=1;
        let line:String = match input_line
        {
            Ok(line)=>line,
            Err(_)=>continue,
        };
        let mut tok_iter = line.split_whitespace();
        let row_str = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>continue,
        };
        let row_idx:usize =  match row_str.parse::<usize>()
        {
            Ok(i)=>i,
            Err(e)=>{
                eprintln!("Line {} not a row #:{} (see kda-stretch) Error={}",line_count,row_str,e);
                continue
            },
        };
        let _ignore_this = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>{
                eprintln!("Line {} Cannot process: {}. Expected 3 tokens / line (see kda-stretch).",line_count,line);
                continue;
            },
        };
        let key = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>{
                eprintln!("Line {} Cannot process: {}. Expected 3 tokens / line (see kda-stretch).",line_count,line);
                continue;
            }
        };
        let cur_size:usize = item_columns.len();
        let col_idx = *item_columns.entry(key.to_string()).or_insert(cur_size);
        //insert a 1 (or +=1) the row/column.
        let current_value = data_entries.entry((row_idx,col_idx)).or_insert(0);
        *current_value=*current_value + 1;
        processed_lines+=1;
    }
    let factor_matrix = DMatrix::<f32>::zeros(2, 3);
    eprintln!("{}",factor_matrix);
    eprintln!("Read: {} rows and {} variables",processed_lines,data_entries.len());
}
