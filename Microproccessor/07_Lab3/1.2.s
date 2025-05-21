        AREA |.rodata|, DATA, READONLY

; -------- 32-bit variables --------
X32     DCD     0x0000000A      ; X32 = 10
Y32     DCD     0x00000005      ; Y32 = 5


; -------- Results for 32-bit logical operations --------
A32     DCD     0               ; AND
O32     DCD     0               ; OR
NO32    DCD     0               ; NOR
NA32    DCD     0               ; NAND
XO32    DCD     0               ; XOR
XN32    DCD     0               ; XNOR
;DCD for 32-bit variable

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        
        ; ==== 32-bit Logical Operations ====
        ; Load 32-bit X and Y
        LDR     R0, =X32
        LDR     R1, [R0]        ; R1 = X32

        LDR     R0, =Y32
        LDR     R2, [R0]        ; R2 = Y32

        ; AND
        AND     R3, R1, R2
        LDR     R0, =A32
        STR     R3, [R0]

        ; OR
        ORR     R4, R1, R2
        LDR     R0, =O32
        STR     R4, [R0]

        ; NOR
        ORR     R5, R1, R2
        MVN     R5, R5
        LDR     R0, =NO32
        STR     R5, [R0]

        ; NAND
        AND     R6, R1, R2
        MVN     R6, R6
        LDR     R0, =NA32
        STR     R6, [R0]

        ; XOR
        EOR     R7, R1, R2
        LDR     R0, =XO32
        STR     R7, [R0]

        ; XNOR
        EOR     R8, R1, R2
        MVN     R8, R8
        LDR     R0, =XN32
        STR     R8, [R0]

        ; Infinite loop
stop    B stop

        END
