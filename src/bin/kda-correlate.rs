use std::ops::DivAssign;
use std::io;
use std::io::BufRead;
use std::string::String;
use nalgebra::base::DMatrix;

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

    if num_vars==0 || num_rows==0 {
        return Err("Did not load any data!".to_string());
    }
    
    let all_inputs = DMatrix::<f32>::from_fn(
        num_rows, num_vars+4, 
        |i,j|
        if j < 4
        {
            *kdab_entries.entry( (i,j) ).or_insert(0.0)
        } else {
            *data_entries.entry( (i,j-4) ).or_insert(0.0)
        }
    );

    let all_counts = all_inputs.row_sum();

    /* 
    //because of the kvc journal format, you never start a column without at
    //least one row >0 in that column
    Not so! It can be zero in fact! We are appending zeros for KDAB, when
    we explicitly consruct those vectors when no data may exist for them.
    for i in 0..all_counts.nrows()
    {
        for j in  0..all_counts.ncols()
        {
            let element  = all_counts.get( (i,j) ).unwrap();
            assert!( *element>0.0, std::format!("{},{} was: {} in {}",i,j,*element,all_counts));
        }
    }
    */

    let exp_counts = all_counts / num_rows as f32;
    let val_minus_mean = DMatrix::<f32>::from_fn(
        num_rows, num_vars+4, 
        |i,j|
        (*all_inputs.get( (i,j) ).unwrap()) - (*exp_counts.get( (0,j) ).unwrap())
    );
    let mut cov= val_minus_mean.transpose()*val_minus_mean;

    assert!(cov.nrows() == num_vars + 4);
    assert!(cov.ncols() == num_vars + 4);

    let scalar = num_rows as f32 - 1.0;
    cov.div_assign(scalar);
    let pairwise_corr = DMatrix::<f32>::from_fn(
        num_vars+4,num_vars+4,
        |i,j|
        if *cov.get( (i,j) ).unwrap() == 0.0{
            0.0
        } else {
            (*cov.get( (i,j) ).unwrap()) /
            (*cov.get( (i,i) ).unwrap()).sqrt() /
            (*cov.get( (j,j) ).unwrap()).sqrt()
        }
    );

    assert!(pairwise_corr.nrows() == num_vars + 4);
    assert!(pairwise_corr.ncols() == num_vars + 4);

    print!("{} {} {} {} {} ","*","K","D","A","B");
    for i in 0..num_vars{
        print!("{} ",col_to_name.entry(i).or_insert("??".to_string()));
    }
    println!("");
    for i in 0..pairwise_corr.nrows(){
        let row_name = match i{
            0=>"K",
            1=>"D",
            2=>"A",
            3=>"B",
            _=> col_to_name.entry(i-4).or_insert("??".to_string()),
        };
        print!("{} ",row_name);
        for j in 0..pairwise_corr.nrows(){
            print!("{} ",std::format!("{:1.4}", pairwise_corr.get( (i,j)).unwrap()));
        }
        println!("");
    }
    //done:
    Ok(())
}
