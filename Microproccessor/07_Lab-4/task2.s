        AREA |.rodata|, DATA, READONLY
NUM8     DCB     20
DEN8     DCB     5
NUM16    DCW     300
DEN16    DCW     15
NUM32    DCD     600
DEN32    DCD     10

        AREA |.data|, DATA, READWRITE
QUO8     DCB     0
QUO16    DCW     0
QUO32    DCD     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; 8-bit Division
        LDR     R0, =NUM8
        LDRB    R1, [R0]
        LDR     R0, =DEN8
        LDRB    R2, [R0]
        UDIV    R3, R1, R2
        LDR     R0, =QUO8
        STRB    R3, [R0]

        ; 16-bit Division
        LDR     R0, =NUM16
        LDRH    R1, [R0]
        LDR     R0, =DEN16
        LDRH    R2, [R0]
        UDIV    R3, R1, R2
        LDR     R0, =QUO16
        STRH    R3, [R0]

        ; 32-bit Division
        LDR     R0, =NUM32
        LDR     R1, [R0]
        LDR     R0, =DEN32
        LDR     R2, [R0]
        UDIV    R3, R1, R2
        LDR     R0, =QUO32
        STR     R3, [R0]

stop    B stop
        END
