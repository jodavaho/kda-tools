
extern crate clap;
use clap::{App,Arg};
use nalgebra::DVector;
use nalgebra::base::DMatrix;
use std::collections::{HashMap,HashSet};
use std::f32;
use std::io::{Write,stdin,stdout,BufRead};
use std::string::String;


/**
 * Takes a long sequence of kda stats (see kda-stretch), and computes correlations
 */
fn main() -> Result<(),String> {

    let input_args = App::new("kda-compare")
        .version( &kda_tools::version()[..] )
        .author("Joshua Vander Hook <josh@vanderhook.info>")
        .about("Conducts analysis on a KVC log. See https://github.com/jodavaho/kda-tools for more info")
        .arg(Arg::with_name("kda")
            .long("kda")
            .takes_value(false)
            .help("Include the extra output KDA = (K+A)/D. Must include K,D, and A in the input data or this will fail")
        )
        .arg(Arg::with_name("command")
            .short("c")
            .long("command")
            .value_name("COMMAND")
            .takes_value(true)
            .help("Command a comparison like this: 'K (: [<+|-><item>] vs [<+|-><item>]+ )' e.g., 'K: +pistol vs +shotgun' to compare kills with shotguns vs pistols. use '_' to denote 'all items'. DEFAULT='K D A B' for all relevant stats")
        )
        .get_matches();
    let local_sin = stdin();
    let line_itr = local_sin.lock().lines();

    let ( size,data, names ) = kvc::load_table_from_kvc_stream_default(line_itr);
    let ( num_matches, num_vars ) = size;
    eprintln!("Processed. Read: {} rows and {} variables", num_matches,num_vars);
    if num_matches==0{
        return Err("No input data recieved".to_string());
    }
    
    //create name->idx lookup table
    let mut idx_lookup:HashMap<String,usize> = HashMap::new();
    for idx in  0..names.len() {
        eprint!("{} ",&names[idx]);
        idx_lookup.insert(names[idx].to_string(),idx);
    }
    eprintln!();

    let command = input_args.value_of("command").unwrap_or("K");
    eprintln!("processing: {}",command);
    let mut inout :Vec<String> = command.split(":").map(|x| x.to_string()).collect();
    if input_args.is_present("kda"){
        inout.push("K".to_string());
        inout.push("D".to_string());
        inout.push("A".to_string());
    } 
 
    assert!(inout.len()<=2,"Got more than one ':', cannont process:{}",command);
    assert!(inout.len()>0,"Did not receive a valid command:{}",command);

    let mut ins:Vec<String> = Vec::new();
    if inout.len()==2{
       ins= inout[1].split("vs").map(|x| x.trim().to_string()).collect() ;
    } else {
        assert_eq!(ins.len(),0);
    }

    //verify all input variables
    let mut in_idxs:HashSet<usize> = HashSet::new();
    for idx in 0..ins.len(){
        let variable_name:String = ins[idx].clone();
        eprintln!("Received input: {}",variable_name);
        match &variable_name[..]{
            //don't bother verifying reserved keywords at this stage
            "_"|"*"=>(),
            _=>{
                assert!( idx_lookup.contains_key(&variable_name),std::format!("Variable '{}' not found in input data.",ins[idx]));
                in_idxs.insert( *idx_lookup.get(&variable_name).unwrap() );
            },
        }
    }


    //verify all output variables
    let outs:Vec<String> = inout[0].split_whitespace().map(|x| x.to_string()).collect();
    let mut out_idxs:HashSet<usize> = HashSet::new();
    for idx in 0..outs.len(){
        assert!( idx_lookup.contains_key(&outs[idx]),std::format!("Variable '{}' not found in input data.",outs[idx]));
        out_idxs.insert( *idx_lookup.get(&outs[idx]).unwrap() );
    }

    //create a handy data looker-upper
    let data_lookup:HashMap<(usize,usize),String> = data.clone().into_iter().collect();
    if ins.len()==0{
        ins.push( "_".to_string() );
    }

    //create the relevant pairings to iterate over for experiments
    let mut split_list:Vec<Vec<usize>> = Vec::new();
    let mut split_names:Vec<String> = Vec::new();
    for out_experiment in ins{
        match &out_experiment[..]{
            "*"=> {
                eprintln!("Fetching: '*' as 'all individual inputs'");
                //there's one special case, "*" which means all-to-all compare,
                //in which case we can append all inputs as single experiments
                for variable_name in names.clone(){
                    let data_idx = *idx_lookup.get(&variable_name[..]).unwrap();
                    //for which matches did that interesting event occur?
                    let to_insert = data.clone().into_iter()
                        //filter first by idx matching the one in question
                        .filter(| ((_,idx),_) |  *idx==data_idx )
                        //and return only those rows
                        .map(| ((time,_),_) | time ).collect::<Vec<_>>();
                    split_list.push(to_insert);
                    split_names.push(variable_name.clone());
                }
            },
            "_"=> {
                eprintln!("Fetching: '_' as 'baseline'");
                //compare vs all data "baseline"
                split_list.push( (0..num_matches).collect::<Vec<_>>() );
                split_names.push("Baseline".to_string());
            },
            _=>{
                //user specifically requested some data by name.
                //what column of the data corresponds to the  interesting one?
                assert!(idx_lookup.contains_key(&out_experiment[..]),"Could not find {} in input data!",out_experiment);
                eprintln!("Fetching {} by name",out_experiment);
                let data_idx = *idx_lookup.get(&out_experiment[..]).unwrap();
                //for which matches did that interesting event occur?
                let to_insert = data.clone().into_iter()
                    //filter first by idx matching the one in question
                    .filter(| ((_,idx),_) |  *idx==data_idx )
                    //and return only those rows
                    .map(| ((time,_),_) | time ).collect::<Vec<_>>();
                split_list.push(to_insert);
                split_names.push(out_experiment.clone());
            },
        }; 
    }

    write!(stdout(),"{:>10}","Name").unwrap_or(());
    write!(stdout(),"{:>10}","Output").unwrap_or(());
    write!(stdout()," {:>5}","N").unwrap_or(());
    write!(stdout()," {:>5}","E[x|a]").unwrap_or(());
    write!(stdout()," {:>5}","p(x=0)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=1)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=2)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=3)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=4)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=5)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=6)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=7)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=8)").unwrap_or(());
    write!(stdout()," {:>5}","p(x=9)").unwrap_or(());
    writeln!(stdout()).unwrap_or(());
    for split_idx in 0..split_list.len(){
        let relevant_matches = split_list[split_idx].clone();
        let split_name = split_names[split_idx].clone();
        let relevant_len = relevant_matches.len();
        let out_len = out_idxs.len();

        //now let's construct a matrix of all output variables across those matches
        let mut out_m = DMatrix::<f32>::zeros( relevant_len,out_len);
        let mut out_names = Vec::new();
        let mut out_counter = 0;

        for col_idx in 0..num_vars{
            if out_idxs.contains(&col_idx){

                let column = DVector::<f32>::from_fn(
                    relevant_len, |i,_| 
                    match data_lookup.get(&(relevant_matches[i],col_idx))
                    {
                        None=>0.0,
                        Some(x)=> x.parse::<f32>().unwrap_or(0.0),
                    },
                );
                out_m.set_column(out_counter,&column);
                out_names.push( names[col_idx].clone());
                out_counter+=1;
            } 
        }
        //calculate expected # occurances for poisson dist (k events / time period)
        let out_occurances = out_m.row_sum();

        let out_lambda = out_occurances/relevant_len as f32;
        let mut histogram: HashMap<i32,usize> = HashMap::new();


        for idx in 0..out_names.len()
        {
            //baseline / condition 1
            histogram.clear();
            for m in 0..relevant_len{
                let v = out_m.get((m,idx)).unwrap();
                let count = histogram.entry(v.floor() as i32).or_insert(0);
                *count = *count + 1;
            }
            write!(stdout(),"{:>10}",split_name).unwrap_or(());
            write!(stdout(),"{:>10}",out_names[idx]).unwrap_or(());
            write!(stdout(),"{:>5}",relevant_len).unwrap_or(());
            write!(stdout()," {:>5} ",
                std::format!("{:2.2}",out_lambda[idx])).unwrap_or(());
            for x in 0..10{
                let count_x = *histogram.entry(x).or_insert(0);
                write!(stdout()," {:>5} ",
                    std::format!("{:2.2}",count_x as f32 / relevant_len as f32)).unwrap_or(());
            }
            //comparison / condition 2
            writeln!(stdout(),"").unwrap_or(());
        }
    }

    Ok(())
}
