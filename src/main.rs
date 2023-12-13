use regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::f32::EPSILON;
use std::f32::consts::PI;
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
#[derive(Debug, PartialEq, Clone, Copy)]
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
    let mut frags: Fragments = toml::from_str(&buf)?;
    frags.fragment1.sort();
    frags.fragment2.sort();
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

fn map_dips_to_fragment(frag: Vec<usize>, nmbrs_nd_dps: &[Dipole]) -> Vec<Dipole> {
    let mut dipls_per_frag: Vec<Dipole> = Vec::with_capacity(frag.capacity());
    for indx in frag {
        dipls_per_frag.push(nmbrs_nd_dps[indx-1])
    }
    dipls_per_frag
}

fn dot_prd(vec1: (f32, f32, f32), vec2: (f32,f32,f32)) -> f32 {
    vec1.0 * vec2.0 + vec1.1 * vec2.1 + vec1.2 * vec2.2
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
    let nmbrs_nd_dps: Vec<Dipole> =
        parse_dipole_from_prp(&prp_file, state_label).unwrap();
    let dipoles_fragmnt1: Vec<Dipole> = map_dips_to_fragment(frags.fragment1, &nmbrs_nd_dps);
    let dipoles_fragmnt2: Vec<Dipole> = map_dips_to_fragment(frags.fragment2, &nmbrs_nd_dps);
    let zero = (0_f32, 0_f32, 0_f32);
    let total1 = dipoles_fragmnt1.iter().fold(zero, |acc, dpl| { (acc.0 + dpl.compnents[0], acc.1 + dpl.compnents[1], acc.2 + dpl.compnents[2]) });
    let total2 = dipoles_fragmnt2.iter().fold(zero, |acc, dpl| { (acc.0 + dpl.compnents[0], acc.1 + dpl.compnents[1], acc.2 + dpl.compnents[2]) });
    let total = (total1.0 + total2.0, total1.1 + total2.1, total1.2 + total2.2);
    println!("Fragment 1 has partial t-dipole: {:?}", total1);
    println!("Fragment 2 has partial t-dipole: {:?}", total2);
    println!("Total t-dipole is              : {:?}", total);
    let len_total1:f32 = dot_prd(total1, total1).sqrt();
    let len_total2:f32 = dot_prd(total2, total2).sqrt();
    let dot_12: f32= dot_prd(total1, total2);
    println!("Length of partial t-dipole on 1: {:?}", len_total1);
    println!("Length of partial t-dipole on 2: {:?}", len_total2);
    println!("Angle between partial dipoles  : {:?}", (dot_12/(len_total1 * len_total2)).acos() * 180.0_f32/PI);
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
