#![crate_name = "embed"]

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

/// * `CORE_SIZE` is the total number of cells addressable by the virtual machine
pub const CORE_SIZE: usize = 0x8000;
/// * `SP0` is the starting point of the variable stack
pub const SP0: u16 = 0x2200;
/// * `RP0` is the starting point of the return stack
pub const RP0: u16 = 0x7fff;

/// # Embed Virtual Machine in Rust
///
/// * LICENSE:    MIT
/// * AUTHOR:     Richard James: Howe
/// * COPYRIGHT:  Richard James Howe (2018)
/// * CONTACT:    howe.r.j.89@gmail.com
/// * REPOSITORY: <https://github.com/howerj/rust-embed-forth>
///
/// This project implements a 16-bit dual stack virtual machine (VM) tailored to
/// execute Forth, it should also come with an image which this VM can run,
/// which will be in a separate file. The VM is not that robust and incorrect
/// code that overflows the stack might cause a panic.
/// 
/// The original C VM is available at <https://github.com/howerj/embed>, along
/// with more up to date VM images.
/// 
/// TODO: Implement Index trait for u16?
/// TODO: Add a default image in here for completeness sake
/// TODO: Add tests, not just documentation examples
pub struct VM {
	/// `trace` can be set true to enable logging, logging is very verbose
	pub trace: bool, 
	/// The virtual machine has minimal state, a program counter (`pc`),
	/// a return stack pointer `rp`, a data stack pointer `sp` and a top
	/// of stack pointer `t`.
	pc: u16, rp: u16, sp: u16, t: u16, 
	/// `core` contains the program, data, and both stacks
	core: [u16; CORE_SIZE] }

impl VM {
	pub fn new() -> VM { 
		VM {
			trace: false, pc: 0, rp: RP0, sp: SP0, t: 0, core: [0; CORE_SIZE]
		}
	}

	/// `run` executes the virtual machine on the currently loaded program
	/// in `core`. The specification for the virtual machine is too long
	/// for this document, but visit <https://github.com/howerj/embed> for
	/// more documentation.
	///
	/// # Arguments
	///
	/// * `input`  - Input file to read from
	/// * `output` - Output file to write to
	/// * `block`  - Optional name of file to write sections of memory to
	///
	/// # Example
	///
	/// The following example loads code into a new VM instance from file
	/// *eforth.blk*, and sets input and output to the standard streams.
	/// The save opcode will save to the file *new.blk*. What the VM will
	/// do depends entirely on the code in the *eforth.blk* file.
	///
	/// ```
	/// extern crate embed;
	/// use std::fs::File;
	/// use std::path::Path;
	/// 
	/// let mut evm = embed::VM::new();
	/// let mut file = File::open(&Path::new("eforth.blk")).unwrap();
	/// evm.load(&mut file);
	/// evm.run(Some("new.blk"), &mut std::io::stdin(), &mut std::io::stdout());
	/// ```
	/// 
	pub fn run(&mut self, block: Option<&str>, input: &mut Read, output: &mut Write) -> i32 {
		let (mut pc, mut rp, mut sp, mut t) = (self.pc, self.rp, self.sp, self.t);
		let mut d: u32;
		let mut m = self.core;

		'eval: loop {
			let instruction = m[pc as usize];
			const DELTA: [u16; 4] = [0, 1, 0xfffe, 0xffff];

			if self.trace { VM::trace(self, &mut std::io::stderr(), pc, instruction, t, sp, rp) }

			if 0x8000 & instruction == 0x8000 { /* literal */
				sp += 1;
				m[sp as usize] = t;
				t = instruction & 0x7fff;
				pc += 1;
			} else if 0xe000 & instruction == 0x6000 { /* ALU */
				let mut tp = t;
				let mut n = m[sp as usize];
				pc = if instruction & 0x10 == 0x10 { m[rp as usize] >> 1 } else { pc + 1 };

				let alu = ((instruction >> 8) & 0x1f) as u8;
				match alu {
					0  => { /* tp = t */ }
					1  => { tp = n }
					2  => { tp = m[rp as usize] }
					3  => { tp = m[(t >> 1) as usize] }
					4  => { m[(t >> 1) as usize] = n; sp = sp - 1; tp = m[sp as usize] }
					5  => { d = (t as u32) + (n as u32); tp = (d >> 16) as u16; m[sp as usize] = d as u16; n = d as u16 }
					6  => { d = (t as u32) * (n as u32); tp = (d >> 16) as u16; m[sp as usize] = d as u16; n = d as u16 }
					7  => { tp &= n }
					8  => { tp |= n }
					9  => { tp ^= n }
					10 => { tp = !t }
					11 => { tp = tp.wrapping_sub(1) }
					12 => { tp = if t == 0 { 0xffff } else { 0 } }
					13 => { tp = if t == n { 0xffff } else { 0 } }
					14 => { tp = if n  < t { 0xffff } else { 0 } }
					15 => { tp = if (n as i16) < (t as i16) { 0xffff } else { 0 } }
					16 => { tp = n >> t }
					17 => { tp = n << t }
					18 => { tp = sp << 1 }
					19 => { tp = rp << 1 }
					20 => { sp = t >> 1 }
					21 => { rp = t >> 1; tp = n }
					22 => { tp = VM::save_file(self, block, n >> 1, (((t as u32) + 1) >> 1) as u16) } 
					23 => { tp = VM::fputc(output, t as u8) } 
					24 => { tp = VM::fgetc(input) }
					25 => { if t != 0 { tp = n / t; t = n % t; n = t } else { pc = 1; tp = 10 } }
					26 => { 
						if t != 0 { 
							tp = ((n as i16) / (t as i16)) as u16; 
							t = ((n as i16) % (t as i16)) as u16; 
							n = t 
						} else { pc = 1; tp = 10 } }
					27 => { break 'eval; }
					_  => { }
				}

				sp = sp.wrapping_add(DELTA[ (instruction       & 0x3) as usize]);
				rp = rp.wrapping_sub(DELTA[((instruction >> 2) & 0x3) as usize]);
				if instruction & 0x20 == 0x20 { tp = n; }
				if instruction & 0x40 == 0x40 { m[rp as usize] = t }
				if instruction & 0x80 == 0x80 { m[sp as usize] = t }
				t = tp;
			} else if 0xe000 & instruction == 0x4000 { /* call */
				rp -= 1;
				m[rp as usize] = (pc + 1) << 1;
				pc = instruction & 0x1fff;
			} else if 0xe000 & instruction == 0x2000 { /* 0branch */
				pc = if t == 0 { instruction & 0x1fff } else { pc + 1 };
				t = m[sp as usize];
				sp -= 1;
			} else { /* branch */
				pc = instruction & 0x1fff;
			}
		}
	
		self.pc = pc;
		self.rp = rp;
		self.sp = sp;
		self.t  = t;

		(t as i16) as i32
	}

