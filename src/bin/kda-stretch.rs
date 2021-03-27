extern crate regex;
use std::collections::HashMap;
use std::io::Write;
use std::io::{stdin,stdout};
use std::io::BufRead;
use std::string::String;
extern crate kda_tools;

fn main() {

    eprintln!("kda-tools version:{}",kda_tools::version::version());
    eprintln!("Using kvc version:{}",kvc::version());

    //we're humans here, we speak in Natural Numbers, you wouldn't understand, Borg.
    let mut line_count = 1;  
    let sin = stdin();
    let mut line_itr = sin.lock().lines();
    let date_reg = regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    while let Some(Ok(input_line)) = line_itr.next()
    {
        let mut line_strings: HashMap<String,String> = HashMap::new();
        let mut line_counter: HashMap<String,f32> = HashMap::new();
        let mut tok_iter = input_line.split_whitespace();

        while let Some(kvpair) = tok_iter.next(){

            //sure hope I understand what that split_whitespace() was up to.
            assert!(kvpair.len() > 0);
            if kvpair.chars().next().unwrap()=='#'{
                break;
            }
            let mut kvitr = kvpair.split(":");
            if let Some(key)=kvitr.next(){
                //got a key, that's good.
                //if it's a date-matching key, we can specially process that one
                if date_reg.is_match(key){
                    line_strings.insert("Date".to_string(),key.to_string());
                    continue;
                }

                //It's not one of the speically formatted keys, so let's just parse as accumulator keys
                //These are of the form K K K K K , which should compress to K:5
                //or K:4 K, which should compress also to K:5
                //e.g., of the form K:I, and if no :I, then let's assume :1.
                //get val -- thestuff after ':'
                let val=match kvitr.next(){
                    None=>1.0,
                    Some(s)=>{
                        if let Ok(f_val) = s.parse::<f32>(){
                            f_val
                        } else {
                            eprintln!("Got a non-accumulator (int/float) here: {}:{}",key,s);
                            continue;
                        }
                    },
                };
                let countref = line_counter.entry(key.to_string()).or_insert(0.0);
                *countref =  *countref + val;
            } else {
                panic!("Bug! Cannot process: '{}' from '{}'",kvpair,input_line);
            }
        }
        //did we get anything from this?
        if line_strings.len()==0 && line_counter.len()==0
        {
            continue;
        }
    
        for (key,val) in line_strings.into_iter()
        {
            writeln!(stdout(),"{} {} {}",line_count,key,val).unwrap_or(());
        }
        for (key,val) in line_counter.into_iter()
        {
            writeln!(stdout(),"{} {} {}",line_count,key,val).unwrap_or(());
        }
        line_count+=1;
    }
}
