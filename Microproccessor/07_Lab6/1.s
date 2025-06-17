        AREA |.rodata|, DATA, READONLY
buffer   DCD     10, 20, 30, 40, 50      ; 5-element array

        AREA |.data|, DATA, READWRITE
First   DCD     0
Last    DCD     0

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =buffer      ; Base address
        
        LDR     R1, [R0]        ; Load first element
        LDR     R2, [R0, #16]   ; Load last element (4*4 = 16 offset)

        LDR     R3, =First
        STR     R1, [R3]

        LDR     R4, =Last
        STR     R2, [R4]

stop    B stop

        END

