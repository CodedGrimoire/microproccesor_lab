        AREA |.rodata|, DATA, READONLY
; -------- 16-bit variables --------
X16     DCW     0x000A          ; X16 = 10
Y16     DCW     0x0005          ; Y16 = 5



        AREA |.data|, DATA, READWRITE
; -------- Results for 16-bit logical operations --------
A16     DCW     0               ; AND
O16     DCW     0               ; OR
NO16    DCW     0               ; NOR
NA16    DCW     0               ; NAND
XO16    DCW     0               ; XOR
XN16    DCW     0               ; XNOR
;DCW for 16-bit variable


        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; ==== 16-bit Logical Operations ====
        ; Load 16-bit X and Y
        LDR     R0, =X16
        LDRH    R1, [R0]        ; R1 = X16

        LDR     R0, =Y16
        LDRH    R2, [R0]        ; R2 = Y16

        ; AND
        AND     R3, R1, R2
        LDR     R0, =A16
        STRH    R3, [R0]

        ; OR
        ORR     R4, R1, R2
        LDR     R0, =O16
        STRH    R4, [R0]

        ; NOR
        ORR     R5, R1, R2
        MVN     R5, R5
        LDR     R0, =NO16
        STRH    R5, [R0]

        ; NAND
        AND     R6, R1, R2
        MVN     R6, R6
        LDR     R0, =NA16
        STRH    R6, [R0]

        ; XOR
        EOR     R7, R1, R2
        LDR     R0, =XO16
        STRH    R7, [R0]

        ; XNOR
        EOR     R8, R1, R2
        MVN     R8, R8
        LDR     R0, =XN16
        STRH    R8, [R0]

       

        ; Infinite loop
stop    B stop

        END
