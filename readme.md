# Rust Embed Forth: Rust Virtual Machine Version

| Project   | Forth Virtual Machine Written in Rust |
| --------- | ------------------------------------- |
| Author    | Richard James Howe                    |
| Copyright | 2018 Richard James Howe               |
| License   | MIT                                   |
| Email     | howe.r.j.89@gmail.com                 |

This project was derived from a Forth virtual machine and image available at
<https://github.com/howerj/embed>, this is a clone of the virtual machine
written in [Rust][] and containing a pre-compiled image for the virtual
machine, which contains a Forth interpreter. The meta-compiler and source for
the image are absent from this project, as is the virtual machine specification
and extensive documentation for the Forth interpreter and system, they are
available in the original project. For the latest version of the image
**eforth.blk**, download <https://github.com/howerj/embed/raw/master/eforth.blk>
and replace **eforth.blk** with this file.

## Building and Running

Type "cargo run" to build and run, or "make run". This will build the virtual
machine and execute it, it should read from the standard input stream and write
to the standard output stream. You should be greeted with a message that looks
like this:

	eFORTH V 1984
	 157E 2A82

Type 'words' for a list of all implemented Forth functions, for about eForth
visit <http://forth.org/eforth.html>, or look at the original [embed][] project.

For a list of problems, a 'To-Do' list, and more comments about this project
view the source file [embed.rs][]

[Rust]: https://www.rust-lang.org/en-US/
[embed]: https://github.com/howerj/embed
[embed.rs]: embed.rs
