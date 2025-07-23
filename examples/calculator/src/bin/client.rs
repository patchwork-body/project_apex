#![allow(missing_docs)]

use apex::Apex;
use apex::prelude::*;
use calculator::Calculator;

fn main() {
    let _ = Apex::new().hydrate(tmpl! { <Calculator /> });
}
