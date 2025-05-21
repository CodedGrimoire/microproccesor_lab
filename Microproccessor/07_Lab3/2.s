        AREA |.rodata|, DATA, READONLY


; -------- 32-bit variables --------
X     DCD     0x0000000A      ; X32 = 10 binary =1010


        AREA |.data|, DATA, READWRITE

; -------- Results for 32-bit logical operations --------
Logical_left_shift      DCD     0               ; AND
Logical_right_shift     DCD     0               ; OR
Arithmetic_Shift_Right  DCD     0               ; NOR

;DCD for 32-bit variable

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
       
        LDR     R0, =X
        LDRH    R1, [R0]        ; R1 = X16

        

        ; LSL (Logical Shift Left by 1)
        LSL     R2, R1, #1
        LDR     R0, =Logical_left_shift
        STR     R2, [R0]

         ; LSR (Logical Shift Right by 1)
        LSR     R3, R1, #1
        LDR     R0, =Logical_right_shift
        STR     R3, [R0]

        ; ASR (Arithmetic Shift Right by 1)
        ASR     R4, R1, #1
        LDR     R0, =Arithmetic_Shift_Right
        STR     R4, [R0]


        ; Infinite loop
stop    B stop

        END
