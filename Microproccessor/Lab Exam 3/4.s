        AREA |.rodata|, DATA, READONLY
Array   DCD     1, 2, 3, 4, 9        ; Array of 5 elements
N       DCD     0x000005               ; Number of elements

        AREA |.data|, DATA, READWRITE
sorted  DCD     1                    ; Assume sorted initially

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =Array           ; R0 = pointer to Array
        LDR     R1, =sorted          ; R1 = address of 'sorted'
        MOV     R2, #1               ; Start from index 1
        MOV     R3, #0               ; R3 = index for previous element
        MOV     R4, #1               ; R4 = assume sorted = 1
        BL      Check_Sorted_Array

        BX      LR


Check_Sorted_Array
loop
        CMP     R2, #5               ;i<=N
        BGE     done

        LDR     R5, [R0, R3, LSL #2] ; previous element
        LDR     R6, [R0, R2, LSL #2] ; current element

        CMP     R5, R6
        BLS     next                 ; if prev <= current, continue
        MOV     R4, #0               ; else, not sorted
        B       done                

next
        ADD     R2, R2, #1           ;increment index
        ADD     R3, R3, #1           ;increment index
        B       loop

done
        STR     R4, [R1]             ; store sorted status
        BX      LR

        END
