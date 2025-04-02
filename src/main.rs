use std::{io::{BufRead as _, BufReader, BufWriter, Read, Write}, net::TcpListener};

use shade_eval_print_loop::init;
use steel::{steel_vm::{engine::Engine, register_fn::RegisterFn}, SteelVal};

fn main() {
    init();
}
