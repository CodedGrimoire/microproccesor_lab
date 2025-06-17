        AREA |.rodata|, DATA, READONLY
Array   DCD     1, 2, 3, 4, 9

        AREA |.data|, DATA, READWRITE
Max     DCD     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =Array
        LDR     R1, [R0]        
        MOV     R2, #1

loop
        CMP     R2, #5
        BEQ     done

        LDR     R3, [R0, R2, LSL #2]
        CMP     R3, R1
        BLS     skip
        MOV     R1, R3

skip
        ADD     R2, R2, #1
        B       loop

done
        LDR     R4, =Max
        STR     R1, [R4]

stop    B       stop
        END
