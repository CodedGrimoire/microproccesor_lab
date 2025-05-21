        AREA |.rodata|, DATA, READONLY
X64_LO     DCD     0x0000000A        ; Low part of 64-bit number X
X64_HI     DCD     0x00000000        ; High part of 64-bit number X
Y64_LO     DCD     0x00000005        ; Low part of 64-bit number Y
Y64_HI     DCD     0x00000000        ; High part of 64-bit number Y

        AREA |.data|, DATA, READWRITE
ADD_LO     DCD     0
ADD_HI     DCD     0
SUB_LO     DCD     0
SUB_HI     DCD     0
MUL_LO     DCD     0
MUL_HI     DCD     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; Load low and high 32 bits of X
        LDR     R0, =X64_LO
        LDR     R1, [R0]
        LDR     R0, =X64_HI
        LDR     R2, [R0]

        ; Load low and high 32 bits of Y
        LDR     R0, =Y64_LO
        LDR     R3, [R0]
        LDR     R0, =Y64_HI
        LDR     R4, [R0]

        ; --- ADD 64-bit ---
        ADDS    R5, R1, R3
        ADC     R6, R2, R4
        LDR     R0, =ADD_LO
        STR     R5, [R0]
        LDR     R0, =ADD_HI
        STR     R6, [R0]

        ; --- SUB 64-bit ---
        SUBS    R5, R1, R3
        SBC     R6, R2, R4
        LDR     R0, =SUB_LO
        STR     R5, [R0]
        LDR     R0, =SUB_HI
        STR     R6, [R0]

        ; --- MUL (lower 32-bit only) ---
        UMULL   R5, R6, R1, R3
        LDR     R0, =MUL_LO
        STR     R5, [R0]
        LDR     R0, =MUL_HI
        STR     R6, [R0]

stop    B stop
        END
