#![allow(missing_docs)]

use apex::Apex;
use counter::Counter;

fn main() {
    let _ = Apex::new().hydrate(Counter { children: None });
}
