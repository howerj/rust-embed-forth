/* Embed Virtual Machine in Rust
 *
 * LICENSE:    MIT
 * AUTHOR:     Richard James: Howe
 * COPYRIGHT:  Richard James Howe (2018)
 * CONTACT:    howe.r.j.89@gmail.com
 * REPOSITORY: <https://github.com/howerj/rust-embed-forth>
 *
 * This project implements a 16-bit dual stack virtual machine (VM) tailored to
 * execute Forth, it should also come with an image which this VM can run,
 * which will be in a separate file.
 * 
 * This will need to be documented, turned into idiomatic rust, made to be a
 * reusable library and generally cleaned up. The project is a work in
 * progress and the initial versions will be a straight forward translation of
 * the original C 'embed' Virtual Machine.
 * 
 * The original C VM is available at <https://github.com/howerj/embed>.
 */

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

const CORE_SIZE: usize = 0x8000;
const SP0: u16 = 8704;
const RP0: u16 = 32767;

struct EmbedVM { pc: u16, rp: u16, sp: u16, t: u16, core: [u16; CORE_SIZE] }

impl EmbedVM {
	// TODO: initialize from file and/or from memory block
	pub fn new() -> EmbedVM { 
		EmbedVM {
			pc: 0, rp: RP0, sp: SP0, t: 0, core: [0; CORE_SIZE]
		}
	}

	pub fn run(&mut self, input: &mut Read, output: &mut Write) -> i32 {
		let delta: [u16; 4] = [0, 1, 0xfffe, 0xffff];
		let mut pc = self.pc;
		let mut rp = self.rp;
		let mut sp = self.sp;
		let mut t  = self.t;
		let mut d: u32;
		let mut m = self.core;
		let mut halted = false;

		while !halted {
			let instruction = m[pc as usize];

			//println!("trace: {}", format!("{:04x} {:04x} {:02x} {:02x}", pc, instruction, sp, rp));

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
					22 => { tp = 0xffff } // TODO: implement this: save core
					23 => { 
						let u: [u8; 1] = [t as u8];
						tp = if 1 == output.write(&u).unwrap() { t } else { 0xffff };
					} 
					24 => { 
						let mut u: [u8; 1] = [0];
						tp = if 1 == input.read(&mut u).unwrap() { u[0] as u16 } else { 0xffff };
					}
					25 => { if t != 0 { tp = n / t; t = n % t; n = t } else { pc = 1; tp = 10 } }
					26 => { 
						if t != 0 { 
							tp = ((n as i16) / (t as i16)) as u16; 
							t = ((n as i16) % (t as i16)) as u16; 
							n = t 
						} else { pc = 1; tp = 10 } }
					27 => { halted = true }
					_ =>  { }
				}

				if !halted  {
					sp = sp.wrapping_add(delta[ (instruction       & 0x3) as usize]);
					rp = rp.wrapping_sub(delta[((instruction >> 2) & 0x3) as usize]);
					if instruction & 0x20 == 0x20 { tp = n; }
					if instruction & 0x40 == 0x40 { m[rp as usize] = t }
					if instruction & 0x80 == 0x80 { m[sp as usize] = t }

					t = tp;
				}
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
	// pub fn save(&self, string)
	//pub fn load(&self, name: String) {
	//} 

	//pub fn load(&self, name: &[u8]) {
	//}
	// fn trace(&self, output)
}

// TODO: Argument handling
fn main()
{
	// Replace with "let mut f = try!(File::open("embed.blk"));"?
	let path = Path::new("eforth.blk");
	let display = path.display();
	let mut evm = EmbedVM::new();

	let mut file = match File::open(&path) {
		Err(why) => panic!("couldn't open {}: {}", display, why.description()),
		Ok(file) => file,
	};

	// TODO: Read byte at a time, so temporary storage is not needed
	let mut b: [u8; CORE_SIZE*2] = [0; CORE_SIZE*2];
	let len = match file.read(&mut b) {
		Err(why) => panic!("read failed: {}", why.description()),
		Ok(result) => result
	};
	// println!("read: {}", format!("{}", len));

	let mut i = 0;
	while i < len {
		let lo = b[i] as u16;
		let hi = (b[i+1] as u16) << 8;
		evm.core[i/2] = lo | hi;
		i += 2;
	}

	let _ignore = evm.run(&mut std::io::stdin(), &mut std::io::stdout());
	//println!("returned: {}", r);
	// TODO: return val
}