	/// TODO: Generate output consumable by csv2vcd, which can then be viewed by GTKwave
	/// See <https://github.com/carlos-jenkins/csv2vcd> and <http://gtkwave.sourceforge.net/>
	fn trace(&self, output: &mut Write, pc: u16, instruction: u16, t: u16, sp: u16, rp: u16) -> () {
		let _ignore = writeln!(output, "{}", format!("{:04x}, {:04x}, {:04x}, {:02x}, {:02x}", pc, instruction, t, sp, rp));
	}

	fn fputc(output: &mut Write, t: u8) -> u16 {
		let u: [u8; 1] = [t as u8];
		if 1 == output.write(&u).unwrap() { t as u16 } else { 0xffff }
	}

	fn fgetc(input: &mut Read) -> u16 {
		let mut u: [u8; 1] = [0];
		if 1 == input.read(&mut u).unwrap() { u[0] as u16 } else { 0xffff }
	}

	fn save_file(&self, block: Option<&str>, start: u16, length: u16) -> u16 {
		let name = match block { None => return 0xffff, Some(name) => name };

		let mut file = match File::create(&Path::new(name)) {
			Err(r) => { println!("failed to create block \"{}\": {}", name, r); return 0xffff },
			Ok(r) => r
		};

		match VM::save_block(self, &mut file, start, length) {
			None => 0xffff,
			Some(r) => r
		}
	}

	fn save_block(&self, block: &mut Write, start: u16, length: u16) -> Option<u16> {
		if ((start as u32) + (length as u32)) > 0xffff { return None }

		for i in start..length {
			let lo =  self.core[i as usize] as u8;
			let hi = (self.core[i as usize] >> 8) as u8;
			let u: [u8; 2] = [lo, hi];
			if let Err(r) = block.write(&u) {
				let _ignore = r;
				return None;
			}
		}
		Some(0)
	}

	/// `save` the virtual machine to a sink, this saves the program/data
	/// core but none of the registers.
	///
	/// # Arguments
	///
	/// `output` - Output sink to write to, usually a file
	///
	/// # Example 
	///
	/// TODO: Add default image and example saving it to disk
	pub fn save(&self, output: &mut Write) -> Option<u16> {
		VM::save_block(self, output, 0, CORE_SIZE as u16)
	}

	/// `load` the virtual machine from a source, this also reinitializes
	/// the VM registers to their default values
	///
	/// # Arguments
	///
	/// * `input` - Input source to read from containing core code
	///
	/// # Example
	///
	/// ```
	/// use embed::VM;
	/// use std::fs::File;
	/// use std::path::Path;
	/// let mut evm = embed::VM::new();
	/// let mut file = File::open(&Path::new("eforth.blk")).unwrap();
	/// evm.load(&mut file);
	/// ```
	///
	pub fn load(&mut self, input: &mut Read) -> Option<u16> {
		let mut i = 0 as u16;
		self.pc = 0;
		self.t = 0;
		self.rp = RP0;
		self.sp = SP0;
		while i < (CORE_SIZE as u16) {
			let lo = VM::fgetc(input);
			let hi = VM::fgetc(input);
			if lo == 0xffff || hi == 0xffff { return Some(i) }
			self.core[i as usize] = lo | (hi << 8);
			i += 1
		};
		Some(i)
	}
}

