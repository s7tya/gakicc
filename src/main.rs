use std::env::args;

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    println!("  .globl main");
    println!("main:");
    println!("  li a0, {}", args[1]);
    println!("  ret");
}
