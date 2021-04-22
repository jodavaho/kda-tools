
extern crate clap;
use poisson_rate_test::poisson_lhr_test;
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
        .about(&kda_tools::about()[..])
        .arg(Arg::with_name("command")
        .required(true)
        .default_value("K D A : _")
        .help("The A/B comparison to run, of the form '<some variables : <other variables>'. e.g., 'K: pistol' will check kills with and wtihout pitols")
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
    eprintln!("Varibables found:");
    for idx in  0..names.len() {
        eprint!("{} ",&names[idx]);
        idx_lookup.insert(names[idx].to_string(),idx);
    }
    eprintln!();

    let command = input_args.value_of("command").unwrap();
    eprintln!("Debug: processing: {}",command);
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
            if cfg!(debug_assertions){
                eprintln!("Received input: {}",item);
            }
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
                        if cfg!(debug_assertinos){
                            eprintln!("Fetching: '*' as 'all individual inputs'");
                        }
                        //there's one special case, "*" which means all-to-all compare,
                        //which is weird and probably should be handled seperately ... it means n *rows* not n *columns* 
                    },
                    "_"=> {
                        if cfg!(debug_assertinos){
                            eprintln!("Fetching: '_' as 'baseline'");
                        }
                        //compare vs all data "baseline"
                        //relevent_matches.push( (0..num_matches).collect::<Vec<_>>() );
                        //split_names.push("Baseline".to_string());
                    },
                    _=>{
                        //user specifically requested some data by name as part of a multi-item grouping
                        //what column of the data corresponds to the  interesting one?
                        assert!(idx_lookup.contains_key(item),"Could not find {} in input data!",item);
                        if cfg!(debug_assertinos){
                            eprintln!("Debug: Fetching {} by name",item);
                        }
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

            let metric_values_with_group = data.iter()
                                    //filter first by idx matching the one in question
                                    .filter(| ((match_number,variable_idx),_) |  grouping_occurances.contains(match_number) && variable_idx == metric_idx)
                                    //and return only those rows and values
                                    .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
            let metric_values_without_group = data.iter()
                                    //filter first by idx matching the one in question
                                    .filter(| ((match_number,variable_idx),_) |  grouping_non_occurances.contains(match_number) && variable_idx == metric_idx )
                                    //and return only those rows and values
                                    .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();

            
            // -- Let's just do CI testing for now

            //now we have everything to do a with/without comparison for this grouping
            let n_group = grouping_occurances.len();
            let n_non_group = grouping_non_occurances.len();
            let n_metric_group = metric_values_with_group.len();
            let n_metric_non_group = metric_values_without_group.len();
            debug_assert!(n_group + n_non_group == num_matches);
            debug_assert!(n_group >= n_metric_group,"Bug: Got more metric entries than grouping entries");
            debug_assert!(n_non_group >= n_metric_non_group,"Bug: Got more metric entries than grouping entries");
            if cfg!(debug_assertions){
                eprintln!("Debug: N with grouping: {}",n_group);
                eprintln!("Debug: N w/o grouping: {}",n_non_group);
                eprintln!("Debug: matches with at least a metric value & grouping: {}",n_metric_group);
                eprintln!("Debug: matches with at least a metric value & w/o group: {}",n_metric_non_group);
            }

            //we may have missed some matches for which the metric never occured (no kills is common in matches)
            //for those, assume zeros, so we just need to know *how many* zeros to pad
            let n_zeros_group = n_group - n_metric_group;
            let n_zeros_non_group  = n_non_group - n_metric_non_group;
            debug_assert!(n_zeros_group + n_metric_group == n_group);
            debug_assert!(n_zeros_non_group + n_metric_non_group == n_non_group);
            if cfg!(debug_assertions){
                eprintln!("Debug: Assumed zeros for matches w/grouping: {}",n_zeros_group);
                eprintln!("Debug: Assumed zeros for matches w/o grouping: {}",n_zeros_non_group);
            }

            let sum_metric_group:f64 = metric_values_with_group.iter().sum();
            let sum_metric_non_group:f64 = metric_values_without_group.iter().sum();
            let obs_rate_group = sum_metric_group / n_group as f64;
            let obs_rate_non_group = sum_metric_non_group / n_non_group as f64;
            if n_group <1 {
                eprintln!(
                    "No matches found with grouping '{}', this test is useless. Skipping!",grouping_name.to_string()
                );
                continue;
            }
            if sum_metric_group == 0.0 && cfg!(debug_assertions){
                eprintln!("No matches with grouping and metric, reduced to p(0|M) ");
            }
            if n_non_group == 0 && cfg!(debug_assertions){
                eprintln!("No matches without grouping, cannot do A/B comparisons");
            }
            //Note, these debugs are commented out because they are not fail-fast conditions any more. 
            //debug_assert!(sum_metric_non_group>0.0);
            //debug_assert!(obs_rate_group>0.0);
            //debug_assert!(obs_rate_non_group>0.0);

            let p_val = poisson_lhr_test(
                sum_metric_group, n_group,
                sum_metric_non_group, n_non_group,
                1.0 /*Test for equivalent rates*/
            );
            //specific case of probability 0 | non-group rate and only n_group trials
            let mut output_string = String::new();
            output_string += &grouping_name;
            output_string += " ";
            output_string += &std::format!("{:0.2}/{:0.2} = {:0.2} ",sum_metric_group,n_group ,obs_rate_group);
            if n_non_group>0
            {
                output_string += &" vs ";
                output_string += &std::format!("{:0.2}/{:0.2} = {:0.2} ",sum_metric_non_group,n_non_group,obs_rate_non_group);
                output_string += &std::format!("Rates are same with p={:0.3}",p_val);
            } else {
                output_string += " all matches contain grouping "
            }
            writeln!(stdout(), "{}", output_string).unwrap_or(());


        }
    }

    Ok(())
}
