extern crate regex;
use regex::Regex;
use std::collections::HashMap;
use std::io::Lines;
use std::io::BufRead;

pub fn load_from_stream<B:BufRead>( lines_input:Lines<B>)-> (
    usize, //n rows
    HashMap<(usize,usize),f32> , //kdab_entries
    HashMap<(usize,usize),f32> , // data_entries
    HashMap<usize,String>, //col_to_name
    HashMap<String,usize> //name_to_col
)
{
    let mut rows = 0;
    let mut col_to_name: HashMap<usize,String> = HashMap::new();
    let mut name_to_col: HashMap<String,usize> = HashMap::new();
    let mut data_entries: HashMap<(usize,usize),f32> = HashMap::new();
    let mut kdab_entries: HashMap<(usize,usize),f32> = HashMap::new();

    let keywords = kvc::get_reserved_matchers();
    for line_res in lines_input{
        let line = match line_res{
            Ok(l)=>l,
            Err(_)=>"".to_string(),
        };
        eprintln!("{}",line);
        let (key_counts,_)=kvc::read_kvc_line(&line,&keywords);
        if key_counts.len()> 0
        {
            rows+=1;
            for (key,count) in key_counts{
                match &key[..]{
                    "K"=> {
                        let cur_count_ref = kdab_entries.entry( (rows,0)).or_insert(0.0);
                        *cur_count_ref = *cur_count_ref + count;
                    },
                    "D"=> {
                        let cur_count_ref = kdab_entries.entry( (rows,1)).or_insert(0.0);
                        *cur_count_ref = *cur_count_ref + count;
                    },
                    "A"=> {
                        let cur_count_ref = kdab_entries.entry( (rows,2)).or_insert(0.0);
                        *cur_count_ref = *cur_count_ref + count;
                    },
                    "B"=> {
                        let cur_count_ref = kdab_entries.entry( (rows,3)).or_insert(0.0);
                        *cur_count_ref = *cur_count_ref + count;
                    },
                    _ => {
                        let colsize = name_to_col.len();
                        let colidx = name_to_col.entry(key.to_string()).or_insert(colsize);
                        col_to_name.insert(*colidx,key.to_string());
                        let cur_count_ref = data_entries.entry( (rows,*colidx)).or_insert(0.0);
                        *cur_count_ref = *cur_count_ref + count;
                    }
                }
            }
        }
    }
    return (rows,kdab_entries,data_entries,col_to_name,name_to_col);
}

pub fn print_nicely(name:String,mean:f32,stdd:f32,char_max:i32,min_val:f32,max_val:f32)
{
    eprint!("{:>20} {:>4} +/-  {:<4} |", 
        name, 
        std::format!("{:2.2}",mean),
        std::format!("{:2.2}",stdd)
    );
    let spread = max_val - min_val;
    let pct_left  = (mean-stdd-min_val)/spread;
    let chars_left = pct_left * char_max as f32;
    let pct_mid = (mean-min_val)/spread;
    let chars_mid = pct_mid * char_max as f32;
    let pct_right = (mean+stdd-min_val)/spread;
    let chars_right = pct_right  * char_max as f32;
    eprint!("|");
    for _ in 0..chars_left as i32{
        eprint!(" ");
    }
    for _ in chars_left as i32..chars_mid as i32{
        eprint!("-");
    }
    eprint!("({:2.2})",mean);
    for _ in chars_mid as i32..chars_right as i32{
        eprint!("-");
    }
    for _ in chars_right as i32..char_max {
        eprint!(" ");
    }
    eprint!("|");
    eprintln!("");
}