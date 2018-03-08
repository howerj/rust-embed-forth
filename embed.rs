#![crate_name = "embed"]

//use std::default::Default;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
mod eforth;

/// * `CORE_SIZE` is the total number of cells addressable by the virtual machine
const CORE_SIZE: usize = 0x8000;
/// * `SP0` is the starting point of the variable stack
const SP0: u16 = 0x2200;
/// * `RP0` is the starting point of the return stack
const RP0: u16 = 0x7fff;

/// `fputc` writes a single character of output to a file, and returns
/// all bits set on an error. It emulates the C function of the same name,
/// and is not a recommended way to output data in Rust, but is required for
/// the VM.
///
/// # Arguments
///
/// * `output`  - Output stream to write to
/// * `t`       - Single byte to write
///
/// # Returns
///
/// This function returns `t` on success and `0xffff` on error
///
fn fputc(output: &mut Write, t: u8) -> u16 {
	let u: [u8; 1] = [t as u8];
	if 1 == output.write(&u).unwrap() { t as u16 } else { 0xffff }
}

/// `fputc` gets a single character from an input stream, like the C function
/// with the same name, it returns all bits set (-1) on error. This is not a
/// very idiomatic way of doing things from a Rust point of view, but this
/// function is used by the virtual machine to get input, and it expects
/// errors to be signaled in the message.
///
/// # Arguments
///
/// `input` - Input stream to read from
///
/// # Returns
///
/// This function returns a single byte on success in the lower half a
/// 16-bit value, and all bits set (or `0xffff`) on failure.
fn fgetc(input: &mut Read) -> u16 {
	let mut u: [u8; 1] = [0];
	if 1 == input.read(&mut u).unwrap() { u[0] as u16 } else { 0xffff }
}

/// # Embed Virtual Machine in Rust
///
/// * LICENSE:    MIT
/// * AUTHOR:     Richard James: Howe
/// * COPYRIGHT:  Richard James Howe (2018)
/// * CONTACT:    <howe.r.j.89@gmail.com>
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
/// * TODO: Implement Index trait for u16?
pub struct VM {
	/// `tracing` can be set true to enable logging, logging is very verbose
	tracing: bool,
	/// `count` is the number instructions executed so far, it is only updated
	/// if tracing is on.
	count: u64,
	/// The virtual machine has minimal state, a program counter (`pc`),
	/// a return stack pointer `rp`, a data stack pointer `sp` and a top
	/// of stack pointer `t`.
	pc: u16, rp: u16, sp: u16, t: u16, 
	/// `core` contains the program, data, and both stacks which index
	/// into `core` with `rp` and `sp`
	//#[derive(Copy, Clone)]
	core: [u16; CORE_SIZE] 
}

impl VM {

	/// `new` constructs a new virtual machine image that can be passed to `run`
	/// straight away, as the program memory is copied from a default image
	/// that contains a eForth interpreter.
	pub fn new() -> Self { 
		let mut r = VM { tracing: false, count: 0, pc: 0, rp: RP0, sp: SP0, t: 0, core: [0; CORE_SIZE] };

		for i in 0..eforth::EFORTH_CORE.len() {
			r.core[i] = eforth::EFORTH_CORE[i];
		}
		r
	}

	/// `reset` sets the VMs registers back to their defaults, it does not zero
	/// out the program memory or the stack contents, but the stack pointers, top
	/// of stack register, and the program counter.
	pub fn reset(&mut self) {
		self.pc = 0;
		self.rp = RP0;
		self.sp = SP0;
		self.t  = 0;
	}

