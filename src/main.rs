#![allow(static_mut_refs)]

mod core;
mod uci;

fn main() {
    let mut uci_class = uci::Uci::new();
    uci_class.connect_to_terminal();
}
