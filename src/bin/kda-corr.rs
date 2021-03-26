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

    row_names.insert(0,"Time".to_string());
    item_columns.insert("Time".to_string(),0);

    let local_sin = io::stdin();
    let mut line_itr = local_sin.lock().lines();
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
        let row_idx = input_row_number - 1 + offset; 

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
                let time = row_idx  as f32;
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
                *current_value=*current_value + 1.0;

                //and finally, add the element for time
                //It's ok to do this many times, and we will, since there are many of the same row_index values 
                // on many lines (see kda-stretch)
                data_entries.insert((  row_idx  ,0), (time+0.1).ln() );
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
    x_all.set_column(3,&x_a);
    x_all.set_column(2,&x_b);

    //this is silly, why can't Matrix implement copy?
    //Create a bunch of copies manually for later operations
    let mut ft = DMatrix::zeros(cur_size,row_max);
    let mut f = DMatrix::zeros(row_max,cur_size);
    factor_matrix.transpose_to(&mut ft);
    ft.transpose_to(&mut f);

    let factor_square = ft * f;
    let lu_decom_factor = factor_square.lu();
    let ftx_all = factor_matrix.transpose() * x_all;

    println!("Solving :");
    let weighting = match lu_decom_factor.solve( & ftx_all) 
    {
        Some(weights)=>weights,
        None=>{
            return Err("Could not solve for W".to_string());
        },
    };
    eprintln!("{}",weighting);
    let mut mat_ptr = weighting.iter();
    let cur_size = ftx_all.nrows();
    let default_value = "??".to_string();

    //we should call next() 4*cur_size times.
    eprintln!("{:>15}:{:>6}{:>6}{:>6}{:>6}","Weight","K","D","A","B");
    for idx in 0..cur_size {
        let row_name = row_names.get(&idx).unwrap_or_else(|| &default_value);
        eprint!("{:>15}:",row_name);
        for _ in 0..4{
            let weight = match mat_ptr.next() {
                None=>0.0,
                Some(w)=>*w,
            };
            let pretty_string = format!("{:0.2}",weight);
            eprint!("{:>6}",pretty_string);
        }

        eprintln!("");
    }
    assert_eq!(mat_ptr.next(),None);
    //done:
    Ok(())
}
