        AREA |.rodata|, DATA, READONLY

; -------- 32-bit variables --------
X     DCD     0x00C123      ; X = C123 in hexadecimal



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

       

       

       
        MVN     R2,R1           ;1's comoplement of X

        
        
        LDR     R0, =Result
        STR     R2, [R0]        ;the final result F   

      
        ; Infinite loop
stop    B stop

        END
