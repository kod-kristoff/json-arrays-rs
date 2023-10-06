use std::env;

use json_arrays::load_from_file;

fn main() {
    let mut args = env::args().skip(1);
    let input = args.next().expect("INPUT required");
    dbg!(&input);

    for entry in load_from_file(input).expect("that INPUT exists") {
        println!("{:?}", entry);
    }
}
