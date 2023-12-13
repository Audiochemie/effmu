use regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::f32::EPSILON;
use std::fs;
use std::io::Read;
fn print_usage() {
    println!(
        "effmu [fragment file toml] [proper file mulliken] [state to construct effective mu for]"
    );
    panic!("Did not call correctly!")
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Fragments {
    fragment1: Vec<usize>,
    fragment2: Vec<usize>,
}

impl Fragments {
    fn new(f1: &[usize], f2: &[usize]) -> Self {
        Self {
            fragment1: Vec::from(f1),
            fragment2: Vec::from(f2),
        }
    }
}

/// Represents a dipole vector, i.e. a charge weighted cartesian vector.
#[derive(Debug, PartialEq)]
struct Dipole{compnents: [f32;3]}

impl Default for Dipole {
    fn default() -> Self {
        Self{compnents: [EPSILON, EPSILON, EPSILON]}
    }
}

fn parse_fragment_toml_file(frag_file: &mut str) -> Result<Fragments, Box<dyn Error>> {
    let mut fh = fs::File::open(frag_file)?;
    let mut buf = String::new();
    fh.read_to_string(&mut buf)?;
    let frags: Fragments = toml::from_str(&buf)?;
    Ok(frags)
}


fn parse_dipole_from_prp(
    prp_file: &str,
    state_label: String,
) -> Result<Vec<Dipole>, Box<dyn Error>> {
    let cmpnts = ["y", "z"];
    let mut fh = fs::File::open(prp_file)?;
    let mut buf = String::new();
    fh.read_to_string(&mut buf)?;
    let mut lines = buf.lines();
    let rx = regex::Regex::new(&(String::from("t-density") + ".*" + &state_label)).unwrap();
    // Find start of transition density output
    lines.position(|x| rx.is_match(x)).unwrap();
    let rx = regex::Regex::new(&(state_label.clone() + ".*dipole.*" + "x")).unwrap();
    // Find start of input for x-component. Is always the first
    lines.position(|x| rx.is_match(x)).unwrap();
    let mut dip_vec: Vec<Dipole> = Vec::new();
    for line in lines.by_ref() {
            if line.starts_with("total") {
                break;
            }
            let cntn: Vec<&str> = line.split_whitespace().collect();
            let dpl = Dipole{compnents: [cntn[3].parse::<f32>().unwrap(), EPSILON, EPSILON]};
            dip_vec.push(dpl);
        }
    for (index, cmp) in cmpnts.iter().enumerate() {
        let rx = regex::Regex::new(&(state_label.clone() + ".*dipole.*" + cmp)).unwrap();
        lines.position(|x| rx.is_match(x)).unwrap();
        for line in lines.by_ref() {
            if line.starts_with("total") {
                break;
            }
            let cntn: Vec<&str> = line.split_whitespace().collect();
            dip_vec[cntn[0].parse::<usize>().unwrap()-1].compnents[index+1] = cntn[3].parse::<f32>().unwrap();
        }
    }
    Ok(dip_vec)
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        print_usage()
    }
    let state_label = args.pop().unwrap();
    let prp_file = args.pop().unwrap();
    let mut frag_file = args.pop().unwrap();
    let frags = parse_fragment_toml_file(&mut frag_file).unwrap();
    let nmbrs_nd_chrgs: Vec<Dipole> =
        parse_dipole_from_prp(&prp_file, state_label).unwrap();
}

#[cfg(test)]
mod unit_test {
    use crate::{parse_dipole_from_prp, Fragments, Dipole};

    #[test]
    fn test_parse_fragment_toml_file() {
        let frags: Fragments = toml::from_str(
            r#"
        fragment1=[1,2,3]
        fragment2=[4,5,6]"#,
        )
        .unwrap();
        let expected = Fragments::new(&[1_usize, 2, 3], &[4_usize, 5, 6]);
        assert_eq!(frags, expected)
    }

    #[test]
    fn test_parse_dipole_from_prp() {
        let test_file = "singlet.refconf.nos.prp";
        let line = parse_dipole_from_prp(test_file, String::from("2a")).unwrap();
        assert_eq!(line[0], Dipole{ compnents: [-0.00877_f32, -0.00950,-0.01102] })
    }
}
