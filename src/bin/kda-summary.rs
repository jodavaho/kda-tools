//use std::collections::HashMap;
use std::io::Write;
use std::io::{stdin,stdout};
use std::io::BufRead;
//use std::fs::File;
//use std::path::Path;
use std::string::String;

fn main() {
    //parse the i/o "item" count
    let mut index = 0;
    let mut last_index = -1;

    //human-readable N
    let mut match_count = 1; 
    let mut date_string = String::new();
    let mut linebuf = String::new();
    
    let mut kills = 0.0;
    let mut deaths = 0.0;
    let mut assists = 0.0;
    let mut bounties = 0.0;
    let mut kc= 0;
    let mut dc= 0;
    let mut ac= 0;
    let mut bc= 0;

    //create and dump header for the stream.
    let mut header = String::new();

    header+=&std::format!("{:>5} ","n");
    header+=&std::format!("{:>10} ","Date");

    header+=&std::format!("{:>3} ","K");
    header+=&std::format!("{:>3} ","D");
    header+=&std::format!("{:>3} ","A");
    header+=&std::format!("{:>3} ","B");
    header+=&std::format!("{:>5} ","KDA");

    header+=&std::format!("{:>5} ","sK");
    header+=&std::format!("{:>5} ","sD");
    header+=&std::format!("{:>5} ","sA");
    header+=&std::format!("{:>5} ","sB");

    header+=&std::format!("{:>5} ","mKDA");
    header+=&std::format!("{:>5} ","mK");
    header+=&std::format!("{:>5} ","mD");
    header+=&std::format!("{:>5} ","mA");
    header+=&std::format!("{:>5} ","mB");

    writeln!(stdout(),"{}",header).unwrap_or(());

    for input_line in stdin().lock().lines()
    {
        let line:String = match input_line
        {
            Ok(line)=>line,
            Err(_)=>break, //this is a condition similar to 'there are no more lines to process'
            //At this point, we should be dumping the last data, which happens at the end.
        };

        //let's see what this line contains:
        let mut tok_iter = line.split_whitespace();

        //getting the token marker first, to see if this is new (dump old data) or old (no dump yet)
        index = match tok_iter.next()
        {
            Some(tok)=> tok.parse::<i32>().unwrap_or(-1),
            None=>{
                eprintln!("Cannot process {} as integer. Skipping entry: '{}' -- did you use kda-stretch?",index,line);
                continue
            }, // This is the condition in which we've encountered an empty line, or one which was not correctly formatted so skip it
        };

        if last_index != index && linebuf.len()>0 {
            //this is the  condition in which we've encounterd a *new* "frame", so we can dump the last one
            //make sure I haven't put a bug in the logic:
            assert!(linebuf.len() > 0);
            writeln!(stdout(),"{}",linebuf).unwrap_or(());
            //Now, note we've logged another match
            match_count += 1;
            //copy index over
            last_index = index;
            //clear match-based counters
            kc=0;dc=0;ac=0;bc=0;
        }
        //Now, let's read in the k,v pairs for the continuing (or recently initialized) frame
        //Clear line buffer so we can repopulate. This could be done with just carrying forward integers and changing
        //the wrineln! to process_and_write(those integers) ... but nah.
        linebuf=String::new();
        //Now, we grab anything related to this line and tally up
        if let Some(key) = tok_iter.next() {
            // NBD if we don't have a value
            let val = tok_iter.next().unwrap_or("");
            //the presence of a key is an implicit +1 (undocumented "feature")
            let val_int = val.parse::<i32>().unwrap_or(1);
            //Get the keys we care about. I should really warn people about entries like 0 K K, which counts as 1 kill. 
            match key
            {                
                "K"=>{ kc+=val_int; kills    +=val_int as f32; },
                "D"=>{ dc+=val_int; deaths   +=val_int as f32; },
                "A"=>{ ac+=val_int; assists  +=val_int as f32; },
                "B"=>{ bc+=val_int; bounties +=val_int as f32; },
                "date"|"Date"=>{date_string = val.to_string().clone();},
                _ => (),
            }
            //calc relevant stats up till now
            let gkda:f32;
            let lkda:f32;
            if deaths == 0.0{
                gkda = kills +assists ;
            } else {
                gkda = (kills +assists )/deaths;
            }
            if dc== 0{
                lkda = kc as f32 +ac as f32;
            } else {
                lkda = (kc as f32 + ac as f32)/ dc as f32
            }
            let bpm = bounties/match_count as f32;
            let apm = assists/match_count as f32;
            let kpm = kills/match_count as f32;
            let dpm = deaths/match_count as f32;

            //create the buffer
            linebuf+=&std::format!("{:>5} ",match_count);
            linebuf+=&std::format!("{:>10} ",date_string);

            linebuf+=&std::format!("{:>3} ",kc);
            linebuf+=&std::format!("{:>3} ",dc);
            linebuf+=&std::format!("{:>3} ",ac);
            linebuf+=&std::format!("{:>3} ",bc);
            linebuf+=&std::format!("{:>5} ",std::format!("{:0.2}",lkda));

            linebuf+=&std::format!("{:>5} ",std::format!("{:2.0}",kills));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.0}",deaths));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.0}",assists));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.0}",bounties));

            linebuf+=&std::format!("{:>5} ",std::format!("{:2.2}",gkda));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.2}",kpm));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.2}",dpm));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.2}",apm));
            linebuf+=&std::format!("{:>5} ",std::format!("{:2.2}",bpm));

        }
    }
    //spit out the last line
    writeln!(stdout(),"{}",linebuf).unwrap_or(());
}
