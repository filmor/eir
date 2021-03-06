extern crate itertools;
extern crate petgraph;
extern crate either;
extern crate pretty;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate lalrpop_util;

#[macro_use]
extern crate matches;

extern crate num_bigint;
extern crate num_traits;

extern crate eir;
extern crate pattern_compiler;
extern crate util as util_c;
extern crate cps_transform;

//pub mod intern;
//pub use self::intern::{ Atom, Variable };
pub use ::eir::intern::Atom;
pub type Variable = Atom;

pub mod parser;
pub mod ir;
pub mod util;
mod ssa;

//#[cfg(test)]
//mod erl_tests;

use pretty::{ BoxDoc, Doc };
pub trait ToDoc {
    fn to_doc<'a>(&'a self) -> Doc<'a, BoxDoc>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }

    #[test]
    fn basic_regress() {
        use std::fs;
        use std::io::Read;
        let paths = fs::read_dir("../test_data/basic_regress").unwrap();

        for path in paths {
            let path = path.unwrap();
            if path.file_name().to_str().unwrap().starts_with(".") {
                continue
            }

            println!("File: {:?}", path.path());
            assert!(path.file_type().unwrap().is_file());

            let mut f = fs::File::open(path.path()).unwrap();
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();

            let res = ::parser::parse(&contents).unwrap();
            let _hir = ::ir::from_parsed(&res.0);

        }
    }

}
