// Produktberechnung: RAM[2] = RAM[0] * RAM[1]

	// i = R1
	@R1
	D=M
	@i
	M=D
	// R2 = 0
	@R2
	M=0
(LOOP)
	// if i == 0 goto END
	@i
	D=M
	@END
	D;JEQ
	// R2 = R2 + R0
	@R2
	D=M
	@R0
	D=D+M
	@R2
	M=D
	// i = i - 1
	@i
	M=M-1
	@LOOP
	0;JMP
(END)
	@END
	0;JMP
