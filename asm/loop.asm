BITS 64
;  ========================================================================
;
;  (C) Copyright 2023 by Molly Rocket, Inc., All Rights Reserved.
;
;  This software is provided 'as-is', without any express or implied
;  warranty. In no event will the authors be held liable for any damages
;  arising from the use of this software.
;
;  Please see https://computerenhance.com for more information
;
;  ========================================================================

;  ========================================================================
;  LISTING 132
;  ========================================================================

global Write1
global Write2
global Write3
global Write4
global Write5

section .text

;
; NOTE(casey): These ASM routines are written for the Windows
; 64-bit ABI. They expect RCX to be the first parameter (the count),
; and in the case of MOVAllBytesASM, RDX to be the second
; parameter (the data pointer). To use these on a platform
; with a different ABI, you would have to change those registers
; to match the ABI.
;

Write1:
    align 64
.loop:
    mov [rdx], rax 
    sub rcx, 1
    jnle .loop
    ret

Write2:
    align 64
.loop:
    mov [rdx], rax 
    mov [rdx], rax 
    sub rcx, 2
    jnle .loop
    ret

Write3:
    align 64
.loop:
    mov [rdx], rax 
    mov [rdx], rax 
    mov [rdx], rax 
    sub rcx, 3
    jnle .loop
    ret

Write4:
    align 64
.loop:
    mov [rdx], rax 
    mov [rdx], rax 
    mov [rdx], rax 
    mov [rdx], rax 
    sub rcx, 4
    jnle .loop
    ret



Write5:
    align 64
.loop:
    mov [rdx], rax 
    mov [rdx], rax 
    mov [rdx], rax 
    mov [rdx], rax 
    mov [rdx], rax 
    sub rcx, 5
    jnle .loop
    ret

