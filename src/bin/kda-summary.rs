//use std::collections::HashMap;
use std::collections::HashMap;
use std::io::Write;
use std::io::{stdin,stdout};
use std::io::BufRead;
//use std::fs::File;
//use std::path::Path;
use std::string::String;

fn main() {
    //parse the i/o "item" count
    let mut index = 0;

    //human-readable N
    let mut kills = 0.0;
    let mut deaths = 0.0;
    let mut assists = 0.0;
    let mut bounties = 0.0;

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
        //per-match counters:
        let mut kc= 0.0;
        let mut dc= 0.0;
        let mut ac= 0.0;
        let mut bc= 0.0;

        let line:String = match input_line
        {
            Ok(line)=>line,
            Err(_)=>break, //this is a condition similar to 'there are no more lines to process'
            //At this point, we should be dumping the last data, which happens at the end.
        };

        let (keycounts,reserved_pairs)=kvc::read_kvc_line_default(&line);
        if keycounts.len()==0 && reserved_pairs.len()==0 {
            //these are not the lines we're looking for
            continue;
        }
        index+=1;
        let mut reserved_lookup: HashMap<String,String>  = HashMap::new();
        for (key,value) in reserved_pairs{
            reserved_lookup.insert(key,value);
        }
        let date_string = match reserved_lookup.get("Date")
        {
            None=>index.to_string(),
            Some(s)=>s.clone(),
        };
        //Now, let's read in the k,v pairs for the continuing (or recently initialized) frame
        //Clear line buffer so we can repopulate. This could be done with just carrying forward integers and changing
        //the wrineln! to process_and_write(those integers) ... but nah.
        //Now, we grab anything related to this line and tally up
        for (key,counts) in keycounts{

            match &key[..]
            {                
                "K"=>{ kc+=counts; kills    +=counts; },
                "D"=>{ dc+=counts; deaths   +=counts; },
                "A"=>{ ac+=counts; assists  +=counts; },
                "B"=>{ bc+=counts; bounties +=counts; },
                _ => (),
            }

        }
        //calc relevant stats up till now
        let gkda:f32;
        let lkda:f32;
        if deaths == 0.0{
            gkda = kills +assists ;
        } else {
            gkda = (kills +assists )/deaths;
        }
        if dc== 0.0{
            lkda = kc as f32 +ac as f32;
        } else {
            lkda = (kc as f32 + ac as f32)/ dc as f32
        }
        let bpm = bounties/index as f32;
        let apm = assists/index as f32;
        let kpm = kills/index as f32;
        let dpm = deaths/index as f32;

        //create the buffer
        let mut linebuf=String::new();
        linebuf+=&std::format!("{:>5} ",index);
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
        writeln!(stdout(),"{}",linebuf).unwrap_or(());
    }
}
