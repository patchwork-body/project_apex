#![allow(missing_docs)]

use apex::Apex;
use apex::prelude::*;
use counter::Counter;

fn main() {
    let _ = Apex::new().hydrate(tmpl! { <Counter /> });
}
