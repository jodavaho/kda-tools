use std::collections::HashMap;
use std::cmp::max;
use std::io::BufRead;
use std::io;
//use std::fs::File;
//use std::path::Path;
//use std::io::BufRead;
use std::string::String;
use nalgebra::base::DMatrix;
use nalgebra::DVector;

/**
 * Takes a long sequence of kda stats (see kda-stretch), and computes correlations
 */
fn main() -> Result<(),String> {

    //helper variables (not used in optimization)
    let mut offset = 0;
    let mut line_count = 0;
    let mut processed_lines =0;

    //optimization varaibles read in
    let mut item_columns: HashMap<String,usize> = HashMap::new();
    let mut row_names: HashMap<usize,String> = HashMap::new();
    let mut data_entries: HashMap<(usize,usize),f32> = HashMap::new();
    
    //size of the state:
    let mut row_max=0;

    //more, the target functions
    let mut b_out: HashMap<usize,f32> = HashMap::new();
    let mut k_out: HashMap<usize,f32> = HashMap::new();
    let mut a_out: HashMap<usize,f32> = HashMap::new();
    let mut d_out: HashMap<usize,f32> = HashMap::new();

    let local_sin = io::stdin();
    let mut line_itr = local_sin.lock().lines();

    k_out.insert(0,0.0);
    d_out.insert(0,0.0);
    a_out.insert(0,0.0);
    b_out.insert(0,0.0);

    while let Some(line_read) = line_itr.next()
    {
        let line = line_read.unwrap_or("".to_string());
        line_count+=1;
        let mut tok_iter = line.split_whitespace();
        let row_txt = match tok_iter.next()
        {
            None=>continue,
            Some(s)=>s,
        };
        let input_row_number = match row_txt.parse::<usize>()
        {
            Ok(i)=>i,
            Err(e)=>{
                eprintln!("Line {} not a row #:{} (see kda-stretch) Error={}",line_count,row_txt,e);
                continue
            },
        };
        if input_row_number==0  && offset==0
        {
            eprintln!("Found zero-indexed row, assuming offset=1");
            offset=1;
        }
        let row_idx = input_row_number -1 + offset; 

        //tally up the "size" of the state ... allowing the user
        //to force us to assume zeros if they skip state indecies
        row_max = max(row_max,input_row_number);

        let key = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>{
                eprintln!("Line {} Cannot process: {}. Expected 3 tokens / line (see kda-stretch).",line_count,line);
                continue;
            },
        };
        let val = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>{
                eprintln!("Line {} Cannot process: {}. Expected 3 tokens / line (see kda-stretch).",line_count,line);
                continue;
            }
        };

        /*
         * We're basically constructing a hashmap that maps element row/column to values. 
         * That is, we're constructing a sparse matrix / vector for each things we're trying to solve. 
         * Later, after I learn how to use DMatrix in ~30 lines, I'll just put it into DMatrix.
         */
         if key=="Date"{
             continue;
         }
         //all other keys have float values
         let fval = match val.parse::<f32>()
         {
             Err(e)=>{
                 eprintln!("Error: Cannot get a float/int value from: {} on line {} -- {}. Skipping",val,line,e);
                 continue;
             },
             Ok(f)=>f,
         };
         match key{
            "K"=>{
                let k_ptr = k_out.entry(row_idx).or_insert(0.0);
                *k_ptr = *k_ptr + fval;
            },
            "D"=>{
                let d_ptr = d_out.entry(row_idx).or_insert(0.0);
                *d_ptr = *d_ptr + fval;
            },
            "A"=>{
                let a_ptr = a_out.entry(row_idx).or_insert(0.0);
                *a_ptr = *a_ptr + fval;
            },
            "B"=>{
                let b_ptr = b_out.entry(row_idx).or_insert(0.0);
                *b_ptr = *b_ptr + fval;
            },
            _ => {
                //process as input variable We're building a matrix F with
                //columns = the state rows retrieve the column name (Or
                //initialize new column), which confusingly correpsonds to the
                //row name. This is cause we assume KDAB = W * X.
                let cur_size:usize = item_columns.len();
                let col_idx = *item_columns.entry(key.to_string()).or_insert(cur_size);

                //save the name for later output
                row_names.insert(col_idx,key.to_string());

                //insert a 1 (or +=1) the row/column. We're doing present / not present.
                let current_value = data_entries.entry((row_idx,col_idx)).or_insert(0.0);
                *current_value=1.0;

                processed_lines+=1;
            }
        }
    }

    let cur_size = item_columns.len();

    eprintln!("Processed {} lines, read: {} rows and {} variables",processed_lines, row_max,cur_size);
    for i in 0..row_names.len(){
        eprint!(" {} ",row_names.entry(i).or_insert("??".to_string()));
    }
    eprintln!("");
    //well this is nice. Might as well rename "from_fn" to "for fun":
    let factor_matrix =DMatrix::<f32>::from_fn(row_max,cur_size, |i,j| *data_entries.entry((i,j)).or_insert(0.0) );

    let x_k =          DVector::<f32>::from_fn(row_max, |i,_| * k_out.entry(i).or_insert(0.0) );
    let x_d =          DVector::<f32>::from_fn(row_max, |i,_| * d_out.entry(i).or_insert(0.0) );
    let x_a =          DVector::<f32>::from_fn(row_max, |i,_| * a_out.entry(i).or_insert(0.0) );
    let x_b =          DVector::<f32>::from_fn(row_max, |i,_| * b_out.entry(i).or_insert(0.0) );
    //ok, do some math to find how much each one contributed to the result (I think)
    let mut x_all = DMatrix::<f32>::zeros(row_max,4);
    x_all.set_column(0,&x_k);
    x_all.set_column(1,&x_d);
    x_all.set_column(2,&x_a);
    x_all.set_column(3,&x_b);

    let kd_spread = x_k.clone() - x_d.clone();
    let kda_spread = (x_k.clone() + x_a.clone()) - x_d.clone();

    // calculate baseline mKDA
    let kdab_sum = x_all.row_sum();
    let kdab_mean = x_all.row_mean();
    let kdab_var = x_all.row_variance();
    let kds_mean = kd_spread.row_mean();
    let kds_var = kd_spread.row_variance();
    let kdsa_mean = kda_spread.row_mean();
    let kdsa_var = kda_spread.row_variance();

    //we want -1 sigma and +1 sigma for all of: Baseline and all equipment

    //summ up kdab for each factor:
    let factor_specific_kdab = factor_matrix.transpose() * x_all;
    let factor_counts = factor_matrix.row_sum().transpose();
    let mut repeat_factor_counts = DMatrix::<f32>::zeros(cur_size,4);
    for i in 0..4{
        repeat_factor_counts.set_column(i,&factor_counts);
    }
    //I wonder what 0/0 is around here
    let factor_means = factor_specific_kdab.component_div(&repeat_factor_counts);

    //all right, all that so we can get sums and means for everything. 
    //how to get variances ... 

    //create factor-sized matrix of means

    //this will be just like factor_matrix, but will have (i,j)==> match i had j Kills.
    //we're doing this only for kills right now. (0th column in factor_means)

    /*
    let factor_divisors = DMatrix::<f32>::from_fn( cur_size,cur_size, 
        |i,_| 
        factor_counts.get(i).unwrap() / row_max  as f32
    );
    */
    //sample covariance divisor is N-1 for all elements of the cov matrix

    /* 
     * NOTE: IN WHAT FOLLOWS 0 --> Kills. We need tomake it easier to use 0..4
     * for KDAB
     */
    let factor_divisors = DMatrix::<f32>::from_fn( cur_size,cur_size, 
        |i,j| 
        1.0 / factor_counts.get(i).unwrap()
    );
    let factor_multiplicands = DMatrix::<f32>::from_fn ( 
        row_max,cur_size,
        |i,j| 
        factor_matrix.get( (i,j) ).unwrap() * x_k.get( i ).unwrap() - factor_means.get( (j,0) ).unwrap()
    );
    let factor_covariance = (factor_multiplicands.transpose() * factor_multiplicands).component_div(&factor_divisors);
    eprintln!("kill variances{}",factor_covariance);

    //let's print kill spread per item.
    eprintln!("{:>10} {:>2.2} +/-  {:>2.2}", "ALL", kdab_mean.get(0).unwrap(), kdab_var.get(0).unwrap().sqrt());
    for i in 0..cur_size
    {
        eprintln!("{:>10} {:>2.2} +/-  {:>2.2}", row_names.entry(i).or_insert("??".to_string()),  factor_means.get( (i,0 )).unwrap(), factor_covariance.get( (i,i) ).unwrap().sqrt());
    }

    Ok(())
}
