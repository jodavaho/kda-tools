

pub fn print_nicely(name:String,mean:f32,stdd:f32,char_max:i32,min_val:f32,max_val:f32)
{
    eprint!("{:>20} {:>4} +/-  {:<4} |", 
        name, 
        std::format!("{:2.2}",mean),
        std::format!("{:2.2}",stdd)
    );
    let spread = max_val - min_val;
    let pct_left  = (mean-stdd-min_val)/spread;
    let chars_left = pct_left * char_max as f32;
    let pct_mid = (mean-min_val)/spread;
    let chars_mid = pct_mid * char_max as f32;
    let pct_right = (mean+stdd-min_val)/spread;
    let chars_right = pct_right  * char_max as f32;
    eprint!("|");
    for _ in 0..chars_left as i32{
        eprint!(" ");
    }
    for _ in chars_left as i32..chars_mid as i32{
        eprint!("-");
    }
    eprint!("({:2.2})",mean);
    for _ in chars_mid as i32..chars_right as i32{
        eprint!("-");
    }
    for _ in chars_right as i32..char_max {
        eprint!(" ");
    }
    eprint!("|");
    eprintln!("");
}
