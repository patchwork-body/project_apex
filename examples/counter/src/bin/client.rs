use apex::Apex;
use counter::CounterPage;

fn main() {
    let _ = Apex::new().hydrate(CounterPage::new());
}
