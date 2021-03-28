use std::collections::HashMap;
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
    let mut line_count = 0;

    //optimization varaibles read in
    let mut item_columns: HashMap<String,usize> = HashMap::new();
    let mut row_names: HashMap<usize,String> = HashMap::new();
    let mut data_entries: HashMap<(usize,usize),f32> = HashMap::new();
    
    //more, the target functions
    let mut b_out: HashMap<usize,f32> = HashMap::new();
    let mut k_out: HashMap<usize,f32> = HashMap::new();
    let mut a_out: HashMap<usize,f32> = HashMap::new();
    let mut d_out: HashMap<usize,f32> = HashMap::new();

    row_names.insert(0,"Time".to_string());
    item_columns.insert("Time".to_string(),0);

    let local_sin = io::stdin();
    let mut line_itr = local_sin.lock().lines();

    k_out.insert(0,0.0);
    d_out.insert(0,0.0);
    a_out.insert(0,0.0);
    b_out.insert(0,0.0);

    let keywords = kvc::get_reserved_matchers();
    while let Some(line_read) = line_itr.next()
    {
        let line = line_read.unwrap_or("".to_string());
        let (keycounts,_) = kvc::read_kvc_line(&line, &keywords);
        if keycounts.len() > 0
        {
            line_count+=1;//we want to start with 1, mind you
            for (key,fval) in keycounts{
                match &key[..]{
                    "K"=>{
                        let k_ptr = k_out.entry(line_count).or_insert(0.0);
                        *k_ptr = *k_ptr + fval;
                    },
                    "D"=>{
                        let d_ptr = d_out.entry(line_count).or_insert(0.0);
                        *d_ptr = *d_ptr + fval;
                    },
                    "A"=>{
                        let a_ptr = a_out.entry(line_count).or_insert(0.0);
                        *a_ptr = *a_ptr + fval;
                    },
                    "B"=>{
                        let b_ptr = b_out.entry(line_count).or_insert(0.0);
                        *b_ptr = *b_ptr + fval;
                    },
                    _ => {
                        let time = line_count  as f32;
                        //process as input variable We're building a matrix F with
                        //columns = the state rows retrieve the column name (Or
                        //initialize new column), which confusingly correpsonds to the
                        //row name. This is cause we assume KDAB = W * X.
                        let cur_size:usize = item_columns.len();
                        let col_idx = *item_columns.entry(key.to_string()).or_insert(cur_size);

                        //save the name for later output
                        row_names.insert(col_idx,key.to_string());

                        //insert a 1 (or +=1) the row/column. We're doing present / not present.
                        let current_value = data_entries.entry((line_count,col_idx)).or_insert(0.0);
                        *current_value=1.0;

                        //and finally, add the element for time
                        //It's ok to do this many times, and we will, since there are many of the same row_index values 
                        // on many lines (see kda-stretch)
                        data_entries.insert((  line_count ,0), time );
                    }
                }
            }
        }
    }

    let cur_size = item_columns.len();

    //now add penalty row
    for col in 0..cur_size{
        data_entries.insert( (0,col)  , 1.0);
    }

    eprintln!("Processed {} rows and {} variables",line_count,cur_size);
    for i in 0..row_names.len(){
        eprint!(" {} ",row_names.entry(i).or_insert("??".to_string()));
    }
    eprintln!("");
    //well this is nice. Might as well rename "from_fn" to "for fun":
    let factor_matrix =DMatrix::<f32>::from_fn(line_count,cur_size, |i,j| *data_entries.entry((i,j)).or_insert(-1.0) );

    let x_k =          DVector::<f32>::from_fn(line_count, |i,_| * k_out.entry(i).or_insert(0.0) );
    let x_d =          DVector::<f32>::from_fn(line_count, |i,_| * d_out.entry(i).or_insert(0.0) );
    let x_a =          DVector::<f32>::from_fn(line_count, |i,_| * a_out.entry(i).or_insert(0.0) );
    let x_b =          DVector::<f32>::from_fn(line_count, |i,_| * b_out.entry(i).or_insert(0.0) );
    //ok, do some math to find how much each one contributed to the result (I think)
    let mut x_all = DMatrix::<f32>::zeros(line_count,4);
    x_all.set_column(0,&x_k);
    x_all.set_column(1,&x_d);
    x_all.set_column(2,&x_a);
    x_all.set_column(3,&x_b);

    //this is silly, why can't Matrix implement copy?
    //Create a bunch of copies manually for later operations
    let mut ft = DMatrix::zeros(cur_size,line_count);
    let mut f = DMatrix::zeros(line_count,cur_size);
    factor_matrix.transpose_to(&mut ft);
    ft.transpose_to(&mut f);

    let factor_square = ft * f;
    let lu_decom_factor = factor_square.lu();
    let ftx_all = factor_matrix.transpose() * x_all;

    println!("Solving :");
    eprintln!("{}",factor_matrix);
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
