
pub mod helpers;
pub fn version()->String{
    return std::format!("0.7.0, built with kvc {} and poisson-rate-test {}",kvc::version(),poisson_rate_test::version()).to_string();
}
pub fn about()->String{
    "Some basic conditional probabilities and bayesian analysis on a KVC log. \n\
    Copyright (C) 2021 Joshua Vander Hook\n\
    This program comes with ABSOLUTELY NO WARRANTY.\n\
    This is free software, and you are welcome to redistribute\n\
    it under certain conditions. See LICENSE for more details or\n\
    https://github.com/jodavaho/kda-tools for more info. ".to_string()
}
