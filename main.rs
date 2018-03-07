extern crate embed;

use std::fs::File;
use std::path::Path;
use std::env;

fn main()
{
	let args: Vec<_> = env::args().collect();
	if args.len() != 2 && args.len() != 3 {
		panic!("Usage: {} vm.blk new.blk?");
	}

	let new = if args.len() == 3 { &args[2] } else { &args[1] };

	let mut vm = embed::VM::new();
	let mut file = File::open(&Path::new(&args[1])).unwrap();
	vm.load(&mut file);

	std::process::exit(vm.run(Some(new), &mut std::io::stdin(), &mut std::io::stdout()));
}

