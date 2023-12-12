use std::error::Error;
use std::fs;
use std::env;
use std::io::Read;
use serde::{Serialize, Deserialize};
fn print_usage() {
    println!("effmu [fragment file toml] [proper file mulliken] [state to construct effective mu for]");
    panic!("Did not call correctly!")
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Fragments {
    fragment1: Vec<usize>,
    fragment2: Vec<usize>
}

impl Fragments {
   fn new(f1: &[usize], f2: &[usize]) -> Self {
        Self { fragment1: Vec::from(f1), fragment2: Vec::from(f2)}
   }
}
 
fn parse_fragment_toml_file(frag_file: &mut str) -> Result<Fragments, Box<dyn Error>> {
   let mut fh = fs::File::open(frag_file)?;
   let mut buf = String::new();
   fh.read_to_string(&mut buf)?;
   let frags: Fragments = toml::from_str(&buf)?;
   Ok(frags)
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        print_usage()
    }
    let state_label = args.pop().unwrap();
    let mut prp_file = args.pop().unwrap();
    let mut frag_file = args.pop().unwrap();
    let frags = parse_fragment_toml_file(&mut frag_file).unwrap();


}

#[cfg(test)]
mod unit_test {
    use crate::Fragments;

    #[test]
    fn test_parse_fragment_toml_file() {
        let frags: Fragments = toml::from_str(r#"
        fragment1=[1,2,3]
        fragment2=[4,5,6]"#).unwrap();
        let expected = Fragments::new(&[1_usize,2,3], &[4_usize,5,6]);
        assert_eq!(frags, expected)
    }
}
