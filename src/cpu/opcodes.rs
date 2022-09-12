use crate::bus::MemoryBus;
use super::CPU;
use super::registers::Flag::{C, N, Z, H};

impl CPU {

    pub fn stack_push(&mut self, val: u16) {
        self.regs.sp -= 2;
        self.mem.write_word(self.regs.sp, val);
    }

    fn stack_pop(&mut self) -> u16 {
        let val = self.mem.read_word(self.regs.sp);
        self.regs.sp += 2;
        val
    }

    // ADD A, n - add n (+ carry) to A.
    fn alu_add(&mut self, n: u8, carry: bool) {
        let a: u8 = self.regs.a;
        let c: u8 = if self.regs.get_flag(C) && carry {1} else {0};
        let res = a.wrapping_add(n).wrapping_add(c);
        self.regs.set_flag(C, (a as u16) + (n as u16) + (c as u16) > 0xFF);     // Set if carry from bit 7.        
        self.regs.set_flag(H, (a&0xF) + (n&0xF) + c > 0xF);                     // Set if carry from bit 3. 
        self.regs.set_flag(N, false);                                           // Reset.
        self.regs.set_flag(Z, res == 0);                                        // Set if result is 0.
        self.regs.a = res;
    }

    // SUB n - subtract n (+ carry) from A.
    fn alu_sub(&mut self, n: u8, carry: bool) {
        let a: u8 = self.regs.a;
        let c: u8 = if self.regs.get_flag(C) && carry {1} else {0};
        let res = a.wrapping_sub(n).wrapping_sub(c);
        self.regs.set_flag(H, (a&0xF) < (n&0xF)+ c);    // Set if no borrow from bit 4.
        self.regs.set_flag(C, a < n + c);               // Set if no borrow.
        self.regs.set_flag(N, true);                    // Set.
        self.regs.set_flag(Z, res == 0);                // Set if result is 0.
        self.regs.a = res;
    }
    
    // AND n - logically AND n with A, result in A.
    fn alu_and(&mut self, n: u8) {
        let res = self.regs.a & n;
        self.regs.set_flag(Z, res == 0);    // Set if result is 0.
        self.regs.set_flag(N, false);       // Reset.
        self.regs.set_flag(H, true);        // Set.
        self.regs.set_flag(C, false);       // Reset.
        self.regs.a = res;
    } 

    // OR n - logical OR n with A, result in A.
    fn alu_or(&mut self, n: u8) {
        let res = self.regs.a | n;
        self.regs.set_flag(Z, res == 0);    // Set if result is 0.
        self.regs.set_flag(N, false);       // Reset.
        self.regs.set_flag(H, false);       // Reset.
        self.regs.set_flag(C, false);       // Reset.
        self.regs.a = res;
    }

    // XOR n - logical XOR n with A, result in A.
    fn alu_xor(&mut self, n: u8) {
        let res = self.regs.a ^ n;
        self.regs.set_flag(Z, res == 0);    // Set if result is 0.
        self.regs.set_flag(N, false);       // Reset.
        self.regs.set_flag(H, true);        // Set.
        self.regs.set_flag(C, false);       // Reset.
        self.regs.a = res;
    }

    // CP n - compare A with n, basically alu_sub but results discarded.
    fn alu_cp(&mut self, n: u8) {
        let prev = self.regs.a; 
        self.alu_sub(n, false);
        self.regs.a = prev;
    }

    // INC n - increment register n.
    fn alu_inc(&mut self, n: u8) -> u8 {
        let res = n.wrapping_add(1);
        self.regs.set_flag(Z, res == 0);                // Set if result is 0.
        self.regs.set_flag(N, false);                   // Reset.
        self.regs.set_flag(H, (n&0xF) + 1 > 0xF);       // Set if carry from bit 3.
        res
    }

    // DEC n - decrement register n.
    fn alu_dec(&mut self, n: u8) -> u8 {
        let res = n.wrapping_sub(1);
        self.regs.set_flag(Z, res == 0);        // Set if result is 0.
        self.regs.set_flag(N, true);            // Set.
        self.regs.set_flag(H, (n&0xF) == 0);    // Set if no borrow from bit 4.
        res
    }

    // 16-bit arithmetic.
    // ADD HL, n - add n to hl.
    fn alu_add16(&mut self, n: u16) {
        let hl = self.regs.get_hl();
        let res = hl.wrapping_add(n);
        self.regs.set_flag(N, false);                           // Reset.
        self.regs.set_flag(H, (hl&0x7FF) + (n&0x7FF) > 0x7FF);  // Set if carry from bit 11.
        self.regs.set_flag(C, (hl&0xFF) + (n&0xFF) > 0xFF);     // Set if carry from bit 15.
        self.regs.set_hl(res);
    }

    // SWAP n - swap upper and lower nibles of n.
    fn alu_swap(&mut self, n: u8) -> u8 {
        self.regs.set_flag(C, false);   // Reset.
        self.regs.set_flag(H, false);   // Reset.
        self.regs.set_flag(N, false);   // Reset.
        self.regs.set_flag(Z, n == 0);  // Set if n is 0.
        (n >> 4) | (n << 4)
    }

