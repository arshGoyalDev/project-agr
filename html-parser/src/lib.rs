pub mod node;
pub mod parser;

pub use node::{Element, Node, Text};
pub use parser::{HTMLParser, print_tree};
