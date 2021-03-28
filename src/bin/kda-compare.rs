
use std::f32;
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

    eprintln!("KVC version: {}",kvc::version());
    eprintln!("KDA-tools version: {}",kda_tools::version());
    
    let local_sin = io::stdin();
    let line_itr = local_sin.lock().lines();

    let (num_rows,mut kdab_entries,mut data_entries,mut col_to_name,_name_to_col) = 
    kda_tools::helpers::load_from_stream(line_itr);

    let num_vars = col_to_name.len();//# variables
    eprintln!("Processed. Read: {} rows and {} variables", num_rows,num_vars);
    
    //well this is nice. Might as well rename "from_fn" to "for fun":
    let factor_matrix =DMatrix::<f32>::from_fn(num_rows,num_vars, |i,j| *data_entries.entry((i,j)).or_insert(0.0) );

    let x_k =          DVector::<f32>::from_fn(num_rows, |i,_| * kdab_entries.entry( (i,0) ).or_insert(0.0) );
    let x_d =          DVector::<f32>::from_fn(num_rows, |i,_| * kdab_entries.entry( (i,1) ).or_insert(0.0) );
    let x_a =          DVector::<f32>::from_fn(num_rows, |i,_| * kdab_entries.entry( (i,2) ).or_insert(0.0) );
    let x_b =          DVector::<f32>::from_fn(num_rows, |i,_| * kdab_entries.entry( (i,3) ).or_insert(0.0) );
    
    let mut x_all = DMatrix::<f32>::zeros(num_rows,4);
    x_all.set_column(0,&x_k);
    x_all.set_column(1,&x_d);
    x_all.set_column(2,&x_a);
    x_all.set_column(3,&x_b);

    let kd_spread = x_k.clone() - x_d.clone();
    let kda_spread = (x_k.clone() + x_a.clone()) - x_d.clone();

    // calculate baseline mKDA
    let _kdab_sum = x_all.row_sum();
    let kdab_mean = x_all.row_mean();
    let kdab_var = x_all.row_variance();
    let _kds_mean = kd_spread.row_mean();
    let _kds_var = kd_spread.row_variance();
    let _kdsa_mean = kda_spread.row_mean();
    let _kdsa_var = kda_spread.row_variance();

    //we want -1 sigma and +1 sigma for all of: Baseline and all equipment

    //summ up kdab for each factor:
    let factor_specific_kdab = factor_matrix.transpose() * x_all;
    let factor_counts = factor_matrix.row_sum().transpose();
    let mut repeat_factor_counts = DMatrix::<f32>::zeros(num_vars,4);
    for i in 0..4{
        repeat_factor_counts.set_column(i,&factor_counts);
    }
    //I wonder what 0/0 is around here
    let factor_means = factor_specific_kdab.component_div(&repeat_factor_counts);

    /* 
     * NOTE: IN WHAT FOLLOWS 0 --> Kills. We need tomake it easier to use 0..4
     * for KDAB
     */
    let factor_divisors = DMatrix::<f32>::from_fn( num_vars,num_vars, 
        | i , _j | 
        *factor_counts.get(i).unwrap()
    );
    let factor_multiplicands = DMatrix::<f32>::from_fn ( 
        num_rows,num_vars,
        |i,j| 
        factor_matrix.get( (i,j) ).unwrap()  * ( x_k.get( i ).unwrap() - factor_means.get( (j,0) ).unwrap())
    );

    let factor_covariance = (factor_multiplicands.transpose() * factor_multiplicands).component_div(&factor_divisors);

    let mut min_val:f32 = std::f32::INFINITY;
    let mut max_val:f32 = std::f32::NEG_INFINITY;
    let char_width = 64;
    for i in 0..num_vars
    {
        let mean = factor_means.get( (i,0) ).unwrap();
        let stdd = factor_covariance.get( (i,i) ).unwrap().sqrt();
        min_val = min_val.min(mean-stdd);
        max_val = max_val.max(mean+stdd);
    }
    kda_tools::helpers::print_nicely(
        "ALL".to_string(), 
        *kdab_mean.get(0).unwrap(), 
        kdab_var.get(0).unwrap().sqrt(),
        char_width,
        min_val,
        max_val);
    for i in 0..num_vars {
        let mean = *factor_means.get( (i,0) ).unwrap();
        let stdd = factor_covariance.get( (i,i) ).unwrap().sqrt();
        let name = col_to_name.entry(i).or_insert("??".to_string());
        kda_tools::helpers::print_nicely(
            name.to_string(),
            mean,
            stdd,
            char_width,
            min_val,
            max_val) ;
    }

    Ok(())
}
