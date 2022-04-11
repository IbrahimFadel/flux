use super::*;
use std::fmt;

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // self.top_level_declarations
        //     .into_iter()
        //     .for_each(|d| match d {
        //         Decl::FnDecl(x) => write!(f, "{}", x.to_string()),
        //         _ => write!(f, String::from("Illegal")),
        //     })
        write!(f, "{:#?}", self)
    }
}

// impl fmt::Display for FnDecl {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Function")
//     }
// }
