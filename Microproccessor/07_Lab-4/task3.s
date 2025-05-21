        AREA |.rodata|, DATA, READONLY
BCD_VAL   DCB     0x42         ; Example BCD: 0x42 → 42 decimal

        AREA |.data|, DATA, READWRITE
HEX_VAL   DCB     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =BCD_VAL
        LDRB    R1, [R0]            ; Load BCD
        ; Convert BCD → HEX
        ; (upper_nibble × 10) + lower_nibble
        MOV     R2, R1
        AND     R3, R2, #0x0F       ; lower nibble
        AND     R4, R2, #0xF0
        LSRS    R4, R4, #4
        MOV     R5, #10
        MUL     R4, R4, R5
        ADD     R6, R4, R3

        LDR     R0, =HEX_VAL
        STRB    R6, [R0]

stop    B stop
        END
