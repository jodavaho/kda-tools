
extern crate clap;
use clap::{App,Arg};
use std::collections::{HashMap,HashSet};
use std::io::{Write,stdin,stdout,BufRead};
use std::string::String;


/**
 * Takes a long sequence of kda stats (see kda-stretch), and computes correlations
 */
fn main() -> Result<(),String> {

    let input_args = App::new("kda-compare")
        .version( &kda_tools::version()[..] )
        .author("Joshua Vander Hook <josh@vanderhook.info>")
        .about("Some basic conditional probabilities and bayesian analysis on a KVC log. See https://github.com/jodavaho/kda-tools for more info. ")
        .arg(Arg::with_name("kda")
            .long("kda")
            .takes_value(false)
            .help("Include the extra output KDA = (K+A)/D. You'll need to have K, D, and A entries in your log or this will fail loudly.")
        )
        .arg(Arg::with_name("command")
            .short("c")
            .long("command")
            .value_name("COMMAND")
            .takes_value(true)
            .help("Command a comparison like this: 'K (: [<item>] vs [<item>] )' e.g., 'K: pistol vs shotgun' to compare kills with shotguns vs pistols. use '_' to denote 'baseline (avg over all matches)'.")
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
    let inout :Vec<String> = command.split(":").map(|x| x.to_string()).collect();
 
    assert!(inout.len()<=2,"Got more than one ':', cannont process:{}",command);
    assert!(inout.len()>0,"Did not receive a valid command:{}",command);

    let mut comparisons:Vec< Vec<String>> = Vec::new();
    let all_comparisons:Vec<String>= inout[1].split("vs").map(|x| x.trim().to_string()).collect() ;
    for grouping in all_comparisons{
        //it's a string that's a list of items
        comparisons.push( grouping.split_whitespace().map(|x| x.to_string()).collect());
    }

    //verify all input variables
    for grouping in comparisons.iter(){
        //a grouping is a vec of strings
        for item in grouping.iter(){
            eprintln!("Received input: {}",item);
            match &item[..]{
                //don't bother verifying reserved keywords at this stage
                "_"|"*"=>(),
                _=>{
                    assert!( idx_lookup.contains_key(item),std::format!("Variable '{}' not found in input data.",item) );
                },
            }
        }
    }


    //verify all output variables
    let metrics:Vec<String> = inout[0].split_whitespace().map(|x| x.to_string()).collect();
    for metric in metrics.iter(){
        assert!( idx_lookup.contains_key(metric),std::format!("Requested output variable '{}' not found in input data.",metric));
    }


    for metric in metrics.iter(){
        //these are "rows"
        //get the metrics / match
        let metric_idx = idx_lookup.get(metric).unwrap();
        for grouping in comparisons.iter(){
            //initialize the metric "return value", which is a list of values we compare against
            let mut grouping_name = metric.to_string()+":( ";
            //calculate metrics for this grouping, starting with "all" and downselecting
            let mut grouping_occurances : HashSet<_> = (0..num_matches).collect();
            for item in grouping{
                grouping_name+=&(item.to_string()+" ");

                match &item[..]{
                    "*"=> {
                        eprintln!("Fetching: '*' as 'all individual inputs'");
                        //there's one special case, "*" which means all-to-all compare,
                        //which is weird and probably should be handled seperately ... it means n *rows* not n *columns* 
                    },
                    "_"=> {
                        eprintln!("Fetching: '_' as 'baseline'");
                        //compare vs all data "baseline"
                        //relevent_matches.push( (0..num_matches).collect::<Vec<_>>() );
                        //split_names.push("Baseline".to_string());
                    },
                    _=>{
                        //user specifically requested some data by name as part of a multi-item grouping
                        //what column of the data corresponds to the  interesting one?
                        assert!(idx_lookup.contains_key(item),"Could not find {} in input data!",item);
                        eprintln!("Fetching {} by name",item);
                        let data_idx = *idx_lookup.get(item).unwrap();
                        //for which matches did that item appear?
                        let item_occurances = data.iter()
                            //filter first by idx matching the one in question
                            .filter(| ((_,idx),_) |  *idx==data_idx )
                            //and return only those rows
                            .map(| ((time,_),_) | *time ).collect::<HashSet<usize>>();
                        //use intersetino for AND relationship
                        grouping_occurances.retain(|x| item_occurances.contains(x));
                    },
                }; 
            }
            grouping_name+=")";
            let all_matches = (0..num_matches).collect::<HashSet<_>>();
            let grouping_non_occurances = all_matches.symmetric_difference(&grouping_occurances).collect::<HashSet<_>>();

            //now we have a grouping for which to request data later. 
            //what about zeros ... times when metric did not occur but grouping did? The rest of those are just diff in len
            let metric_values = data.iter()
                                    //filter first by idx matching the one in question
                                    .filter(| ((match_number,variable_idx),_) |  grouping_occurances.contains(match_number) && variable_idx == metric_idx)
                                    //and return only those rows and values
                                    .map(| ((_,_),v) | v ).collect::<Vec<_>>();
            let metric_non_values = data.iter()
                                    //filter first by idx matching the one in question
                                    .filter(| ((match_number,variable_idx),_) |  grouping_non_occurances.contains(match_number) && variable_idx == metric_idx )
                                    //and return only those rows and values
                                    .map(| ((_,_),v) | v ).collect::<Vec<_>>();
            //now we have everything to do a with/without comparison for this grouping
            let n_with = grouping_occurances.len();
            let n_without = grouping_non_occurances.len();
            assert!(n_with + n_without == num_matches);
            eprintln!("N with grouping: {}",n_with);
            eprintln!("N w/o grouping: {}",n_without);

            let metric_occured_with_grouping = metric_values.len();
            let metric_occured_without_grouping = metric_non_values.len();
            eprintln!("matches with at least a metric value & grouping: {}",metric_occured_with_grouping);
            eprintln!("matches with at least a metric value & w/o group: {}",metric_occured_without_grouping);
            assert!(n_with >= metric_values.len(),"Bug: Got more metric entries than grouping entries");
            assert!(n_without >= metric_non_values.len(),"Bug: Got more metric entries than grouping entries");

            //we may have missed some matches for which the metric never occured (no kills is common in matches)
            //for those, assume zeros, so we just need to know *how many* zeros to pad
            let n_extra_zeros_with = n_with - metric_values.len();
            let n_extra_zeros_without = n_without - metric_non_values.len();
            eprintln!("Assumed zeros for matches w/grouping: {}",n_extra_zeros_with);
            eprintln!("Assumed zeros for matches w/o grouping: {}",n_extra_zeros_without);

            let mut output_string = String::new();
            writeln!(stdout(), "{}", output_string).unwrap_or(());

        }
    }

    Ok(())
}
