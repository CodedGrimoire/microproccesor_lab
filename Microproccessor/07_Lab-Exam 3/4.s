        AREA |.rodata|, DATA, READONLY
N       DCD     8                      ; Number of Fibonacci numbers

        AREA |.data|, DATA, READWRITE
Array   SPACE   32                     ; (8*4 = 32 bytes) array=0 1 1 2 3 5 8 13

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =Array             ; R0 = base address of Array
        LDR     R1, =N                 ; R1 = address of N
        LDR     R1, [R1]               ; R1 = value of N
        BL      Fibonacci_Sequence     ; Call subroutine
        BX      LR


Fibonacci_Sequence
        MOV     R2, #0                 ; i = 0
        MOV     R3, #0                 ; fib0 = 0
        MOV     R4, #1                 ; fib1 = 1

loop
        CMP     R2, R1                 ; if i >= N;  
        BGE     done

        CMP     R2, #0
        BEQ     store_fib0

        CMP     R2, #1
        BEQ     store_fib1

        ADD     R5, R3, R4            
        MOV     R3, R4                 
        MOV     R4, R5                 

        STR     R5, [R0, R2, LSL #2]   ; Array[i] = next
        ADD     R2, R2, #1
        B       loop

store_fib0
        STR     R3, [R0, R2, LSL #2]   ; Array[0] = 0
        ADD     R2, R2, #1
        B       loop

store_fib1
        STR     R4, [R0, R2, LSL #2]   ; Array[1] = 1
        ADD     R2, R2, #1
        B       loop

done
        BX      LR
        END
