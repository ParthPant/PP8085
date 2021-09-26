; COMMENT DESCRIPTION
        MVI A, 7h
NEXT:   DCR A
        OUT ffh  
        JNZ NEXT 
        HLT
