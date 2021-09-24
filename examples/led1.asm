	ORG 8000h
	
	JMP START	; Jump to ROM

START:  MVI A, 0C0h	; LED on
        SIM

        MVI A, 0FFh	; Delay
        MOV B, A
D1PT1:  DCR A
D1PT2:  DCR B
        JNZ D1PT2
        CPI 00h
        JNZ D1PT1

        MVI A, 40h	; LED off
        SIM

        MVI A, 0FFh	; Delay
        MOV B, A
D2PT1:  DCR A
D2PT2:  DCR B
        JNZ D2PT2
        CPI 00h
        JNZ D2PT1

        JMP START	; Loop forever
