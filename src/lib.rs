use rand::thread_rng;
use  rand_distr::{Poisson,Distribution};

pub fn version()->String{
    return std::format!("0.8.1-Alpha, built with kvc {} and poisson-rate-test {}",kvc::version(),poisson_rate_test::version()).to_string();
}
pub fn about()->String{
    "Some basic conditional probabilities and bayesian analysis on a KVC log. \n\
    Copyright (C) 2021 Joshua Vander Hook\n\
    This program comes with ABSOLUTELY NO WARRANTY.\n\
    This is free software, and you are welcome to redistribute\n\
    it under certain conditions. See LICENSE for more details or\n\
    https://github.com/jodavaho/kda-tools for more info. ".to_string()
}
pub fn boostrap_kda(
    rate_k:f64,
    rate_d:f64,
    rate_a:f64
){
    //we need to create a likelihood function for KDA from the poisson variables
    //K,D and A Then, we use that to look-up the KDA likelihood w/ and w/o a set
    //of values and do a likelihood ratio test. We'll  use wilks' theorem: The
    //ratio is asymptotically disributed as chi-square with #DOF equal to the
    //difference in parameter dimension. 

    let n_samples = 1000;
    let kpdf = Poisson::new(rate_k).unwrap();
    let dpdf = Poisson::new(rate_d).unwrap();
    let apdf = Poisson::new(rate_a).unwrap();
    let s_k:Vec<f64> = kpdf.sample_iter(rand::thread_rng()).take(n_samples).collect();
    let s_d:Vec<f64> = dpdf.sample_iter(rand::thread_rng()).take(n_samples).collect();
    let s_a:Vec<f64> = apdf.sample_iter(rand::thread_rng()).take(n_samples).collect();

    //Actually no, we just need to test that hypothesis that (K+A)/D> some_kda.
    //That's actaully what we already built, since K+A is a poisson, and so is D
}