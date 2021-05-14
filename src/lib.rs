use std::cmp::Ordering;
pub fn version()->String{
    return std::format!("{}-Alpha, built with kvc {} and poisson-rate-test {}",
    env!("CARGO_PKG_VERSION").to_string(),
    kvc::version(),
    poisson_rate_test::version()).to_string();
}
pub fn about()->String{
    "Some basic conditional probabilities and bayesian analysis on a KVC log. \n\
    Copyright (C) 2021 Joshua Vander Hook\n\n\
    This program comes with ABSOLUTELY NO WARRANTY.\n\
    This is free software, and you are welcome to redistribute\n\
    it under certain conditions. See LICENSE for more details or\n\
    https://github.com/jodavaho/kda-tools for more info. ".to_string()
}

#[derive(Debug)]
pub struct ResultRecord{
    pub metric_name:String,
    pub variable_groupings:String,
    pub p_val:f64,
    pub n_with:usize,
    pub n_without:usize,
    pub metric_with:f64,
    pub metric_without:f64,
    pub numer_with:usize,
    pub denom_with:usize,
    pub numer_without:usize,
    pub denom_without:usize,
    pub comment:String,
}
pub fn by_pval(a:&ResultRecord,b: &ResultRecord) -> Ordering
{
    if a.p_val.is_nan(){
        return std::cmp::Ordering::Greater;
    } else if b.p_val.is_nan(){
        return std::cmp::Ordering::Less;
    } else {
        return a.p_val.partial_cmp(&b.p_val).unwrap();
    }
}

#[derive(Debug)]
pub struct ShortRecord{
    pub metric_name:String,
    pub variable_groupings:String,
    pub p_val:f64,
    pub n_with:usize,
    pub n_without:usize,
    pub metric_with:f64,
    pub metric_without:f64,
    pub comment:String,
}
pub fn by_short_pval(a:&ShortRecord,b: &ShortRecord) -> Ordering
{
    if a.p_val.is_nan(){
        return std::cmp::Ordering::Greater;
    } else if b.p_val.is_nan(){
        return std::cmp::Ordering::Less;
    } else {
        return a.p_val.partial_cmp(&b.p_val).unwrap();
    }
}

pub fn align_output(outs: &Vec<String>,widths:&Vec<usize>,seperator:&str)
-> String{
    let len:usize=widths.iter().filter(|x| x < &&usize::MAX ).sum();
    let final_len = len + outs.len()-1;//<-seperators
    let mut ret = String::with_capacity(final_len);
    for i in 0..outs.len(){
        let wi=widths[i];
        let s = &outs[i];
        let slen = s.len().min(wi);
        ret.push_str(&s[0..slen]);
        if i<outs.len()-1{
            ret.push_str(&seperator[..]);
            for _ in slen..wi{
                ret.push(' ');
            }
        } 
    }
    ret
}

mod test{
    #[test]
    fn test_inf_compare(){
        use claim::{assert_gt,assert_lt};
        let inf = f64::INFINITY;
        let x = 0.0;
        //comparisons succeed w/ inf
        assert_gt!(inf,x);
        assert_lt!(x,inf);
    }
    #[test]
    fn test_nan_compare_for_pvals(){
        
        let nan = f64::NAN;
        let x = 0.0;
        let y  =1.0;
        let cmpres = nan.partial_cmp(&x);
        match cmpres{
            Some(_)=>assert!(false,"Should not reach here, should never work"),
            None=>assert!(true),
        }
        let mut list = vec![nan,x,y];
        list.sort_by(
            |a,b| {
                if a.is_nan(){
                    return std::cmp::Ordering::Greater;
                } else if b.is_nan(){
                    return std::cmp::Ordering::Less;
                } else {
                    return a.partial_cmp(b).unwrap();
                }
            }
        );
        assert_eq!(list[0..2],[x,y]);
        assert!(list[2].is_nan());
    }
}