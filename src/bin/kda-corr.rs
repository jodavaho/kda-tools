use std::collections::HashMap;
use std::cmp::max;
use std::io::BufRead;
use std::io;
//use std::fs::File;
//use std::path::Path;
//use std::io::BufRead;
use std::string::String;
use nalgebra::base::DMatrix;

/**
 * Takes a long sequence of kda stats (see kda-stretch), and computes correlations
 */
fn main() -> Result<(),i32> {

    //helper variables (not used in optimization)
    let mut offset = 0;
    let mut line_count = 0;
    let mut processed_lines =0;

    //optimization varaibles read in
    let mut item_columns: HashMap<String,usize> = HashMap::new();
    let mut column_names: HashMap<usize,String> = HashMap::new();
    let mut data_entries: HashMap<(usize,usize),f32> = HashMap::new();
    //size of the state:
    let mut row_max=0;

    //more, the target functions
    let mut output_b: HashMap<usize,f32> = HashMap::new();
    let mut output_kd: HashMap<usize,f32> = HashMap::new();
    let mut output_ad: HashMap<usize,f32> = HashMap::new();

    column_names.insert(0,"Time".to_string());
    item_columns.insert("Time".to_string(),0);

    for input_line in io::stdin().lock().lines()
    {
        line_count+=1;
        let line:String = match input_line
        {
            Ok(line)=>line,
            Err(_)=>continue,
        };
        let mut tok_iter = line.split_whitespace();
        let row_str = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>continue,
        };
        let row_idx:usize =  match row_str.parse::<usize>()
        {
            Ok(i)=>i,
            Err(e)=>{
                eprintln!("Line {} not a row #:{} (see kda-stretch) Error={}",line_count,row_str,e);
                continue
            },
        };
        if row_idx==0  && offset==0
        {
            eprintln!("Found zero-indexed row, assuming offset=1");
            offset=1;

        }
        let _ignore_this = match tok_iter.next()
        {
            Some(tok)=>tok,
            None=>{
                eprintln!("Line {} Cannot process: {}. Expected 3 tokens / line (see kda-stretch).",line_count,line);
                continue;
            },
        };
        let key = match tok_iter.next()
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
        //filter out the output variables:
        if key=="D" {
            let ad_spread_ptr = output_ad.entry(row_idx).or_insert(0.0);
            *ad_spread_ptr = *ad_spread_ptr - 1.0;
            let kd_spread_ptr = output_kd.entry(row_idx).or_insert(0.0);
            *kd_spread_ptr= *kd_spread_ptr - 1.0;
        } else if key=="A" {
            let ad_spread_ptr = output_ad.entry(row_idx).or_insert(0.0);
            *ad_spread_ptr = *ad_spread_ptr + 1.0;
        } else if key=="K" {
            let kd_spread_ptr = output_kd.entry(row_idx).or_insert(0.0);
            *kd_spread_ptr= *kd_spread_ptr + 1.0;
        } else if key=="B" {
            let b_spread_ptr = output_b.entry(row_idx).or_insert(0.0);
            *b_spread_ptr= *b_spread_ptr + 1.0;
        } else {
            let time = row_idx  as f32;
            //process as input variable
            //tally the column name
            let cur_size:usize = item_columns.len();
            let col_idx = *item_columns.entry(key.to_string()).or_insert(cur_size);

            //save the name for later output
            column_names.insert(col_idx,key.to_string());

            //insert a 1 (or +=1) the row/column.
            let current_value = data_entries.entry((row_idx,col_idx)).or_insert(0.0);

            //tally up the state size
            row_max = max(row_max,row_idx+offset);
            *current_value=*current_value + 1.0;

            //and finally, add the element for time
            data_entries.insert((row_idx,0),time);
            processed_lines+=1;
        }
    }
    let cur_size = item_columns.len();
    eprintln!("Processed {} lines, read: {} rows and {} variables",processed_lines, row_max,cur_size);
    //well this is nice. Might as well rename "from_fn" to "for fun":
    let factor_matrix =  DMatrix::<f32>::from_fn(row_max,cur_size, |i,j| *data_entries.entry((i,j)).or_insert(-1.0) );
    let _x_ad =          DMatrix::<f32>::from_fn(row_max,1, |i,_| * output_ad.entry(i).or_insert(0.0) );
    let _x_kd =          DMatrix::<f32>::from_fn(row_max,1, |i,_| * output_kd.entry(i).or_insert(0.0) );
    let x_b  =           DMatrix::<f32>::from_fn(row_max,1, |i,_| * output_b.entry(i).or_insert(0.0) );
    //ok, do some math to find how much each one contributed to the result (I think)

    println!("{}",factor_matrix);

    //this is silly, why can't Matrix implement copy?
    //Create a bunch of copies manually for later operations
    let mut ft = DMatrix::zeros(cur_size,row_max);
    let mut f = DMatrix::zeros(row_max,cur_size);
    factor_matrix.transpose_to(&mut ft);
    ft.transpose_to(&mut f);

    //math time!
    let factor_square = ft * f;
    //println!("{} {}",factor_square,x_b);
    let lu_decom_factor = factor_square.lu();
    println!("{}",lu_decom_factor.is_invertible());
    let ftx_b = factor_matrix.transpose() * x_b;
    //println!("{}",ftx_b);
    for (col,name) in column_names{
        eprintln!("{} is {}",col,name);
    }

    let weighting_b = match lu_decom_factor.solve( &ftx_b ) 
    {
        Some(weights)=>weights,
        None=>{
            eprintln!("Couldn't solve for w_b");
            DMatrix::zeros(1,1)
        },
    };
    println!("{}",weighting_b);
    //done:
    Ok(())
}
