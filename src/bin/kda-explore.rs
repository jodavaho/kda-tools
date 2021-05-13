
extern crate clap;
use poisson_rate_test::two_tailed_rates_equal;
use clap::{App,Arg};
use std::collections::{HashMap,HashSet};
use std::io::{Write,stdin,stdout,BufRead};
use std::string::String;

#[derive(Debug)]
struct ResultRecord{
    metric_name:String,
    variable_groupings:String,
    p_val:f64,
    n_with:usize,
    n_without:usize,
    sum_with:f64,
    sum_without:f64,
}
/**
 * Takes a long sequence of kda stats (see kda-stretch), and computes correlations
 */
fn main() -> Result<(),String> {

    let input_args = App::new("kda-explore")
        .version( &kda_tools::version()[..] )
        .author("Joshua Vander Hook <josh@vanderhook.info>")
        .about(
            &
                (kda_tools::about()
                +"\n This tool allows manual exploration of metrics with and without a given loadout. For automated exploration, see kda-compare"
                ) [..]
            )
        .arg(Arg::with_name("command")
        .required(true)
        .default_value("K D A : all")
        .help("The A/B comparison to run, of the form '<some variables : <other variables>'. e.g., 'K: pistol' will check kills with and wtihout pitols")
        )
        .arg(Arg::with_name("out_format")
        .required(false)
        .default_value("wsv")
        .help("Output format. Currently supported:
         - wsv: Whitespace Seperated Values (default)
         - tsv: Tab Seperated Values
         - csv: Comma Seperated Values
         - html: HTML table
         - vnl: Vnlog"
            )
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

    //but skip the keyword fields for this (we just want items/ variables)
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
        if grouping == "all"{
            //create singular groupings from names.
            //comparisons.push(vec![]);
            'namecheck:for n in &names{
                for (possible_keyword,_) in kvc::get_reserved_matchers()
                {
                    if possible_keyword==*n{
                        continue 'namecheck;
                    }
                }
                //nope, not a kvc keyword
                //comparisons.last_mut().unwrap().push(n.clone());
                comparisons.push( vec![n.clone()] ) ;
            }
        } else {
            //it's a string that's a list of items
            comparisons.push( grouping.split_whitespace().map(|x| x.to_string()).collect());
        }
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
        let mut records = Vec::<ResultRecord>::new();
        //these are "rows"
        //get the metrics / match
        let metric_idx = idx_lookup.get(metric).unwrap();

        for grouping in comparisons.iter(){
            //initialize the metric "return value", which is a list of values we compare against
            let mut grouping_name = "".to_string();
            //calculate metrics for this grouping, starting with "all" and downselecting
            let mut grouping_occurances : HashSet<_> = (0..num_matches).collect();
            for item in grouping{
                grouping_name+=&(item.to_string()+" ");
                if cfg!(debug_assertions){
                    eprintln!("Debug: Checking: {}",item);
                }

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
            }
            grouping_name=grouping_name.trim().replace(" ","+");
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
            if n_group <1 {
                eprintln!(
                    "No matches found with grouping '{}', this test is useless. Skipping!",grouping_name.to_string()
                );
                continue;
            }
            if n_metric_non_group <1 {
                eprintln!(
                    "No matches found without grouping '{}', this test is useless. Skipping!",grouping_name.to_string()
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

            let p_val = two_tailed_rates_equal(
                sum_metric_group, n_group as f64,
                sum_metric_non_group, n_non_group as f64
            );

            records.push(
                ResultRecord{
                    metric_name:metric.clone(),
                    variable_groupings:grouping_name.clone(),
                    p_val:1.0-p_val,
                    n_with:n_group,
                    n_without:n_non_group,
                    sum_with:sum_metric_group,
                    sum_without:sum_metric_non_group
                }
            );

            if cfg!(debug_assertions){
                eprintln!("Processed: {}",grouping_name);
            }
        }

        for mut r in records.iter_mut(){
            if r.n_with==0{
                //no matches with variable. Comparison meaningless. 
            } else if r.n_without == 0{
                //no matches without metric. Comparison equals baseline.
            } else if r.sum_with as f32 /(r.n_with as f32) > r.sum_without as f32 /r.n_without as f32 {
                //with has higher rate than without. Let's sort specially as positive p value for display.
                r.p_val=r.p_val.abs();
            } else if r.sum_with as f32 /(r.n_with as f32) < r.sum_without as f32 /r.n_without as f32 {
                //with has lower rate than without. let's sort specially as negative for display
                r.p_val=-r.p_val.abs();
            } else {
               //what do we do when equal??? It'll never happen aahahahahahhaaahahaah 
            }
        }

        //number crunching done. Let's display
        //now sort
        records.sort_by(|a,b| a.p_val.partial_cmp(&b.p_val).unwrap());

        //then align

        let mut output_string = String::new();
        output_string += &std::format!("# {} {:<20} ","metric","grp");
        output_string += &std::format!("{:<6} {:<6} {:<6}","n_w","M_w","r_w");
        output_string += &std::format!("{:<6} {:<6} {:<6}","n","M","r");
        output_string += &std::format!("{:<6}","1-p[n_w|r]");
        writeln!(stdout(), "{}", output_string).unwrap_or(());
        for r in records{
            let obs_rate_group = r.sum_with as f32 / r.n_with as f32;
            let mut output_string = String::new();
            output_string += &std::format!( " {} {:<20} ", metric,r.variable_groupings);
            output_string += &std::format!(" {:<2.2}  {:<2.2}  {:<2.2} ",r.sum_with as i32,r.n_with ,obs_rate_group);
            if r.n_without >0
            {
                let obs_rate_non_group = r.sum_without as f32 / r.n_without as f32;
                output_string += &std::format!(" {:<2.2}  {:<2.2}  {:<2.2} ",r.sum_without as i32,r.n_without,obs_rate_non_group);
                output_string += &std::format!(" {:<2.2}",r.p_val.abs());
            } else {
                output_string += &std::format!(" {:<2.2}  {:<2.2}  {:<2.2} ",0,0,"undef");
                output_string += &std::format!(" {:<2.2}","-");
            }
            writeln!(stdout(), "{}", output_string).unwrap_or(());

        }
    }

    Ok(())
}
