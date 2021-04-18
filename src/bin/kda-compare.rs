
extern crate clap;
use statrs::distribution::Discrete;
use clap::{App,Arg};
use std::collections::{HashMap,HashSet};
use std::io::{Write,stdin,stdout,BufRead};
use std::string::String;
use statrs::distribution::{Poisson, ChiSquared};
use statrs::function::gamma::{gamma_li, gamma};


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
            debug_assert!(sum_metric_group>0.0);
            debug_assert!(sum_metric_non_group>0.0);
            debug_assert!(obs_rate_group>0.0);
            debug_assert!(obs_rate_non_group>0.0);
            if cfg!(debug_assertions){
                eprintln!("Debug: Observed sum with grouping: {}, w/o: {}",sum_metric_group,sum_metric_non_group);
                eprintln!("Debug: Observed rate with grouping: {}, w/o: {} ",obs_rate_group,obs_rate_non_group);
            }

            //by Gu 2008 Testing Ratio of Two Poisson Rates
            //use magic factor R/d, w/ d=t0/t1 (the # trials, I guess ... )
            //so r0 is "rate of kills with gear" and r1 is "rate of kills w/o gear"
            // so t0 is # of games w/ gear, and t1 is #games without gear
            let h0_rate_ratio = 1.0; // Check for equality of rates under null hypothesis
            //calculate the expected rates under the null hypothesis
            //Generally, here, using the funny constants, but in the case R=1 and t1=t0, it's simpler.
            //magic constant #1, d = t1/t0
            let magic_d = n_non_group as f64 / n_group as f64;
            //magic constant #2, g = R/d
            let magic_g = h0_rate_ratio as f64 / magic_d as f64;
            //now using magic constants, calculate the "hypothesis constarained rates"
            //i.e., the expected rates given H0 is true
            if cfg!(debug_assertions){
                eprintln!("magic d: {} and g: {}", magic_d, magic_g);
            }
            let hypothesized_rate_group = (sum_metric_group + sum_metric_non_group) as f64/ (n_group as f64 * (1.0+1.0/magic_g));
            let hypothesized_rate_non_group = (sum_metric_group + sum_metric_non_group) as f64 / (n_non_group as f64 * (1.0+magic_g));
            debug_assert!(hypothesized_rate_group>0.0);
            debug_assert!(hypothesized_rate_non_group>0.0);
            if cfg!(debug_assertions){
                eprintln!("hyp rate w/ : {}, hyp rate w/o : {}",hypothesized_rate_group ,hypothesized_rate_non_group );
                eprintln!("metric w/ : {},  metric w/o : {}",sum_metric_group,sum_metric_non_group);
            }
            //OK so if you follow through magic_g, under the case R=1, t0 == t1, it all cancels out nicely. 
            let maximum_likelihood_h0:f64 = 
                Poisson::new(hypothesized_rate_group * n_group  as f64).unwrap().pmf(sum_metric_group as u64)
                * Poisson::new(hypothesized_rate_non_group * n_non_group as f64).unwrap().pmf(sum_metric_non_group as u64);
            let maximum_likelihood_unconstrained:f64 = 
                Poisson::new(obs_rate_group * n_group as f64).unwrap().pmf(sum_metric_group as u64)
                * Poisson::new(obs_rate_non_group * n_non_group as f64).unwrap().pmf(sum_metric_non_group as u64);
            let test_statistic:f64 = -2.0*(maximum_likelihood_h0 / maximum_likelihood_unconstrained).ln();

            //of course this doesn't work:
            //let p_val = .5* ( 1-ChiSquared::new(1.0).unwrap().checked_inverse_cdf(test_statistic) );
            let p_val = 0.5 * (1.0- gamma_li(0.5,test_statistic as f64) / gamma(0.5) );

            let mut output_string = String::new();
            output_string += &grouping_name;
            output_string += " ";
            output_string += &std::format!("{:0.2}/{:0.2} = {:0.2} ",sum_metric_group,n_group ,obs_rate_group);
            output_string += &" vs ";
            output_string += &std::format!("{:0.2}/{:0.2} = {:0.2} ",sum_metric_non_group,n_non_group,obs_rate_non_group);
            output_string += &std::format!("Rates are same with p={:0.3}",p_val);
            writeln!(stdout(), "{}", output_string).unwrap_or(());
        }
    }

    Ok(())
}
