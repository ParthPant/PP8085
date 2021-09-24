; COMMENT DESCRIPTION
START:  MVI A, 4Ah
NXT:    MVI B, 32h
        SUB B
        out ffh
BACK:   jmp START   ;just a random comment
        rst 7
        jc NXT 
        push psw
        pop psw
END:    HLT
