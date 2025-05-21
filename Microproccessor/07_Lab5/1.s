        AREA |.rodata|, DATA, READONLY

; -------- 32-bit variables --------
X     DCD     0x0000000A      ; X = 10
Y     DCD     0x00000005      ; Y= 5
W     DCD     0x00000008      ; W= 8
Z     DCD     0x00000007      ; Z= 7


; -------- Results for 32-bit logical operations --------
T1        DCD     0              
T2        DCD     0               
T3        DCD     0               
Result    DCD     0               

;DCD for 32-bit variable

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        
       
        LDR     R0, =X
        LDR     R1, [R0]        ; R1 = X

        LDR     R0, =Y
        LDR     R2, [R0]        ; R2 = Y

        LDR     R0, =W
        LDR     R3, [R0]        ; R3 = W

        LDR     R0, =Z
        LDR     R4, [R0]        ; R4 = Z

        ; AND
        AND     R5, R3, R1      ;R5 keeps the value of W.X

        ; AND
        AND     R6, R2, R4      ;R6 keeps the value of Y.Z
        MVN     R7,R6           ;NAND of R6

        ; OR
        ORR     R8, R5, R7
        LDR     R0, =Result
        STR     R8, [R0]        ;the final result F   

      
        ; Infinite loop
stop    B stop

        END