    // RLC n - rotate n left, old bit 7 to carry flag.
    fn alu_rlc(&mut self, n: u8) -> u8 {
        let carry = (n & 0b10000000) >> 7 == 0x1;
        let rotated = (n << 1) | carry as u8;    // Rotate left.
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, rotated == 0);
        rotated
    }

    // RL n - rotate n left through carry flag.
    fn alu_rl(&mut self, n: u8) -> u8 {
        let carry = (n & 0b10000000) >> 7 == 1;
        let rotated = (n << 1) + self.regs.get_flag(C) as u8;
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, rotated == 0);
        rotated
    }

    // RRC n - rotate n right, old bit 0 to carry flag.
    fn alu_rrc(&mut self, n: u8) -> u8 {
        let carry = n & 1 == 1;
        let rotated = (n >> 1) | (if carry {0b10000000} else {0});
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, rotated == 0);
        rotated
    }

    // RR n - rotate n right through carry flag.
    fn alu_rr(&mut self, n: u8) -> u8 {
        let carry = n & 1 == 1;
        let rotated = (n >> 1) | (if self.regs.get_flag(C) {0x80} else {0});
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, rotated == 0);
        rotated
    }

    // SLA n - shift n left into carry, LSB of n set to 0.
    fn alu_sla(&mut self, n: u8) -> u8 {
        let carry = n & 0b10000000 == 0b10000000;
        let shifted = n << 1;
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, shifted == 0);
        shifted
    }

    // SRA n - shift n right into carry, MSB doesn't change.
    fn alu_sra(&mut self, n: u8) -> u8 {
        let carry = n & 1 == 1;
        let shifted = (n >> 1) | (n & 0b10000000);
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, shifted == 0);
        shifted
    }

    // SRL n - shift n right into carry, MSB set to 0.
    fn alu_srl(&mut self, n: u8) -> u8 {
        let carry = n & 1 == 1; 
        let shifted = n >> 1; 
        self.regs.set_flag(C, carry);
        self.regs.set_flag(H, false);
        self.regs.set_flag(N, false);
        self.regs.set_flag(Z, shifted == 0);
        shifted
    }

    // BIT b, r - test bit b in register r.
    fn alu_bit(&mut self, b: u8, n: u8) {
        let res = n & (1 << b) == 0;
        self.regs.set_flag(Z, res);
        self.regs.set_flag(N, false);
        self.regs.set_flag(H, true);
    }

    // Execute next opcode, returns number of cycles.
    pub fn execute(&mut self, opcode: u8) -> u32 {
        println!("Opcode: {:2X}", opcode);
        let cycles = match opcode {
            // http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
            
            // 8-bit loads 
            // LD nn, n - put value nn into n.
            0x06 => { self.regs.b = self.nxt_byte(); 8 },
            0x0E => { self.regs.c = self.nxt_byte(); 8 },
            0x16 => { self.regs.d = self.nxt_byte(); 8 },
            0x1E => { self.regs.e = self.nxt_byte(); 8 },
            0x26 => { self.regs.h = self.nxt_byte(); 8 },
            0x2E => { self.regs.l = self.nxt_byte(); 8 },
            // LD r1, r2 - put value r2 into r1.
            0x7F => { 4 }, // when r1 == r2, nothing happens.
            0x78 => { self.regs.a = self.regs.b; 4 },
            0x79 => { self.regs.a = self.regs.c; 4 },
            0x7A => { self.regs.a = self.regs.d; 4 },
            0x7B => { self.regs.a = self.regs.e; 4 },
            0x7C => { self.regs.a = self.regs.h; 4 },
            0x7D => { self.regs.a = self.regs.l; 4 },
            0x7E => { self.regs.a = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x40 => { 4 },
            0x41 => { self.regs.b = self.regs.c; 4 },
            0x42 => { self.regs.b = self.regs.d; 4 },
            0x43 => { self.regs.b = self.regs.e; 4 },
            0x44 => { self.regs.b = self.regs.h; 4 },
            0x45 => { self.regs.b = self.regs.l; 4 },
            0x46 => { self.regs.b = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x47 => { self.regs.b = self.regs.a; 4 },
            0x48 => { self.regs.c = self.regs.b; 4 },
            0x49 => { 4 },
            0x4A => { self.regs.c = self.regs.d; 4 },
            0x4B => { self.regs.c = self.regs.e; 4 },
            0x4C => { self.regs.c = self.regs.h; 4 },
            0x4D => { self.regs.c = self.regs.l; 4 },
            0x4E => { self.regs.c = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x4F => { self.regs.c = self.regs.a; 4 },
            0x50 => { self.regs.d = self.regs.b; 4 },
            0x51 => { self.regs.d = self.regs.c; 4 },
            0x52 => { 4 },
            0x53 => { self.regs.d = self.regs.e; 4 },
            0x54 => { self.regs.d = self.regs.h; 4 },
            0x55 => { self.regs.d = self.regs.l; 4 },
            0x56 => { self.regs.d = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x57 => { self.regs.d = self.regs.a; 4 },
            0x58 => { self.regs.e = self.regs.b; 4 },
            0x59 => { self.regs.e = self.regs.c; 4 },
            0x5A => { self.regs.e = self.regs.d; 4 },
            0x5B => { 4 },
            0x5C => { self.regs.e = self.regs.h; 4 },
            0x5D => { self.regs.e = self.regs.l; 4 },
            0x5E => { self.regs.e = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x5F => { self.regs.e = self.regs.a; 4 },
            0x60 => { self.regs.h = self.regs.b; 4 },
            0x61 => { self.regs.h = self.regs.c; 4 },
            0x62 => { self.regs.h = self.regs.d; 4 },
            0x63 => { self.regs.h = self.regs.e; 4 },
            0x64 => { 4 },
            0x65 => { self.regs.h = self.regs.l; 4 },
            0x66 => { self.regs.h = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x67 => { self.regs.h = self.regs.a; 4 },
            0x68 => { self.regs.l = self.regs.b; 4 },
            0x69 => { self.regs.l = self.regs.c; 4 },
            0x6A => { self.regs.l = self.regs.d; 4 },
            0x6B => { self.regs.l = self.regs.e; 4 },
            0x6C => { self.regs.l = self.regs.h; 4 },
            0x6D => { 4 },
            0x6E => { self.regs.l = self.mem.read_byte(self.regs.get_hl()); 8 },
            0x6F => { self.regs.l = self.regs.a; 4 },
            0x70 => { self.mem.write_byte(self.regs.get_hl(), self.regs.b); 8 },
            0x71 => { self.mem.write_byte(self.regs.get_hl(), self.regs.c); 8 },
            0x72 => { self.mem.write_byte(self.regs.get_hl(), self.regs.d); 8 },
            0x73 => { self.mem.write_byte(self.regs.get_hl(), self.regs.e); 8 },
            0x74 => { self.mem.write_byte(self.regs.get_hl(), self.regs.h); 8 },
            0x75 => { self.mem.write_byte(self.regs.get_hl(), self.regs.l); 8 },
            0x36 => {let b = self.nxt_byte(); self.mem.write_byte(self.regs.get_hl(), b); 12 },
            
            // LD nn, A - put value A into nn.
            0x02 => { self.mem.write_byte(self.regs.get_bc(), self.regs.a); 8 },
            0x12 => { self.mem.write_byte(self.regs.get_de(), self.regs.a); 8 },
            0x77 => { self.mem.write_byte(self.regs.get_hl(), self.regs.a); 8 },
            0xEA => { let nn = self.nxt_word(); self.mem.write_byte(nn, self.regs.a); 16 },
            
            // LD A, (C) - put value at address $FF00 + reg C into A.
            0xF2 => { self.regs.a = self.mem.read_byte(0xFF00 | self.regs.c as u16); 8 }, 
            // LD (C), A - put A into address $FF00 + reg C.
            0xE2 => { self.mem.write_byte(0xFF00 | self.regs.c as u16, self.regs.a); 8 },
           
            // LD A, (HLD) - put value at address hl into A, decrement hl.
            0x3A => {
                self.regs.a = self.mem.read_byte(self.regs.get_hl());
                self.regs.set_hl(self.regs.get_hl() - 1);
                8
            },
            // LD (HLD), A - put A into memory at address hl, decrement hl.
            0x32 => {
                self.mem.write_byte(self.regs.get_hl(), self.regs.a);
                self.regs.set_hl(self.regs.get_hl() - 1);
                8
            },
            // LD A, (HLI) - put value at address hl into A, increment hl.
            0x2A => {
                self.regs.a = self.mem.read_byte(self.regs.get_hl());
                self.regs.set_hl(self.regs.get_hl() + 1);
                8
            },
            // LD (HLI), A - put A into memory address hl, increment hl.
            0x22 => {
                self.mem.write_byte(self.regs.get_hl(), self.regs.a);
                self.regs.set_hl(self.regs.get_hl() + 1);
                8
            },

            // LDH (n), A - put A into memory address $FF00+n.
            0xE0 => { let a = self.nxt_byte(); self.mem.write_byte(0xFF00 | a as u16, self.regs.a); 12 },
            // LDH A, (n) - put memory address $FF00+n into A.
            0xF0 => { let b = self.nxt_byte(); self.regs.a = self.mem.read_byte(0xFF00 | b as u16); 12 },
            
            // 16-bit loads.
            // LD n, nn - put value nn into n.
            0x01 => { let nn = self.nxt_word(); self.regs.set_bc(nn); 12 },
            0x11 => { let nn = self.nxt_word(); self.regs.set_de(nn); 12 },
            0x21 => { let nn = self.nxt_word(); self.regs.set_hl(nn); 12 },
            0x31 => { self.regs.sp = self.nxt_word(); 12 },
            // LD SP, HL - put hl into stack pointer.
            0xF9 => { self.regs.sp = self.regs.get_hl(); 8 },
            // LDHL SP, n - put SP + n effective address into hl.
            0xF8 => {
                let b = self.nxt_byte() as u16;
                let sp = self.regs.sp;
                self.regs.set_flag(Z, false);
                self.regs.set_flag(N, false);
                self.regs.set_flag(H, (sp & 0x000F) + (b & 0x000F) > 0x000F);
                self.regs.set_flag(C, (sp & 0x00FF) + (b & 0x00FF) > 0x00FF);
                self.regs.set_hl(b + sp); 
                12
            },
            // LD (nn), SP - put stack pointer at address n.
            0x08 => { let n = self.nxt_word(); self.mem.write_word(n, self.regs.sp);  20},
            // PUSH nn - push register pair onto stack, decrement stack pointer twice.
            0xF5 => { self.stack_push(self.regs.get_af()); 16 },
            0xC5 => { self.stack_push(self.regs.get_bc()); 16 },
            0xD5 => { self.stack_push(self.regs.get_de()); 16 },
            0xE5 => { self.stack_push(self.regs.get_hl()); 16 },
            // POP nn - pop two bytes off stack into register pair nn, increment stack pointer twice.
            0xF1 => { let nn = self.stack_pop(); self.regs.set_af(nn); 12 },
            0xC1 => { let nn = self.stack_pop(); self.regs.set_bc(nn); 12 },
            0xD1 => { let nn = self.stack_pop(); self.regs.set_de(nn); 12 },
            0xE1 => { let nn = self.stack_pop(); self.regs.set_hl(nn); 12 },

            // 8-bit ALU
            // Addition.
            0x87 => { self.alu_add(self.regs.a, false); 4 },
            0x80 => { self.alu_add(self.regs.b, false); 4 },
            0x81 => { self.alu_add(self.regs.c, false); 4 },
            0x82 => { self.alu_add(self.regs.d, false); 4 },
            0x83 => { self.alu_add(self.regs.e, false); 4 },
            0x84 => { self.alu_add(self.regs.h, false); 4 },
            0x85 => { self.alu_add(self.regs.l, false); 4 },
            0x86 => { self.alu_add(self.mem.read_byte(self.regs.get_hl()), false); 8 },
            0xC6 => { let b = self.nxt_byte(); self.alu_add(b, false); 8 },
            // Addition with carry.
            0x8F => { self.alu_add(self.regs.a, true); 4 },
            0x88 => { self.alu_add(self.regs.b, true); 4 },
            0x89 => { self.alu_add(self.regs.c, true); 4 },
            0x8A => { self.alu_add(self.regs.d, true); 4 },
            0x8B => { self.alu_add(self.regs.e, true); 4 },
            0x8C => { self.alu_add(self.regs.h, true); 4 },
            0x8D => { self.alu_add(self.regs.l, true); 4 },
            0x8E => { self.alu_add(self.mem.read_byte(self.regs.get_hl()), true); 8 },
            0xCE => { let b = self.nxt_byte(); self.alu_add(b, true); 8 },
            // Subtraction.
            0x97 => { self.alu_sub(self.regs.a, false); 4 },
            0x90 => { self.alu_sub(self.regs.b, false); 4 },
            0x91 => { self.alu_sub(self.regs.c, false); 4 },
            0x92 => { self.alu_sub(self.regs.d, false); 4 },
            0x93 => { self.alu_sub(self.regs.e, false); 4 },
            0x94 => { self.alu_sub(self.regs.h, false); 4 },
            0x95 => { self.alu_sub(self.regs.l, false); 4 },
            0x96 => { self.alu_sub(self.mem.read_byte(self.regs.get_hl()), false); 8 },
            0xD6 => { let b = self.nxt_byte(); self.alu_sub(b, false); 8 },
            // Subtraction with carry.
            0x9F => { self.alu_sub(self.regs.a, true); 4 },
            0x98 => { self.alu_sub(self.regs.b, true); 4 },
            0x99 => { self.alu_sub(self.regs.c, true); 4 },
            0x9A => { self.alu_sub(self.regs.d, true); 4 },
            0x9B => { self.alu_sub(self.regs.e, true); 4 },
            0x9C => { self.alu_sub(self.regs.h, true); 4 },
            0x9D => { self.alu_sub(self.regs.l, true); 4 },
            0x9E => { self.alu_sub(self.mem.read_byte(self.regs.get_hl()), true); 8 },
            // AND
            0xA7 => { self.alu_and(self.regs.a); 4 },
            0xA0 => { self.alu_and(self.regs.b); 4 },
            0xA1 => { self.alu_and(self.regs.c); 4 },
            0xA2 => { self.alu_and(self.regs.d); 4 },
            0xA3 => { self.alu_and(self.regs.e); 4 },
            0xA4 => { self.alu_and(self.regs.h); 4 },
            0xA5 => { self.alu_and(self.regs.l); 4 },
            0xA6 => { self.alu_and(self.mem.read_byte(self.regs.get_hl())); 8 },
            0xE6 => { let b = self.nxt_byte(); self.alu_and(b); 8 },
            // OR
            0xB7 => { self.alu_or(self.regs.a); 4 },
            0xB0 => { self.alu_or(self.regs.b); 4 },
            0xB1 => { self.alu_or(self.regs.c); 4 },
            0xB2 => { self.alu_or(self.regs.d); 4 },
            0xB3 => { self.alu_or(self.regs.e); 4 },
            0xB4 => { self.alu_or(self.regs.h); 4 },
            0xB5 => { self.alu_or(self.regs.l); 4 },
            0xB6 => { self.alu_or(self.mem.read_byte(self.regs.get_hl())); 8 },
            0xF6 => { let b = self.nxt_byte(); self.alu_or(b); 8 },
            // XOR
            0xAF => { self.alu_xor(self.regs.a); 4 },
            0xA8 => { self.alu_xor(self.regs.b); 4 },
            0xA9 => { self.alu_xor(self.regs.c); 4 },
            0xAA => { self.alu_xor(self.regs.d); 4 },
            0xAB => { self.alu_xor(self.regs.e); 4 },
            0xAC => { self.alu_xor(self.regs.h); 4 },
            0xAD => { self.alu_xor(self.regs.l); 4 },
            0xAE => { self.alu_xor(self.mem.read_byte(self.regs.get_hl())); 8 },
            0xEE => { let b = self.nxt_byte(); self.alu_xor(b); 8 },
            // CP
            0xBF => { self.alu_cp(self.regs.a); 4 },
            0xB8 => { self.alu_cp(self.regs.b); 4 },
            0xB9 => { self.alu_cp(self.regs.c); 4 },
            0xBA => { self.alu_cp(self.regs.d); 4 },
            0xBB => { self.alu_cp(self.regs.e); 4 },
            0xBC => { self.alu_cp(self.regs.h); 4 },
            0xBD => { self.alu_cp(self.regs.l); 4 },
            0xBE => { self.alu_cp(self.mem.read_byte(self.regs.get_hl())); 8 },
            0xFE => { let b = self.nxt_byte(); self.alu_cp(b); 8 },
            // Increments
            0x3C => { self.regs.a = self.alu_inc(self.regs.a); 4 },
            0x04 => { self.regs.b = self.alu_inc(self.regs.b); 4 },
            0x0C => { self.regs.c = self.alu_inc(self.regs.c); 4 },
            0x14 => { self.regs.d = self.alu_inc(self.regs.d); 4 },
            0x1C => { self.regs.e = self.alu_inc(self.regs.e); 4 },
            0x24 => { self.regs.h = self.alu_inc(self.regs.h); 4 },
            0x2C => { self.regs.l = self.alu_inc(self.regs.l); 4 },
            0x34 => { 
                let val = self.alu_inc(self.mem.read_byte(self.regs.get_hl()));
                self.mem.write_byte(self.regs.get_hl(), val); 
                12 
            },
            // Decrements
            0x3D => { self.alu_dec(self.regs.a); 4 },
            0x05 => { self.alu_dec(self.regs.b); 4 },
            0x0D => { self.alu_dec(self.regs.c); 4 },
            0x15 => { self.alu_dec(self.regs.d); 4 },
            0x1D => { self.alu_dec(self.regs.e); 4 },
            0x25 => { self.alu_dec(self.regs.h); 4 },
            0x2D => { self.alu_dec(self.regs.l); 4 },
            0x35 => { 
                let val = self.alu_dec(self.mem.read_byte(self.regs.get_hl()));
                self.mem.write_byte(self.regs.get_hl(), val); 
                12 
            },

            // 16-bit arithmetic.
            // Addition
            0x09 => { self.alu_add16(self.regs.get_bc()); 8 },
            0x19 => { self.alu_add16(self.regs.get_de()); 8 },
            0x29 => { self.alu_add16(self.regs.get_hl()); 8 },
            0x39 => { self.alu_add16(self.regs.sp); 8 },
            // ADD SP, n - add n to stack pointer.
            0xE8 => {
                let b = self.nxt_byte() as u16;
                let sp = self.regs.sp;
                self.regs.set_flag(Z, false);   // Reset.
                self.regs.set_flag(N, false);   // Reset.
                self.regs.set_flag(H, (sp&0xF) + (b&0xF) > 0xF);
                self.regs.set_flag(C, (sp&0xFF) + (b&0xFF) > 0xFF);     
                self.regs.sp = self.regs.sp.wrapping_add(b); 
                16
            },
            // INC nn - increment register nn.
            0x03 => { self.regs.set_bc(self.regs.get_bc().wrapping_add(1)); 8 },
            0x13 => { self.regs.set_de(self.regs.get_de().wrapping_add(1)); 8 },
            0x23 => { self.regs.set_hl(self.regs.get_hl().wrapping_add(1)); 8 },
            0x33 => { self.regs.sp = self.regs.sp.wrapping_add(1); 8 },
            // DEC nn - decrement register nn.
            0x0B => { self.regs.set_bc(self.regs.get_bc().wrapping_sub(1)); 8 },
            0x1B => { self.regs.set_de(self.regs.get_de().wrapping_sub(1)); 8 },
            0x2B => { self.regs.set_hl(self.regs.get_hl().wrapping_sub(1)); 8 },
            0x3B => { self.regs.sp = self.regs.sp.wrapping_sub(1); 8 },

            // Misc.
            // CPL - complement A register (flip all bits).
            0x2F => {
                self.regs.a = !self.regs.a;
                self.regs.set_flag(H, true);
                self.regs.set_flag(N, true);
                4
            },
            
            // CCF - complement carry flag.
            0x3F => {
                self.regs.set_flag(C, !self.regs.get_flag(C));
                self.regs.set_flag(N, false);
                self.regs.set_flag(H, false);
                4
            },

            // SCF - set carry flag.
            0x37 => {
                self.regs.set_flag(C, true);
                self.regs.set_flag(N, false);
                self.regs.set_flag(H, false);
                4
            },

            // NOP - no instruction.
            0x00 => { 4 },
            // HALT - power down CPU until interrupt occers. For energy conservation.
            0x76 => { self.halted = true; 4 },
            // STOP - halt CPU and LCD display until button pressed.
            0x10 => {todo!()},

            // DI - interupts disabled after instruciton after DI is executed.
            0xF3 => { self.ei = false; 4 },
            // EI - interrupts are enabled after instruction after EI is excuted.
            0xFB => { self.ei = true; 4 },

            // Rotates and shifts for register A. // MAYBE SET Z FLAG AFTER.
            0x07 => { self.regs.a = self.alu_rlc(self.regs.a); 4 },
            0x17 => { self.regs.a = self.alu_rl(self.regs.a); 4 },
            0x0F => { self.regs.a = self.alu_rrc(self.regs.a); 4 },
            0x1F => { self.regs.a = self.alu_rr(self.regs.a); 4 },

             // DAA - decimal adjust register a.
            0x27 => {
                let mut a = self.regs.a;
                let mut correction = if self.regs.get_flag(C) { 0x60 } else { 0 };
                if self.regs.get_flag(H) {
                    correction |= 0x06;
                }
                if !self.regs.get_flag(N) {
                    if a & 0x0F > 0x09 {
                        correction |= 0x06;
                    }
                    if a > 0x99 {
                        correction |= 0x60;
                    }
                    a = a.wrapping_add(correction);
                } else {
                    a = a.wrapping_sub(correction);
                }

                self.regs.set_flag(C, correction >= 0x60);
                self.regs.set_flag(H, false);
                self.regs.set_flag(Z, a == 0);
                self.regs.a = a;
                4
            },

            // Jumps - jump to address if condition is true.
            0xC3 => { self.regs.pc = self.nxt_word(); 16 },
            0xC2 => { let addr = self.nxt_word(); if !self.regs.get_flag(Z) { self.regs.pc = addr; 16 } else { 12 }},
            0xCA => { let addr = self.nxt_word(); if self.regs.get_flag(Z) { self.regs.pc = addr; 16 } else { 12 }},
            0xD2 => { let addr = self.nxt_word(); if !self.regs.get_flag(C) { self.regs.pc = addr; 16 } else { 12 }},
            0xDA => { let addr = self.nxt_word(); if self.regs.get_flag(C) { self.regs.pc = addr; 16 } else { 12 }},
            // JP (HL) jump to address contained in hl.
            0xE9 => { self.regs.pc = self.regs.get_hl(); 4 },
            // JR n - add n to current address and jump to it.
            0x18 => { self.regs.pc += self.nxt_byte() as u16; 8 },
            // JR cc, n - if condition true, add n to current address and jump to it.
            0x20 => { 
                let addr = self.regs.pc + self.nxt_byte() as u16;  
                if !self.regs.get_flag(Z) { self.regs.pc = addr; 12 } else { 8 }
            },
            0x28 => {
                let addr = self.regs.pc + self.nxt_byte() as u16;
                if self.regs.get_flag(Z) { self.regs.pc = addr; 12 } else { 8 }
            },
            0x30 => {
                let addr = self.regs.pc + self.nxt_byte() as u16;
                if !self.regs.get_flag(C) { self.regs.pc = addr; 12 } else { 8 } 
            },
            0x38 => {
                let addr = self.regs.pc + self.nxt_byte() as u16;
                if self.regs.get_flag(C) { self.regs.pc = addr; 12 } else { 8 }
            },

            // Calls
            // CALL nn - push address of next instruction onto stack and then jump to address nn.
            0xCD => { let addr = self.nxt_word(); self.stack_push(self.regs.pc); self.regs.pc = addr; 12 },
            // CALL cc, nn - call address n if following condition is true.
            0xC4 => {
                let addr = self.nxt_word();
                if !self.regs.get_flag(Z) {
                    self.stack_push(self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else { 12 }
            },
            0xCC => {
                let addr = self.nxt_word();
                if self.regs.get_flag(Z) {
                    self.stack_push(self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else { 12 }
            },
            0xD4 => {
                let addr = self.nxt_word();
                if !self.regs.get_flag(C) {
                    self.stack_push(self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else { 12 }
            },
            0xDC => {
                let addr = self.nxt_word();
                if self.regs.get_flag(C) {
                    self.stack_push(self.regs.pc);
                    self.regs.pc = addr;
                    24
                } else { 12 } 
            },
            
            // Restarts
            // RST n - push present address onto stack, jump to address $0000 + n.
            0xC7 => { self.stack_push(self.regs.pc); self.regs.pc = 0x00; 32 },
            0xCF => { self.stack_push(self.regs.pc); self.regs.pc = 0x08; 32 },
            0xD7 => { self.stack_push(self.regs.pc); self.regs.pc = 0x10; 32 },
            0xDF => { self.stack_push(self.regs.pc); self.regs.pc = 0x18; 32 },
            0xE7 => { self.stack_push(self.regs.pc); self.regs.pc = 0x20; 32 },
            0xEF => { self.stack_push(self.regs.pc); self.regs.pc = 0x28; 32 },
            0xF7 => { self.stack_push(self.regs.pc); self.regs.pc = 0x30; 32 },
            0xFF => { self.stack_push(self.regs.pc); self.regs.pc = 0x38; 32 },

            // Returns
            // RET - pop two bytes from stack and jump to that address.
            0xC9 => { self.regs.pc = self.stack_pop(); 8 },
            0xC0 => { if !self.regs.get_flag(Z) { self.regs.pc = self.stack_pop(); 20 } else { 8 }},
            0xC8 => { if self.regs.get_flag(Z) { self.regs.pc = self.stack_pop(); 20 } else { 8 }},
            0xD0 => { if !self.regs.get_flag(C) { self.regs.pc = self.stack_pop(); 20 } else { 8 }},
            0xD8 => { if self.regs.get_flag(C) { self.regs.pc = self.stack_pop(); 20 } else { 8 }},
            // RETI - pop two bytes from stack and jump to that address then enables interrupts.
            0xD9 => { self.regs.pc = self.nxt_word(); self.ei = true; 8 },

            0xCB => {   // Instruction set extension.
                let cb_opcode = self.nxt_byte();
                match cb_opcode {

                    // Rotates and shifts.
                    0x07 => { self.regs.a = self.alu_rlc(self.regs.a); 8 },
                    0x00 => { self.regs.b = self.alu_rlc(self.regs.b); 8 },
                    0x01 => { self.regs.c = self.alu_rlc(self.regs.c); 8 },
                    0x02 => { self.regs.d = self.alu_rlc(self.regs.d); 8 },
                    0x03 => { self.regs.e = self.alu_rlc(self.regs.e); 8 },
                    0x04 => { self.regs.h = self.alu_rlc(self.regs.h); 8 },
                    0x05 => { self.regs.l = self.alu_rlc(self.regs.l); 8 },
                    0x06 => {
                        let val = self.alu_rlc(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0x17 => { self.regs.a = self.alu_rl(self.regs.a); 8 },
                    0x10 => { self.regs.b = self.alu_rl(self.regs.b); 8 },
                    0x11 => { self.regs.c = self.alu_rl(self.regs.c); 8 },
                    0x12 => { self.regs.d = self.alu_rl(self.regs.d); 8 },
                    0x13 => { self.regs.e = self.alu_rl(self.regs.e); 8 },
                    0x14 => { self.regs.h = self.alu_rl(self.regs.h); 8 },
                    0x15 => { self.regs.l = self.alu_rl(self.regs.l); 8 },
                    0x16 => {
                        let val = self.alu_rl(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0x0F => { self.regs.a = self.alu_rrc(self.regs.a); 8 },
                    0x08 => { self.regs.b = self.alu_rrc(self.regs.b); 8 },
                    0x09 => { self.regs.c = self.alu_rrc(self.regs.c); 8 },
                    0x0A => { self.regs.d = self.alu_rrc(self.regs.d); 8 },
                    0x0B => { self.regs.e = self.alu_rrc(self.regs.e); 8 },
                    0x0C => { self.regs.h = self.alu_rrc(self.regs.h); 8 },
                    0x0D => { self.regs.l = self.alu_rrc(self.regs.l); 8 },
                    0x0E => {
                        let val = self.alu_rrc(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0x1F => { self.regs.a = self.alu_rr(self.regs.a); 8 },
                    0x18 => { self.regs.b = self.alu_rr(self.regs.b); 8 },
                    0x19 => { self.regs.c = self.alu_rr(self.regs.c); 8 },
                    0x1A => { self.regs.d = self.alu_rr(self.regs.d); 8 },
                    0x1B => { self.regs.e = self.alu_rr(self.regs.e); 8 },
                    0x1C => { self.regs.h = self.alu_rr(self.regs.h); 8 },
                    0x1D => { self.regs.l = self.alu_rr(self.regs.l); 8 },
                    0x1E => {
                        let val = self.alu_rr(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },
                    
                    // Shifts
                    0x27 => { self.regs.a = self.alu_sla(self.regs.a); 8 },
                    0x20 => { self.regs.b = self.alu_sla(self.regs.b); 8 },
                    0x21 => { self.regs.c = self.alu_sla(self.regs.c); 8 },
                    0x22 => { self.regs.d = self.alu_sla(self.regs.d); 8 },
                    0x23 => { self.regs.e = self.alu_sla(self.regs.e); 8 },
                    0x24 => { self.regs.h = self.alu_sla(self.regs.h); 8 },
                    0x25 => { self.regs.l = self.alu_sla(self.regs.l); 8 },
                    0x26 => {
                        let val = self.alu_sla(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0x2F => { self.regs.a = self.alu_sra(self.regs.a); 8 },
                    0x28 => { self.regs.b = self.alu_sra(self.regs.b); 8 },
                    0x29 => { self.regs.c = self.alu_sra(self.regs.c); 8 },
                    0x2A => { self.regs.d = self.alu_sra(self.regs.d); 8 },
                    0x2B => { self.regs.e = self.alu_sra(self.regs.e); 8 },
                    0x2C => { self.regs.h = self.alu_sra(self.regs.h); 8 },
                    0x2D => { self.regs.l = self.alu_sra(self.regs.l); 8 },
                    0x2E => {
                        let val = self.alu_sra(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0x3F => { self.regs.a = self.alu_srl(self.regs.a); 8 },
                    0x38 => { self.regs.b = self.alu_srl(self.regs.b); 8 },
                    0x39 => { self.regs.c = self.alu_srl(self.regs.c); 8 },
                    0x3A => { self.regs.d = self.alu_srl(self.regs.d); 8 },
                    0x3B => { self.regs.e = self.alu_srl(self.regs.e); 8 },
                    0x3C => { self.regs.h = self.alu_srl(self.regs.h); 8 },
                    0x3D => { self.regs.l = self.alu_srl(self.regs.l); 8 },
                    0x3E => {
                        let val = self.alu_sla(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    // Bit opcodes
                    0x47 => { self.alu_bit(0, self.regs.a); 8 },
                    0x40 => { self.alu_bit(0, self.regs.b); 8 },
                    0x41 => { self.alu_bit(0, self.regs.c); 8 },
                    0x42 => { self.alu_bit(0, self.regs.d); 8 },
                    0x43 => { self.alu_bit(0, self.regs.e); 8 },
                    0x44 => { self.alu_bit(0, self.regs.h); 8 },
                    0x45 => { self.alu_bit(0, self.regs.l); 8 },
                    0x46 => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(0, a); 16 },

                    0x4F => { self.alu_bit(1, self.regs.a); 8 },
                    0x48 => { self.alu_bit(1, self.regs.b); 8 },
                    0x49 => { self.alu_bit(1, self.regs.c); 8 },
                    0x4A => { self.alu_bit(1, self.regs.d); 8 },
                    0x4B => { self.alu_bit(1, self.regs.e); 8 },
                    0x4C => { self.alu_bit(1, self.regs.h); 8 },
                    0x4D => { self.alu_bit(1, self.regs.l); 8 },
                    0x4E => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(1, a); 16 },

                    0x57 => { self.alu_bit(2, self.regs.a); 8 },
                    0x50 => { self.alu_bit(2, self.regs.b); 8 },
                    0x51 => { self.alu_bit(2, self.regs.c); 8 },
                    0x52 => { self.alu_bit(2, self.regs.d); 8 },
                    0x53 => { self.alu_bit(2, self.regs.e); 8 },
                    0x54 => { self.alu_bit(2, self.regs.h); 8 },
                    0x55 => { self.alu_bit(2, self.regs.l); 8 },
                    0x56 => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(2, a); 16 },

                    0x5F => { self.alu_bit(3, self.regs.a); 8 },
                    0x58 => { self.alu_bit(3, self.regs.b); 8 },
                    0x59 => { self.alu_bit(3, self.regs.c); 8 },
                    0x5A => { self.alu_bit(3, self.regs.d); 8 },
                    0x5B => { self.alu_bit(3, self.regs.e); 8 },
                    0x5C => { self.alu_bit(3, self.regs.h); 8 },
                    0x5D => { self.alu_bit(3, self.regs.l); 8 },
                    0x5E => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(3, a); 16 },

                    0x67 => { self.alu_bit(4, self.regs.a); 8 },
                    0x60 => { self.alu_bit(4, self.regs.b); 8 },
                    0x61 => { self.alu_bit(4, self.regs.c); 8 },
                    0x62 => { self.alu_bit(4, self.regs.d); 8 },
                    0x63 => { self.alu_bit(4, self.regs.e); 8 },
                    0x64 => { self.alu_bit(4, self.regs.h); 8 },
                    0x65 => { self.alu_bit(4, self.regs.l); 8 },
                    0x66 => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(4, a); 16 },

                    0x6F => { self.alu_bit(5, self.regs.a); 8 },
                    0x68 => { self.alu_bit(5, self.regs.b); 8 },
                    0x69 => { self.alu_bit(5, self.regs.c); 8 },
                    0x6A => { self.alu_bit(5, self.regs.d); 8 },
                    0x6B => { self.alu_bit(5, self.regs.e); 8 },
                    0x6C => { self.alu_bit(5, self.regs.h); 8 },
                    0x6D => { self.alu_bit(5, self.regs.l); 8 },
                    0x6E => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(5, a); 16 },

                    0x77 => { self.alu_bit(6, self.regs.a); 8 },
                    0x70 => { self.alu_bit(6, self.regs.b); 8 },
                    0x71 => { self.alu_bit(6, self.regs.c); 8 },
                    0x72 => { self.alu_bit(6, self.regs.d); 8 },
                    0x73 => { self.alu_bit(6, self.regs.e); 8 },
                    0x74 => { self.alu_bit(6, self.regs.h); 8 },
                    0x75 => { self.alu_bit(6, self.regs.l); 8 },
                    0x76 => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(6, a); 16 },

                    0x7F => { self.alu_bit(7, self.regs.a); 8 },
                    0x78 => { self.alu_bit(7, self.regs.b); 8 },
                    0x79 => { self.alu_bit(7, self.regs.c); 8 },
                    0x7A => { self.alu_bit(7, self.regs.d); 8 },
                    0x7B => { self.alu_bit(7, self.regs.e); 8 },
                    0x7C => { self.alu_bit(7, self.regs.h); 8 },
                    0x7D => { self.alu_bit(7, self.regs.l); 8 },
                    0x7E => { let a = self.mem.read_byte(self.regs.get_hl()); self.alu_bit(7, a); 16 },
                    
                    // SET b, r - set bit b in register r.
                    0xC7 => { self.regs.a |= 1 << 0; 8 },
                    0xC0 => { self.regs.b |= 1 << 0; 8 },
                    0xC1 => { self.regs.c |= 1 << 0; 8 },
                    0xC2 => { self.regs.d |= 1 << 0; 8 },
                    0xC3 => { self.regs.e |= 1 << 0; 8 },
                    0xC4 => { self.regs.h |= 1 << 0; 8 },
                    0xC5 => { self.regs.l |= 1 << 0; 8 },
                    0xC6 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 0);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xCF => { self.regs.a |= 1 << 1; 8 },
                    0xC8 => { self.regs.b |= 1 << 1; 8 },
                    0xC9 => { self.regs.c |= 1 << 1; 8 },
                    0xCA => { self.regs.d |= 1 << 1; 8 },
                    0xCB => { self.regs.e |= 1 << 1; 8 },
                    0xCC => { self.regs.h |= 1 << 1; 8 },
                    0xCD => { self.regs.l |= 1 << 1; 8 },
                    0xCE => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 1);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },
                    
                    0xD7 => { self.regs.a |= 1 << 2; 8 },
                    0xD0 => { self.regs.b |= 1 << 2; 8 },
                    0xD1 => { self.regs.c |= 1 << 2; 8 },
                    0xD2 => { self.regs.d |= 1 << 2; 8 },
                    0xD3 => { self.regs.e |= 1 << 2; 8 },
                    0xD4 => { self.regs.h |= 1 << 2; 8 },
                    0xD5 => { self.regs.l |= 1 << 2; 8 },
                    0xD6 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 2);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },
                    
                    0xDF => { self.regs.a |= 1 << 3; 8 },
                    0xD8 => { self.regs.b |= 1 << 3; 8 },
                    0xD9 => { self.regs.c |= 1 << 3; 8 },
                    0xDA => { self.regs.d |= 1 << 3; 8 },
                    0xDB => { self.regs.e |= 1 << 3; 8 },
                    0xDC => { self.regs.h |= 1 << 3; 8 },
                    0xDD => { self.regs.l |= 1 << 3; 8 },
                    0xDE => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 3);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xE7 => { self.regs.a |= 1 << 4; 8 },
                    0xE0 => { self.regs.b |= 1 << 4; 8 },
                    0xE1 => { self.regs.c |= 1 << 4; 8 },
                    0xE2 => { self.regs.d |= 1 << 4; 8 },
                    0xE3 => { self.regs.e |= 1 << 4; 8 },
                    0xE4 => { self.regs.h |= 1 << 4; 8 },
                    0xE5 => { self.regs.l |= 1 << 4; 8 },
                    0xE6 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 4);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xEF => { self.regs.a |= 1 << 5; 8 },
                    0xE8 => { self.regs.b |= 1 << 5; 8 },
                    0xE9 => { self.regs.c |= 1 << 5; 8 },
                    0xEA => { self.regs.d |= 1 << 5; 8 },
                    0xEB => { self.regs.e |= 1 << 5; 8 },
                    0xEC => { self.regs.h |= 1 << 5; 8 },
                    0xED => { self.regs.l |= 1 << 5; 8 },
                    0xEE => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 5);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xF7 => { self.regs.a |= 1 << 6; 8 },
                    0xF0 => { self.regs.b |= 1 << 6; 8 },
                    0xF1 => { self.regs.c |= 1 << 6; 8 },
                    0xF2 => { self.regs.d |= 1 << 6; 8 },
                    0xF3 => { self.regs.e |= 1 << 6; 8 },
                    0xF4 => { self.regs.h |= 1 << 6; 8 },
                    0xF5 => { self.regs.l |= 1 << 6; 8 },
                    0xF6 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 6);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xFF => { self.regs.a |= 1 << 7; 8 },
                    0xF8 => { self.regs.b |= 1 << 7; 8 },
                    0xF9 => { self.regs.c |= 1 << 7; 8 },
                    0xFA => { self.regs.d |= 1 << 7; 8 },
                    0xFB => { self.regs.e |= 1 << 7; 8 },
                    0xFC => { self.regs.h |= 1 << 7; 8 },
                    0xFD => { self.regs.l |= 1 << 7; 8 },
                    0xFE => {
                        let val = self.mem.read_byte(self.regs.get_hl()) | (1 << 7);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    // RES b, r - reset bit b in register r.
                    0x87 => { self.regs.a &= !(1 << 0); 8 },
                    0x80 => { self.regs.b &= !(1 << 0); 8 },
                    0x81 => { self.regs.c &= !(1 << 0); 8 },
                    0x82 => { self.regs.d &= !(1 << 0); 8 },
                    0x83 => { self.regs.e &= !(1 << 0); 8 },
                    0x84 => { self.regs.h &= !(1 << 0); 8 },
                    0x85 => { self.regs.l &= !(1 << 0); 8 },
                    0x86 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 0);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0x8F => { self.regs.a &= !(1 << 1); 8 },
                    0x88 => { self.regs.b &= !(1 << 1); 8 },
                    0x89 => { self.regs.c &= !(1 << 1); 8 },
                    0x8A => { self.regs.d &= !(1 << 1); 8 },
                    0x8B => { self.regs.e &= !(1 << 1); 8 },
                    0x8C => { self.regs.h &= !(1 << 1); 8 },
                    0x8D => { self.regs.l &= !(1 << 1); 8 },
                    0x8E => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 1);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },
                    
                    0x97 => { self.regs.a &= !(1 << 2); 8 },
                    0x90 => { self.regs.b &= !(1 << 2); 8 },
                    0x91 => { self.regs.c &= !(1 << 2); 8 },
                    0x92 => { self.regs.d &= !(1 << 2); 8 },
                    0x93 => { self.regs.e &= !(1 << 2); 8 },
                    0x94 => { self.regs.h &= !(1 << 2); 8 },
                    0x95 => { self.regs.l &= !(1 << 2); 8 },
                    0x96 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 2);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },
                    
                    0x9F => { self.regs.a &= !(1 << 3); 8 },
                    0x98 => { self.regs.b &= !(1 << 3); 8 },
                    0x99 => { self.regs.c &= !(1 << 3); 8 },
                    0x9A => { self.regs.d &= !(1 << 3); 8 },
                    0x9B => { self.regs.e &= !(1 << 3); 8 },
                    0x9C => { self.regs.h &= !(1 << 3); 8 },
                    0x9D => { self.regs.l &= !(1 << 3); 8 },
                    0x9E => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 3);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xA7 => { self.regs.a &= !(1 << 4); 8 },
                    0xA0 => { self.regs.b &= !(1 << 4); 8 },
                    0xA1 => { self.regs.c &= !(1 << 4); 8 },
                    0xA2 => { self.regs.d &= !(1 << 4); 8 },
                    0xA3 => { self.regs.e &= !(1 << 4); 8 },
                    0xA4 => { self.regs.h &= !(1 << 4); 8 },
                    0xA5 => { self.regs.l &= !(1 << 4); 8 },
                    0xA6 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 4);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xAF => { self.regs.a &= !(1 << 5); 8 },
                    0xA8 => { self.regs.b &= !(1 << 5); 8 },
                    0xA9 => { self.regs.c &= !(1 << 5); 8 },
                    0xAA => { self.regs.d &= !(1 << 5); 8 },
                    0xAB => { self.regs.e &= !(1 << 5); 8 },
                    0xAC => { self.regs.h &= !(1 << 5); 8 },
                    0xAD => { self.regs.l &= !(1 << 5); 8 },
                    0xAE => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 5);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xB7 => { self.regs.a &= !(1 << 6); 8 },
                    0xB0 => { self.regs.b &= !(1 << 6); 8 },
                    0xB1 => { self.regs.c &= !(1 << 6); 8 },
                    0xB2 => { self.regs.d &= !(1 << 6); 8 },
                    0xB3 => { self.regs.e &= !(1 << 6); 8 },
                    0xB4 => { self.regs.h &= !(1 << 6); 8 },
                    0xB5 => { self.regs.l &= !(1 << 6); 8 },
                    0xB6 => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 6);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    0xBF => { self.regs.a &= !(1 << 7); 8 },
                    0xB8 => { self.regs.b &= !(1 << 7); 8 },
                    0xB9 => { self.regs.c &= !(1 << 7); 8 },
                    0xBA => { self.regs.d &= !(1 << 7); 8 },
                    0xBB => { self.regs.e &= !(1 << 7); 8 },
                    0xBC => { self.regs.h &= !(1 << 7); 8 },
                    0xBD => { self.regs.l &= !(1 << 7); 8 },
                    0xBE => {
                        let val = self.mem.read_byte(self.regs.get_hl()) & !(1 << 7);
                        self.mem.write_byte(self.regs.get_hl(), val);
                        16
                    },

                    // Swaps
                    0x37 => { self.regs.a = self.alu_swap(self.regs.a); 8 },
                    0x30 => { self.regs.b = self.alu_swap(self.regs.b); 8 },
                    0x31 => { self.regs.c = self.alu_swap(self.regs.c); 8 },
                    0x32 => { self.regs.d = self.alu_swap(self.regs.d); 8 },
                    0x33 => { self.regs.e = self.alu_swap(self.regs.e); 8 },
                    0x34 => { self.regs.h = self.alu_swap(self.regs.h); 8 },
                    0x35 => { self.regs.l = self.alu_swap(self.regs.l); 8 },
                    0x36 => { 
                        let val = self.alu_swap(self.mem.read_byte(self.regs.get_hl()));
                        self.mem.write_byte(self.regs.get_hl(), val); 
                        16
                    },
                }
            }
            e => panic!("unsuppored opcode: {:#2X}", e)
        };
        println!("Execution cycles: {}", cycles);
        cycles
    }
}