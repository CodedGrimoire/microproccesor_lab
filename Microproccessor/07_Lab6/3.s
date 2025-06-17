        AREA |.rodata|, DATA, READONLY
Array   DCD     10, 20, 30, 40, 50

        AREA |.data|, DATA, READWRITE
Rev     DCD     0, 0, 0, 0, 0s

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =Array       ; Base of Array
        ADR     R1, Rev         s
        MOV     R2, #0          ; Index for Array

loop
        CMP     R2, #5
        BEQ     done

        LDR     R3, [R0, R2, LSL #2]
        MOV     R4, #4
        MUL     R5, R4, R2
        RSBS    R5, R5, #16     ; R5 = 16 - (R2*4)
        STR     R3, [R1, R5]

        ADD     R2, R2, #1
        B       loop

done
stop    B       stop
        END
