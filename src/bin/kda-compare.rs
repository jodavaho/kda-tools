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
fn main() -> Result<(),String> {

    let input_args = App::new("kda-compare")
        .version( &kda_tools::version()[..] )
        .author("Joshua Vander Hook <josh@vanderhook.info>")
        .about(
            & (kda_tools::about()
            +
            "\n\n This tool compares the value of KDA across item groupings automatically, and shows kda expected values, spreads, and likelihood ratio test results for all groupings.\
            \n\n If you'd like to do your own comparisons, please use kda-explore
            \n\n It *expects* input in kvc format (one match per line), and processs the variables K, D, and A, as a function of *all other* variables present. It ignores kvc keywords / fields (like dates), but you'll have to specify other things to ignore manually.
            "
            ) [..]
        ).arg(Arg::with_name("ignore")
        .help("List of fields to ignore (if they appear in data). You can ignoring fields A B and C as '-i A,B,C' or '-i A -i B -i C' but not '-i A B C' or '-i A, B, C'. That's because of shell magic, not becuase of the way it was implemented")
        .short("i")
        .multiple(false)
        .required(false)
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
    let k_idx = names.iter().position(|&s| s=="K").unwrap_or(usize::MAX);
    let d_idx = names.iter().position(|&s| s=="D").unwrap_or(usize::MAX);
    let a_idx = names.iter().position(|&s| s=="A").unwrap_or(usize::MAX);
    let b_idx = names.iter().position(|&s| s=="B").unwrap_or(usize::MAX);
    for idx in  0..names.len() {
        eprint!("{} ",&names[idx]);
        idx_lookup.insert(names[idx].to_string(),idx);
    }
    eprintln!();

    //create two new metrics, pvp = (K+A)/D, and pve = B/D

    let groups = Vec::<String>::new();

    //verify all output variables
    let mut records = Vec::<ResultRecord>::new();
    //these are "rows"
    //get the metrics / match
    let metric_idx = idx_lookup.get(metric).unwrap();

    for grouping in groups{
        //initialize the metric "return value", which is a list of values we compare against
        let mut grouping_name = "".to_string();
        //calculate metrics for this grouping, starting with "all" and downselecting
        let mut grouping_occurances : HashSet<_> = (0..num_matches).collect();
        for item in names{
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
        grouping_name.trim().replace(" ","+");
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
    records.sort_by(|a,b| a.p_val.partial_cmp(&b.p_val).unwrap());

    //now sort

    for r in records{
        let obs_rate_group = r.sum_with as f32 / r.n_with as f32;
        let mut output_string = String::new();
        output_string += &(metric.clone()+&r.variable_groupings);
        output_string += " ";
        output_string += &std::format!("{}/{} = {:0.2} ",r.sum_with as i32,r.n_with ,obs_rate_group);
        if r.n_without >0
        {
            let obs_rate_non_group = r.sum_without as f32 / r.n_without as f32;
            output_string += &" vs ";
            output_string += &std::format!("{}/{} = {:0.2} ",r.sum_without as i32,r.n_without,obs_rate_non_group);
            output_string += &std::format!("Rates are different with p={:0.3}",r.p_val.abs());
        } else {
            output_string += " all matches contain grouping "
        }
        writeln!(stdout(), "{}", output_string).unwrap_or(());

    }

    Ok(())
}
