extern crate clap;
extern crate pbr;
use pbr::{ProgressBar};
//use poisson_rate_test::two_tailed_rates_equal;
use poisson_rate_test::bootstrap::param::ratio_events_equal_pval_n;
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
    metric_with:f64,
    metric_without:f64,
}

fn align_output(outs: &Vec<String>,widths:&Vec<usize>,seperator:&str)
-> String{
    let len:usize=widths.iter().sum();
    let final_len = len + outs.len()-1;//<-seperators
    let mut ret = String::with_capacity(final_len);
    for i in 0..outs.len(){
        let wi=widths[i];
        let s = &outs[i];
        let slen = s.len().min(wi);
        ret.push_str(&s[0..slen]);
        if i<outs.len()-1{
            ret.push_str(&seperator[..]);
        }
        for _ in slen..wi{
            ret.push(' ');
        }
    }
    ret
}
fn main() -> Result<(),String> {

    let input_args = App::new("kda-compare")
        .version( &kda_tools::version()[..] )
        .author("Joshua Vander Hook <josh@vanderhook.info>")
        .about(
            & (kda_tools::about()
            +
            "\nThis tool compares the value of KDA across item groupings automatically, and shows kda expected values, spreads, and likelihood ratio test results for all groupings.\
            \n\nIf you'd like to do your own comparisons, please use kda-explore
            \n\nIt *expects* input in kvc format (one match per line), and processs the variables K, D, and A, as a function of *all other* variables present. It ignores kvc keywords / fields (like dates), but you'll have to specify other things to ignore manually.
            \n\nTHIS TOOL IS VERY EXPERIMENTAL, nothing is expected to work.
            "
            ) [..]
        )
        .arg(Arg::with_name("fast")
        .help("Speed up computation by doing a fewer number of iterations. Helpful for quick looks but the ordering of some sets may change across multiple invocations")
        .required(false)
        .takes_value(false)
        .short("f")
        )
        /*
        .arg(Arg::with_name("ignore")
        .help("List of fields to ignore (if they appear in data). You can ignoring fields A B and C as '-i A,B,C' or '-i A -i B -i C' but not '-i A B C' or '-i A, B, C'. That's because of shell magic, not becuase of the way it was implemented")
        .short("i")
        .multiple(false)
        .required(false)
        )*/
        .arg(Arg::with_name("out_format")
        .required(false)
        .short("o")
        .default_value("wsv")
        .possible_values(
            &["wsv", "tsv", "csv", "vnl"]
        )
        .help("Output format which can be one of Vnlog or Whitespace-,  Tab-, or Comma-seperated."
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

    let go_faster = input_args.is_present("fast");
    
    //create name->idx lookup table
    let mut idx_lookup:HashMap<String,usize> = HashMap::new();

    for idx in  0..names.len() {
        idx_lookup.insert(names[idx].to_string(),idx);
    }
    eprintln!();

    let k_idx = *idx_lookup.get("K").unwrap_or(&usize::MAX);
    let d_idx = *idx_lookup.get("D").unwrap_or(&usize::MAX);
    let a_idx = *idx_lookup.get("A").unwrap_or(&usize::MAX);
    let b_idx = *idx_lookup.get("B").unwrap_or(&usize::MAX);
    //create two new metrics, pvp = (K+A)/D, and pve = B/D

    let groups = names.clone();

    //verify all output variables
    let mut pvp_records = Vec::<ResultRecord>::new();
    let mut pve_records = Vec::<ResultRecord>::new();
    //these are "rows"

    let group_count = groups.len();
    let mut process_bar = ProgressBar::new(group_count as u64);
    'nextgrp:  for grouping in groups
    {
        process_bar.inc();
        //initialize the metric "return value", which is a list of values we compare against
        let mut grouping_name = "".to_string();
        //calculate metrics for this grouping, starting with "all" and downselecting
        let mut grouping_occurances : HashSet<_> = (0..num_matches).collect();
        // KLUDGE KLUDGE KLUDGE
        let names = vec![grouping.clone()];
        // ^^ Fix that
        for item in names{
            match &item[..]{
                "K"|"D"|"A"|"B"|"BK"|"Date"=>continue 'nextgrp,
                _=>{},
            }
            grouping_name+=&(item.to_string()+" ");
            if cfg!(debug_assertions){
                eprintln!("Debug: Checking: {}",item);
            }

            assert!(idx_lookup.contains_key(&item),"Could not find {} in input data!",item);
            if cfg!(debug_assertinos){
                eprintln!("Debug: Fetching {} by name",item);
            }
            let data_idx = *idx_lookup.get(&item).unwrap();
            //for which matches did that item appear?
            let item_occurances = data.iter()
                //filter first by idx matching the one in question
                .filter(| ((_,idx),_) |  *idx==data_idx )
                //and return only those rows
                .map(| ((time,_),_) | *time ).collect::<HashSet<usize>>();
            //use intersetino for AND relationship
            grouping_occurances.retain(|x| item_occurances.contains(x));
        }
        grouping_name = grouping_name.trim().replace(" ","+");
        let all_matches = (0..num_matches).collect::<HashSet<_>>();
        let grouping_non_occurances = all_matches.symmetric_difference(&grouping_occurances).collect::<HashSet<_>>();

        //now we have a grouping for which to request data later. 
        //what about zeros ... times when metric did not occur but grouping did? The rest of those are just diff in len

        let k_with_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_occurances.contains(match_number) &&  *variable_idx == k_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let k_without_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_non_occurances.contains(match_number) &&  *variable_idx == k_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let d_with_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_occurances.contains(match_number) &&  *variable_idx == d_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let d_without_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_non_occurances.contains(match_number) &&  *variable_idx == d_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let a_with_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_occurances.contains(match_number) &&  *variable_idx == a_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let a_without_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_non_occurances.contains(match_number) &&  *variable_idx == a_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let b_with_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_occurances.contains(match_number) &&  *variable_idx == b_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();
        let b_without_grp = data.iter()
                                //filter first by idx matching the one in question
                                .filter(| ((match_number,variable_idx),_) |  grouping_non_occurances.contains(match_number) &&  *variable_idx == b_idx)
                                //and return only those rows and values
                                .map(| ((_,_),v) | v.parse::<f64>().unwrap() ).collect::<Vec<_>>();

        
        //now we have everything to do a with/without comparison for this grouping
        let n_group = grouping_occurances.len();
        if n_group <1 {
            eprintln!(
                "No matches found with grouping '{}', this test is useless. Skipping!",grouping_name.to_string()
            );
            continue;
        }
        let n_non_group = grouping_non_occurances.len();
        if n_non_group == 0 && cfg!(debug_assertions){
            eprintln!("No matches without grouping, cannot do A/B comparisons");
        }
        debug_assert!(n_group + n_non_group == num_matches);
        
        let n_k_group = k_with_grp.len();
        let n_k_non_group = k_without_grp.len();
        debug_assert!(n_group >= n_k_group,"Bug: Got more metric entries than grouping entries");
        debug_assert!(n_non_group >= n_k_non_group,"Bug: Got more metric entries than grouping entries");

        let n_d_group = d_with_grp.len();
        let n_d_non_group = d_without_grp.len();
        debug_assert!(n_group >= n_d_group,"Bug: Got more metric entries than grouping entries");
        debug_assert!(n_non_group >= n_d_non_group,"Bug: Got more metric entries than grouping entries");
        
        let n_a_group = a_with_grp.len();
        let n_a_non_group = a_without_grp.len();
        debug_assert!(n_group >= n_a_group,"Bug: Got more metric entries than grouping entries");
        debug_assert!(n_non_group >= n_a_non_group,"Bug: Got more metric entries than grouping entries");
        
        let n_b_group = b_with_grp.len();
        let n_b_non_group = b_without_grp.len();
        debug_assert!(n_group >= n_b_group,"Bug: Got more metric entries than grouping entries");
        debug_assert!(n_non_group >= n_b_non_group,"Bug: Got more metric entries than grouping entries");

        //tally up the key values of K, D, A, and B for with and without this item group
        let ka_group:usize= (k_with_grp.iter().sum::<f64>() + a_with_grp.iter().sum::<f64>()) as usize;
        let d_group:usize = d_with_grp.iter().sum::<f64>() as usize;
        let kda_group:f64 = ka_group as f64 / d_group as f64 ;
        let b_group:usize = b_with_grp.iter().sum::<f64>() as usize;
        let bd_group:f64 = b_group as f64 / d_group as f64;

        let ka_non_group:usize = (k_without_grp.iter().sum::<f64>() + a_without_grp.iter().sum::<f64>()) as usize;
        let d_non_group:usize = d_without_grp.iter().sum::<f64>() as usize;
        let kda_non_group:f64 = ka_non_group as f64 / d_non_group as f64 ;
        let b_non_group:usize = b_without_grp.iter().sum::<f64>() as usize;
        let bd_non_group:f64 = b_non_group as f64 / d_non_group as f64;
       
        if cfg!(debug_assertions){
            eprintln!("Debug: Processing: {}",grouping_name);
        }
        let num_samples = match go_faster{
            true=>250,
            false=>2500,
        };
        let pvp_val_improved = ratio_events_equal_pval_n(
            ka_group,
            d_group,
            n_group,
            ka_non_group,
            d_non_group,
            n_non_group,
            num_samples
        );
        let pve_val_improved = ratio_events_equal_pval_n(
            b_group,
            d_group,
            n_group,
            b_non_group,
            d_non_group,
            n_non_group,
            num_samples
        );
        if pvp_val_improved.is_ok(){
            pvp_records.push(
                ResultRecord{
                    metric_name:"kda".to_string(),
                    variable_groupings:grouping_name.clone(),
                    p_val:pvp_val_improved.unwrap(),
                    n_with:n_group,
                    n_without:n_non_group,
                    metric_with: kda_group,
                    metric_without:kda_non_group,
                }
            );
        }
        if pve_val_improved.is_ok(){
            pve_records.push(
                ResultRecord{
                    metric_name:"b/d".to_string(),
                    variable_groupings:grouping_name.clone(),
                    p_val:pve_val_improved.unwrap(),
                    n_with:n_group,
                    n_without:n_non_group,
                    metric_with: bd_group,
                    metric_without:bd_non_group,
                }
            );
        }

        if cfg!(debug_assertions){
            eprintln!("Debug: Processed: {}",grouping_name);
        }
    }
    process_bar.finish();

    //number crunching done. Let's display NOTE REV COMPARE
    pvp_records.sort_by(|a,b| b.p_val.partial_cmp(&a.p_val).unwrap());
    pve_records.sort_by(|a,b| b.p_val.partial_cmp(&a.p_val).unwrap());

    //now do some very basic alignment
    let mut max_grp_len:usize=0;
     for r in pvp_records.iter(){
         max_grp_len = r.variable_groupings.len().max(max_grp_len);
     }
     for r in pve_records.iter(){
         max_grp_len = r.variable_groupings.len().max(max_grp_len);
     }

     let header_start =match input_args.value_of("out_format").unwrap_or("wsv"){
         "wsv"=>"",
         "tsv"=>"",
         "csv"=>"",
         "vnl"=>"# ",
        _=>panic!("Unrecognized value of out_format"),
     };
     let seperator=match input_args.value_of("out_format").unwrap_or("wsv"){
         "wsv"=>" ",
         "tsv"=>"\t",
         "csv"=>",",
         "vnl"=>" ",
        _=>panic!("Unrecognized value of out_format"),
     };
    let header=vec![
     "met".to_string(),
     "grp".to_string(),
     "val".to_string(),
     "N".to_string(),
     "~val".to_string(),
     "M".to_string(),
     "p".to_string(),
    ];
    let lengths = vec![
     6,max_grp_len+1,5,5,5,5,5
    ];
    assert!(lengths.len()==header.len());
    let output_string = align_output(&header, &lengths, &seperator[..]);
    writeln!(stdout(), "{}{}", header_start,output_string).unwrap_or(());
  
    //print em all
    let mut all_records = Vec::<ResultRecord>::with_capacity(pve_records.len() + pvp_records.len());
    all_records.append(&mut pvp_records);
    all_records.append(&mut pve_records);
    for r in all_records{

        let mut row = vec![
            r.metric_name.clone(), 
            r.variable_groupings.clone(),
            std::format!("{:2.2}",r.metric_with),
            std::format!("{:2.2}",r.n_with),
        ];
        if r.n_without >0
        {
            row.push(std::format!("{:2.2}",r.metric_without));
            row.push(std::format!("{:2.2}",r.n_without));
            row.push(std::format!("{:2.2}",r.p_val));
        } else {
            row.push("-".to_string());
            row.push("0".to_string());
            row.push("-".to_string());
        }
        let output_string = align_output(&row, &lengths, &seperator[..]);
        writeln!(stdout(), "{}", output_string).unwrap_or(());

    }

    Ok(())
}
