//use std::collections::HashMap;
use std::io::Write;
use std::io::{stdin,stdout};
use std::io::BufRead;
//use std::fs::File;
//use std::path::Path;
use std::string::String;

fn main() {
    let mut index = 0;
    let mut last_index = 0;
    let mut match_count = 0;
    let mut date_string = "";
    
    let mut kills = 0.0;
    let mut deaths = 0.0;
    let mut assists = 0.0;
    let mut bounties = 0.0;
    let mut kc= 0;
    let mut dc= 0;
    let mut ac= 0;
    let mut bc= 0;


    let mut header = "".to_string();

    header+=&std::format!("{:>5} ","n");
    header+=&std::format!("{:>10}  ","Date");

    header+=&std::format!("{:>3} ","K");
    header+=&std::format!("{:>3} ","D");
    header+=&std::format!("{:>3} ","A");
    header+=&std::format!("{:>3} ","B");
    header+=&std::format!("{:>3} ","KDA");

    header+=&std::format!("{:>5} ","sK");
    header+=&std::format!("{:>5} ","sD");
    header+=&std::format!("{:>5} ","sA");
    header+=&std::format!("{:>5} ","sB");

    header+=&std::format!("{:>5} ","mK");
    header+=&std::format!("{:>5} ","mD");
    header+=&std::format!("{:>5} ","mA");
    header+=&std::format!("{:>5} ","mB");
    header+=&std::format!("{:>5} ","mKDA");

    header+=&std::format!("{:>3} ","n");

    writeln!(stdout(),"{}",header).unwrap_or(());
    for input_line in stdin().lock().lines()
    {
        let line:String = match input_line
        {
            Ok(line)=>line,
            Err(_)=>"".to_string(),
        };
        let mut tok_iter = line.split_whitespace();
        index = match tok_iter.next()
        {
            Some(tok)=> tok.parse::<i32>().unwrap_or(-1),
            None=>-1,
        };
        if last_index < 0 {
            last_index = index;
        } else if last_index != index || index < 0{
            kills+=kc as f32;
            deaths+=dc as f32;
            assists+=ac as f32;
            bounties+=bc as f32;

            let mut gkda:f32= 0.0;
            let mut lkda:f32= 0.0;

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
            let mut linebuf = "".to_string();

            linebuf+=&std::format!("{:>5} ",match_count);
            linebuf+=&std::format!("{:>10} ","fix-date!");

            linebuf+=&std::format!("{:>3} ",kc);
            linebuf+=&std::format!("{:>3} ",dc);
            linebuf+=&std::format!("{:>3} ",ac);
            linebuf+=&std::format!("{:>3} ",bc);
            linebuf+=&std::format!("{:>5} ",lkda);

            linebuf+=&std::format!("{:>5} ",kills);
            linebuf+=&std::format!("{:>5} ",deaths);
            linebuf+=&std::format!("{:>5} ",assists);
            linebuf+=&std::format!("{:>5} ",bounties);

            linebuf+=&std::format!("{:>5} ",gkda);
            linebuf+=&std::format!("{:>5} ",kpm);
            linebuf+=&std::format!("{:>5} ",dpm);
            linebuf+=&std::format!("{:>5} ",apm);
            linebuf+=&std::format!("{:>5} ",bpm);

            //clear counters
            kc=0;dc=0;ac=0;bc=0;

            writeln!(stdout(),"{}",linebuf).unwrap_or(());

            //Now, note we've logged a match
            match_count += 1;
        }
        //now that we've cleared old data, let's go on to new data
        last_index = index;
        date_string = match tok_iter.next(){
            None=>"",
            Some(s)=>s,
        };
        let item = tok_iter.next().unwrap_or("");
        match item{                
            "K"=>kc+=1,
            "D"=>dc+=1,
            "A"=>ac+=1,
            "B"=>bc+=1,
            _ => (),
        };
    }
}
