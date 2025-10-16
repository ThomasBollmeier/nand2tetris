// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

	// end = KBD
	@KBD
	D=A
	@end
	M=D

(LOOP)
	// if !keypressed goto ELSE
	@KBD
	D=M
	@ELSE
	D;JEQ
	// color = -1
	@color
	M=-1
	// goto INIT
	@INIT
	0;JMP
(ELSE)
	// color = 0
	@color
	M=0
(INIT)
	// addr = SCREEN
	@SCREEN
	D=A
	@addr
	M=D
	
(SCREEN_LOOP)
	// if addr == KBD goto LOOP
	@addr
	D=M
	@KBD
	D=D-A
	@LOOP
	D;JEQ
	// RAM[A] = color
	@color
	D=M
	@addr
	A=M
	M=D
	// addr = addr + 1
	@addr
	M=M+1
	// goto SCREEN_LOOP
	@SCREEN_LOOP
	0;JMP

