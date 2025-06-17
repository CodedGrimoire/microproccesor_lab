        AREA |.rodata|, DATA, READONLY
Array   DCD     1, 2, 3, 4, 5

        AREA |.data|, DATA, READWRITE
Sum     DCD     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =Array       ; Base address
        MOV     R1, #0          ; Index
        MOV     R2, #0          ; Sum

loop
        CMP     R1, #5
        BEQ     done
        LDR     R3, [R0, R1, LSL #2]
        ADD     R2, R2, R3
        ADD     R1, R1, #1
        B       loop

done
        LDR     R4, =Sum
        STR     R2, [R4]

stop    B       stop
        END
