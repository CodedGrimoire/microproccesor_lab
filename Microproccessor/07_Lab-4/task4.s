        AREA |.rodata|, DATA, READONLY
HEX_NUM   DCB     25        ; Decimal 25

        AREA |.data|, DATA, READWRITE
BCD_NUM   DCB     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =HEX_NUM
        LDRB    R1, [R0]

        MOV     R2, #10
        UDIV    R3, R1, R2        ; R3 = tens
        MLS     R4, R3, R2, R1    ; R4 = ones

        LSL     R3, R3, #4        ; shift tens to upper nibble
        ORR     R5, R3, R4        ; combine to form BCD

        LDR     R0, =BCD_NUM
        STRB    R5, [R0]

stop    B stop
        END