	/// Turns logging on/off, capturing each VM instructions execution
	/// 
	/// # Arguments
	///
	/// * `state` - Turn _very_ verbose tracing on/off, each instruction is logged to stderr in CSV format
	///
	pub fn trace(&mut self, state: bool)
	{
		self.tracing = state;
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
	/// # Returns
	///
	/// This function returns an error code suitable for use with 
	/// `std::process:exit()`, negative values usually indicate failure, however
	/// any semantics attached to this number are entirely by convention only,
	/// the program running in the virtual machine can return any number it likes.
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
	/// let mut file = File::open(&Path::new("vm.blk")).unwrap();
	/// evm.load(&mut file);
	/// evm.run(Some("new.blk"), &mut std::io::stdin(), &mut std::io::stdout());
	/// ```
	/// 
	pub fn run(&mut self, block: Option<&str>, input: &mut Read, output: &mut Write) -> i32 {
		let (mut pc, mut rp, mut sp, mut t) = (self.pc, self.rp, self.sp, self.t);
		let mut d: u32;
		let mut m = self.core;

		VM::header(self, &mut std::io::stderr());

		'eval: loop {
			let instruction = m[pc as usize];
			const DELTA: [u16; 4] = [0, 1, 0xfffe, 0xffff];

			VM::csv(self, &mut std::io::stderr(), pc, instruction, t, sp, rp);

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
					23 => { tp = fputc(output, t as u8) } 
					24 => { tp = fgetc(input) }
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

	/// Print a header for a CSV file trace, if tracing is enabled, the output should be consumable
	/// by the utility <https://github.com/carlos-jenkins/csv2vcd> which can turn a CSV file into
	/// a VCD (Value Change Dump) file. This file can be used with a suitable waveform viewer, such
	/// as GTKWave <http://gtkwave.sourceforge.net/> for debugging purposes. This is not a generic
	///
	fn header(&self, output: &mut Write) {
		if !self.tracing { return }
		let _ignore = writeln!(output, "\"pc[15:0]\",\"instruction[15:0]\",\"t[15:0]\",\"sp[7:0]\",\"rp[7:0]\",\"TIME\"");
	}

	/// `csv` is used by `run` to output a CSV file with one line per instruction cycle,
	/// it is for internal use only. Tracing has to be enabled and is off by default as it
	/// produces a lot of output. The output should be compatible with the tool csv2vcd
	/// [csv2vcd](https://github.com/carlos-jenkins/csv2vcd) and which can be viewed with
	/// [GTKWave](http://gtkwave.sourceforge.net/), which should aid in analyzing the copious
	/// amounts of data produced.
	///
	/// It should be noted that `csv` accepts the arguments it will print instead of printing
	/// out the values stored in `self`, as the value for the VM state such as the program
	/// counter and stack pointers are kept in locals until `run` returns, and only then are
	/// they updated.
	/// 
	/// Arguments are logged in order, `pc` being the left most field in a record line and
	/// `rp` the rightmost (of the values passed in, the rightmost field is actually a "time"
	/// field, needed for the VCD format).
	/// 
	/// # Arguments
	/// 
	/// * `output`       - output stream to log to
	/// * `pc`           - the program counter
	/// * `instruction`  - the current instruction being executed, or `self->core[pc]`
	/// * `t`            - top of stack register
	/// * `sp`           - variable stack pointer, index into `core`
	/// * `rp`           - return stack pointer, index into `core`
	/// 
	/// 
	fn csv(&mut self, output: &mut Write, pc: u16, instruction: u16, t: u16, sp: u16, rp: u16) -> () {
		if !self.tracing { return }
		let time = if self.count == 0 { "s" } else { "ns" };
		let _ignore = writeln!(output, "{}", format!("{:04x},{:04x},{:04x},{:02x},{:02x},{}{}", pc, instruction, t, sp, rp, self.count, time));
		self.count += 1;
	}

	/// `save_file` is for internal use only, as it converts any errors into results understandable
	/// by the virtual machine. Its purpose is to save optionally save
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
	/// ```
	/// use std::fs::File;
	/// use std::path::Path;
	/// let mut vm = embed::VM::new();
	/// let mut output = File::create(&Path::new("vm.blk")).unwrap();
	/// vm.save(&mut output);
	/// ```
	///
	/// TODO: Replace Option with proper Result return value
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
	/// use std::fs::File;
	/// use std::path::Path;
	/// let mut vm = embed::VM::new();
	/// let mut input = File::open(&Path::new("vm.blk")).unwrap();
	/// vm.load(&mut input);
	/// ```
	///
	/// TODO: Replace Option with proper Result return value
	pub fn load(&mut self, input: &mut Read) -> Option<u16> {
		let mut i = 0 as u16;
		self.reset();
		while i < (CORE_SIZE as u16) {
			let lo = fgetc(input);
			let hi = fgetc(input);
			if lo == 0xffff || hi == 0xffff { return Some(i) }
			self.core[i as usize] = lo | (hi << 8);
			i += 1
		};
		Some(i)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::cmp;

	const BYE: u16 = 0x7b00;
	const ADD: u16 = 0x6523;
	const DEC: u16 = 0x6B00;

	fn literal(l: u16) -> u16 {
		if l & 0x8000 == 0x8000 { panic!("invalid literal {} > 0x7fff", format!("{}", l)) };
		l | 0x8000
	}

	fn core(dst: &mut [u16], src: &[u16]) {
		let len = cmp::min(src.len(), dst.len());
		for i in 0..len {
			dst[i] = src[i]
		}	
	}

	fn expect(vm: &mut VM, val: i32, program: &[u16]) {
		let (mut input, mut output) = (std::io::stdin(), std::io::stdout());
		core(&mut vm.core, program);
		assert_eq!(vm.run(None, &mut input, &mut output), val);
		vm.reset();
	}

	#[test]
	fn run() {
		let mut vm = VM::new();

		expect(&mut vm, 99, &[literal(99), BYE]);
		expect(&mut vm, 54, &[literal(55), DEC, BYE]);
		expect(&mut vm, 4,  &[literal(2),  literal(2), ADD, BYE]);
	}
}

