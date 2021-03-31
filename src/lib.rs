pub mod helpers;
pub fn version()->String{
    return std::format!("0.2.1, built with kvc v {}",kvc::version()).to_string();
}