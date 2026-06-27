pub mod arm7tdmi {
  pub mod alu {

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ShiftType {
      Lsl = 0,
      Lsr = 1,
      Asr = 2,
      Ror = 3,
    }

    impl ShiftType {
      pub fn from_u8(val: u8) -> Self {
        match val & 3 {
          0 => ShiftType::Lsl,
          1 => ShiftType::Lsr,
          2 => ShiftType::Asr,
          3 => ShiftType::Ror,
          _ => unreachable!(),
        }
      }
    }

    #[inline]
    pub fn barrel_shift(
      value: u32,
      shift_type: ShiftType,
      amount: u8,
      carry_in: bool,
      immediate: bool,
    ) -> (u32, bool) {
      match shift_type {
        ShiftType::Lsl => shift_lsl(value, amount, carry_in),
        ShiftType::Lsr => shift_lsr(value, amount, carry_in, immediate),
        ShiftType::Asr => shift_asr(value, amount, carry_in, immediate),
        ShiftType::Ror => shift_ror(value, amount, carry_in, immediate),
      }
    }

    fn shift_lsl(value: u32, amount: u8, carry_in: bool) -> (u32, bool) {
      match amount {
        0 => (value, carry_in),
        1..=31 => {
          let carry = (value >> (32 - amount)) & 1 != 0;
          (value << amount, carry)
        }
        32 => (0, value & 1 != 0),
        _ => (0, false),
      }
    }

    fn shift_lsr(value: u32, amount: u8, carry_in: bool, immediate: bool) -> (u32, bool) {
      match amount {
        0 => {
          if immediate {
            (0, value >> 31 != 0)
          } else {
            (value, carry_in)
          }
        }
        1..=31 => {
          let carry = (value >> (amount - 1)) & 1 != 0;
          (value >> amount, carry)
        }
        32 => (0, value >> 31 != 0),
        _ => (0, false),
      }
    }

    fn shift_asr(value: u32, amount: u8, carry_in: bool, immediate: bool) -> (u32, bool) {
      match amount {
        0 => {
          if immediate {
            let carry = (value as i32) < 0;
            let result = if carry { 0xFFFF_FFFF } else { 0 };
            (result, carry)
          } else {
            (value, carry_in)
          }
        }
        1..=31 => {
          let carry = ((value as i32) >> (amount - 1)) & 1 != 0;
          ((value as i32 >> amount) as u32, carry)
        }
        _ => {
          let carry = (value as i32) < 0;
          let result = if carry { 0xFFFF_FFFF } else { 0 };
          (result, carry)
        }
      }
    }

    fn shift_ror(value: u32, amount: u8, carry_in: bool, immediate: bool) -> (u32, bool) {
      match amount {
        0 => {
          if immediate {
            let result = (carry_in as u32) << 31 | (value >> 1);
            let carry = value & 1 != 0;
            (result, carry)
          } else {
            (value, carry_in)
          }
        }
        _ => {
          let amount = amount & 31;
          if amount == 0 {
            (value, value >> 31 != 0)
          } else {
            let result = value.rotate_right(amount as u32);
            let carry = result >> 31 != 0;
            (result, carry)
          }
        }
      }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AluOp {
      And = 0x0,
      Eor = 0x1,
      Sub = 0x2,
      Rsb = 0x3,
      Add = 0x4,
      Adc = 0x5,
      Sbc = 0x6,
      Rsc = 0x7,
      Tst = 0x8,
      Teq = 0x9,
      Cmp = 0xA,
      Cmn = 0xB,
      Orr = 0xC,
      Mov = 0xD,
      Bic = 0xE,
      Mvn = 0xF,
    }

    impl AluOp {
      pub fn from_u8(val: u8) -> Self {
        match val & 0xF {
          0x0 => AluOp::And,
          0x1 => AluOp::Eor,
          0x2 => AluOp::Sub,
          0x3 => AluOp::Rsb,
          0x4 => AluOp::Add,
          0x5 => AluOp::Adc,
          0x6 => AluOp::Sbc,
          0x7 => AluOp::Rsc,
          0x8 => AluOp::Tst,
          0x9 => AluOp::Teq,
          0xA => AluOp::Cmp,
          0xB => AluOp::Cmn,
          0xC => AluOp::Orr,
          0xD => AluOp::Mov,
          0xE => AluOp::Bic,
          0xF => AluOp::Mvn,
          _ => unreachable!(),
        }
      }

      pub fn is_test(self) -> bool {
        matches!(self, AluOp::Tst | AluOp::Teq | AluOp::Cmp | AluOp::Cmn)
      }

      pub fn is_logical(self) -> bool {
        matches!(
          self,
          AluOp::And
            | AluOp::Eor
            | AluOp::Tst
            | AluOp::Teq
            | AluOp::Orr
            | AluOp::Mov
            | AluOp::Bic
            | AluOp::Mvn
        )
      }
    }

    #[inline]
    pub fn add_with_carry(a: u32, b: u32, carry_in: bool) -> (u32, bool, bool) {
      let result = (a as u64) + (b as u64) + (carry_in as u64);
      let result32 = result as u32;
      let carry = result > 0xFFFF_FFFF;
      let overflow = ((a ^ result32) & (b ^ result32)) >> 31 != 0;
      (result32, carry, overflow)
    }

    #[inline]
    pub fn sub_with_carry(a: u32, b: u32, carry_in: bool) -> (u32, bool, bool) {
      add_with_carry(a, !b, carry_in)
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_lsl() {
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Lsl, 1, false, false),
          (0, true)
        );
        assert_eq!(
          barrel_shift(1, ShiftType::Lsl, 31, false, false),
          (0x80000000, false)
        );
        assert_eq!(
          barrel_shift(0xFF, ShiftType::Lsl, 0, true, false),
          (0xFF, true)
        );
        assert_eq!(
          barrel_shift(0xFF, ShiftType::Lsl, 0, false, false),
          (0xFF, false)
        );
      }

      #[test]
      fn test_lsr() {
        assert_eq!(barrel_shift(1, ShiftType::Lsr, 1, false, false), (0, true));
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Lsr, 31, false, false),
          (1, false)
        );
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Lsr, 0, false, true),
          (0, true)
        );
      }

      #[test]
      fn test_asr() {
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Asr, 1, false, false),
          (0xC0000000, false)
        );
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Asr, 31, false, false),
          (0xFFFFFFFF, false)
        );
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Asr, 0, false, true),
          (0xFFFFFFFF, true)
        );
        assert_eq!(
          barrel_shift(0x7FFFFFFF, ShiftType::Asr, 0, false, true),
          (0, false)
        );
      }

      #[test]
      fn test_ror() {
        assert_eq!(
          barrel_shift(1, ShiftType::Ror, 1, false, false),
          (0x80000000, true)
        );
        assert_eq!(
          barrel_shift(0x80000000, ShiftType::Ror, 1, false, false),
          (0x40000000, false)
        );
      }

      #[test]
      fn test_rrx() {
        assert_eq!(
          barrel_shift(1, ShiftType::Ror, 0, true, true),
          (0x80000000, true)
        );
        assert_eq!(barrel_shift(1, ShiftType::Ror, 0, false, true), (0, true));
        assert_eq!(
          barrel_shift(0, ShiftType::Ror, 0, true, true),
          (0x80000000, false)
        );
      }

      #[test]
      fn test_add_with_carry() {
        assert_eq!(add_with_carry(0xFFFFFFFF, 1, false), (0, true, false));
        assert_eq!(
          add_with_carry(0x7FFFFFFF, 1, false),
          (0x80000000, false, true)
        );
        assert_eq!(
          add_with_carry(0x80000000, 0x80000000, false),
          (0, true, true)
        );
      }

      #[test]
      fn test_sub_with_carry() {
        assert_eq!(sub_with_carry(5, 3, true), (2, true, false));
        assert_eq!(sub_with_carry(3, 5, true), (0xFFFFFFFE, false, false));
        assert_eq!(sub_with_carry(0, 1, true), (0xFFFFFFFF, false, false));
      }
    }
  }
  pub mod arm {
    use super::Cpu;
    use super::alu::{AluOp, ShiftType, add_with_carry, barrel_shift, sub_with_carry};
    use crate::bus::Bus;
    impl Cpu {
      pub fn execute_arm(&mut self, bus: &mut Bus, opcode: u32) -> u32 {
        let bits_27_20 = (opcode >> 20) & 0xFF;
        let bits_7_4 = (opcode >> 4) & 0xF;
        match bits_27_20 >> 5 {
          0b000 => {
            if bits_27_20 == 0x12 && bits_7_4 == 0x1 {
              self.arm_branch_exchange(opcode)
            } else if (bits_7_4 & 0x9) == 0x9 && (bits_27_20 & 0xE0) == 0 {
              match bits_7_4 {
                0x9 => {
                  if bits_27_20 & 0xFC == 0x00 {
                    self.arm_multiply(bus, opcode)
                  } else if bits_27_20 & 0xF8 == 0x08 {
                    self.arm_multiply_long(bus, opcode)
                  } else if bits_27_20 & 0xFB == 0x10 {
                    self.arm_swap(bus, opcode)
                  } else {
                    self.arm_undefined(opcode)
                  }
                }
                0xB | 0xD | 0xF => self.arm_halfword_transfer(bus, opcode),
                _ => self.arm_data_processing(bus, opcode),
              }
            } else {
              if (bits_27_20 & 0xFB) == 0x10 && bits_7_4 == 0x0 {
                self.arm_mrs(opcode)
              } else if (bits_27_20 & 0xFB) == 0x12 && bits_7_4 == 0x0 {
                self.arm_msr(opcode)
              } else {
                self.arm_data_processing(bus, opcode)
              }
            }
          }
          0b001 => {
            if (bits_27_20 & 0xFB) == 0x32 {
              self.arm_msr(opcode)
            } else {
              self.arm_data_processing(bus, opcode)
            }
          }
          0b010 => self.arm_single_transfer(bus, opcode),
          0b011 => {
            if opcode & (1 << 4) != 0 {
              self.arm_undefined(opcode)
            } else {
              self.arm_single_transfer(bus, opcode)
            }
          }
          0b100 => self.arm_block_transfer(bus, opcode),
          0b101 => self.arm_branch(opcode),
          0b111 => {
            if opcode >> 24 & 0xF == 0xF {
              self.arm_swi(opcode)
            } else {
              self.arm_undefined(opcode)
            }
          }
          _ => self.arm_undefined(opcode),
        }
      }

      fn arm_data_processing(&mut self, _bus: &mut Bus, opcode: u32) -> u32 {
        let i = opcode & (1 << 25) != 0;
        let s = opcode & (1 << 20) != 0;
        let op = AluOp::from_u8(((opcode >> 21) & 0xF) as u8);
        let rn = ((opcode >> 16) & 0xF) as u8;
        let rd = ((opcode >> 12) & 0xF) as u8;
        let shift_by_reg = !i && opcode & (1 << 4) != 0;
        let op1 = if rn == 15 && shift_by_reg {
          self.reg(15).wrapping_add(4)
        } else {
          self.reg(rn)
        };
        let (op2, shifter_carry) = if i {
          let imm = opcode & 0xFF;
          let rotate = ((opcode >> 8) & 0xF) * 2;
          if rotate == 0 {
            (imm, self.cpsr.c())
          } else {
            let result = imm.rotate_right(rotate);
            (result, result >> 31 != 0)
          }
        } else {
          let rm = (opcode & 0xF) as u8;
          let shift_type = ShiftType::from_u8(((opcode >> 5) & 3) as u8);
          let shift_amount = if shift_by_reg {
            let rs = ((opcode >> 8) & 0xF) as u8;
            let rs_val = if rs == 15 {
              self.reg(15).wrapping_add(4)
            } else {
              self.reg(rs)
            };
            rs_val as u8
          } else {
            ((opcode >> 7) & 0x1F) as u8
          };
          let rm_val = if rm == 15 && shift_by_reg {
            self.reg(15).wrapping_add(4)
          } else {
            self.reg(rm)
          };
          let immediate_shift = !shift_by_reg;
          barrel_shift(
            rm_val,
            shift_type,
            shift_amount,
            self.cpsr.c(),
            immediate_shift,
          )
        };
        let extra_internal_cycle: u32 = if shift_by_reg { 1 } else { 0 };
        let (result, carry, overflow) = match op {
          AluOp::And | AluOp::Tst => (op1 & op2, shifter_carry, self.cpsr.v()),
          AluOp::Eor | AluOp::Teq => (op1 ^ op2, shifter_carry, self.cpsr.v()),
          AluOp::Sub | AluOp::Cmp => sub_with_carry(op1, op2, true),
          AluOp::Rsb => sub_with_carry(op2, op1, true),
          AluOp::Add | AluOp::Cmn => add_with_carry(op1, op2, false),
          AluOp::Adc => add_with_carry(op1, op2, self.cpsr.c()),
          AluOp::Sbc => sub_with_carry(op1, op2, self.cpsr.c()),
          AluOp::Rsc => sub_with_carry(op2, op1, self.cpsr.c()),
          AluOp::Orr => (op1 | op2, shifter_carry, self.cpsr.v()),
          AluOp::Mov => (op2, shifter_carry, self.cpsr.v()),
          AluOp::Bic => (op1 & !op2, shifter_carry, self.cpsr.v()),
          AluOp::Mvn => (!op2, shifter_carry, self.cpsr.v()),
        };
        if s {
          if rd == 15 {
            if op.is_test() {
              let spsr = self.spsr();
              let new_mode = spsr.mode();
              self.switch_mode(new_mode);
              self.cpsr = spsr;
            } else {
              self.set_reg_with_flags(rd, result, true);
            }
          } else {
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            if !op.is_logical() {
              self.cpsr.set_v(overflow);
            }
            if !op.is_test() {
              self.regs[rd as usize] = result;
            }
          }
        } else if !op.is_test() {
          self.set_reg(rd, result);
        }
        let pc_write_cycles = if rd == 15 { 2 } else { 0 };
        1 + extra_internal_cycle + pc_write_cycles
      }

      fn arm_multiply(&mut self, _bus: &mut Bus, opcode: u32) -> u32 {
        let a = opcode & (1 << 21) != 0;
        let s = opcode & (1 << 20) != 0;
        let rd = ((opcode >> 16) & 0xF) as u8;
        let rn = ((opcode >> 12) & 0xF) as u8;
        let rs = ((opcode >> 8) & 0xF) as u8;
        let rm = (opcode & 0xF) as u8;
        let result = if a {
          self
            .reg(rm)
            .wrapping_mul(self.reg(rs))
            .wrapping_add(self.reg(rn))
        } else {
          self.reg(rm).wrapping_mul(self.reg(rs))
        };
        self.regs[rd as usize] = result;
        if s {
          self.cpsr.set_nz(result);
        }
        4
      }

      fn arm_multiply_long(&mut self, _bus: &mut Bus, opcode: u32) -> u32 {
        let u = opcode & (1 << 22) != 0;
        let a = opcode & (1 << 21) != 0;
        let s = opcode & (1 << 20) != 0;
        let rd_hi = ((opcode >> 16) & 0xF) as u8;
        let rd_lo = ((opcode >> 12) & 0xF) as u8;
        let rs = ((opcode >> 8) & 0xF) as u8;
        let rm = (opcode & 0xF) as u8;
        let result = if u {
          let result = (self.reg(rm) as i32 as i64) * (self.reg(rs) as i32 as i64);
          if a {
            let acc = ((self.reg(rd_hi) as u64) << 32) | self.reg(rd_lo) as u64;
            (result as u64).wrapping_add(acc)
          } else {
            result as u64
          }
        } else {
          let result = (self.reg(rm) as u64) * (self.reg(rs) as u64);
          if a {
            let acc = ((self.reg(rd_hi) as u64) << 32) | self.reg(rd_lo) as u64;
            result.wrapping_add(acc)
          } else {
            result
          }
        };
        self.regs[rd_lo as usize] = result as u32;
        self.regs[rd_hi as usize] = (result >> 32) as u32;
        if s {
          self.cpsr.set_n((result >> 63) != 0);
          self.cpsr.set_z(result == 0);
        }
        5
      }

      fn arm_single_transfer(&mut self, bus: &mut Bus, opcode: u32) -> u32 {
        let i = opcode & (1 << 25) != 0;
        let p = opcode & (1 << 24) != 0;
        let u = opcode & (1 << 23) != 0;
        let b = opcode & (1 << 22) != 0;
        let w = opcode & (1 << 21) != 0;
        let l = opcode & (1 << 20) != 0;
        let rn = ((opcode >> 16) & 0xF) as u8;
        let rd = ((opcode >> 12) & 0xF) as u8;
        let base = self.reg(rn);
        let offset = if !i {
          opcode & 0xFFF
        } else {
          let rm = (opcode & 0xF) as u8;
          let shift_type = ShiftType::from_u8(((opcode >> 5) & 3) as u8);
          let shift_amount = ((opcode >> 7) & 0x1F) as u8;
          let (shifted, _) =
            barrel_shift(self.reg(rm), shift_type, shift_amount, self.cpsr.c(), true);
          shifted
        };
        let offset_addr = if u {
          base.wrapping_add(offset)
        } else {
          base.wrapping_sub(offset)
        };
        let addr = if p { offset_addr } else { base };
        let mut cycles = 1;
        if l {
          let val = if b {
            bus.read8(addr) as u32
          } else {
            let aligned = addr & !3;
            let val = bus.read32(aligned);
            let rotation = (addr & 3) * 8;
            val.rotate_right(rotation)
          };
          self.set_reg(rd, val);
          cycles += 1;
        } else {
          let val = if rd == 15 {
            self.reg(15).wrapping_add(4)
          } else {
            self.reg(rd)
          };
          if b {
            bus.write8(addr, val as u8);
          } else {
            bus.write32(addr & !3, val);
          }
        }
        if (!p || w) && !(l && rn == rd) && rn != 15 {
          self.regs[rn as usize] = offset_addr;
        }
        cycles
      }

      fn arm_halfword_transfer(&mut self, bus: &mut Bus, opcode: u32) -> u32 {
        let p = opcode & (1 << 24) != 0;
        let u = opcode & (1 << 23) != 0;
        let i = opcode & (1 << 22) != 0;
        let w = opcode & (1 << 21) != 0;
        let l = opcode & (1 << 20) != 0;
        let rn = ((opcode >> 16) & 0xF) as u8;
        let rd = ((opcode >> 12) & 0xF) as u8;
        let sh = (opcode >> 5) & 3;
        let base = self.reg(rn);
        let offset = if i {
          ((opcode >> 4) & 0xF0) | (opcode & 0xF)
        } else {
          let rm = (opcode & 0xF) as u8;
          self.reg(rm)
        };
        let offset_addr = if u {
          base.wrapping_add(offset)
        } else {
          base.wrapping_sub(offset)
        };
        let addr = if p { offset_addr } else { base };
        if l {
          let val = match sh {
            0x1 => {
              let val = bus.read16(addr & !1) as u32;
              if addr & 1 != 0 {
                val.rotate_right(8)
              } else {
                val
              }
            }
            0x2 => bus.read8(addr) as i8 as i32 as u32,
            0x3 => {
              if addr & 1 != 0 {
                bus.read8(addr) as i8 as i32 as u32
              } else {
                bus.read16(addr) as i16 as i32 as u32
              }
            }
            _ => 0,
          };
          self.set_reg(rd, val);
        } else {
          let val = self.reg(rd);
          bus.write16(addr & !1, val as u16);
        }
        if (!p || w) && !(l && rn == rd) && rn != 15 {
          self.regs[rn as usize] = offset_addr;
        }
        if l { 3 } else { 2 }
      }

      fn arm_block_transfer(&mut self, bus: &mut Bus, opcode: u32) -> u32 {
        let p = opcode & (1 << 24) != 0;
        let u = opcode & (1 << 23) != 0;
        let s = opcode & (1 << 22) != 0;
        let w = opcode & (1 << 21) != 0;
        let l = opcode & (1 << 20) != 0;
        let rn = ((opcode >> 16) & 0xF) as u8;
        let rlist = (opcode & 0xFFFF) as u16;
        let base = self.reg(rn);
        let reg_count = rlist.count_ones();
        if rlist == 0 {
          let xfer_addr = match (u, p) {
            (true, false) => base,
            (true, true) => base.wrapping_add(4),
            (false, false) => base.wrapping_sub(0x3C),
            (false, true) => base.wrapping_sub(0x40),
          };
          if l {
            let val = bus.read32(xfer_addr);
            self.branch(val & !1);
          } else {
            bus.write32(xfer_addr, self.reg(15).wrapping_add(4));
          }
          if w {
            self.regs[rn as usize] = if u {
              base.wrapping_add(0x40)
            } else {
              base.wrapping_sub(0x40)
            };
          }
          return 3;
        }
        let mut addr = if u {
          if p { base.wrapping_add(4) } else { base }
        } else {
          let total = reg_count * 4;
          if p {
            base.wrapping_sub(total)
          } else {
            base.wrapping_sub(total).wrapping_add(4)
          }
        };
        let final_addr = if u {
          base.wrapping_add(reg_count * 4)
        } else {
          base.wrapping_sub(reg_count * 4)
        };
        let r15_in_list = rlist & (1 << 15) != 0;
        let use_user_bank = s && !(l && r15_in_list);
        let rn_in_list = rlist & (1 << rn) != 0;
        let rn_is_lowest = rn_in_list && rlist.trailing_zeros() as u8 == rn;
        for i in 0..16u8 {
          if rlist & (1 << i) == 0 {
            continue;
          }
          if l {
            let val = bus.read32(addr & !3);
            if s && r15_in_list {
              if i == 15 {
                let spsr = self.spsr();
                let new_mode = spsr.mode();
                self.switch_mode(new_mode);
                self.cpsr = spsr;
                self.branch(val & !1);
              } else {
                self.regs[i as usize] = val;
              }
            } else if i == 15 {
              self.branch(val & !1);
            } else if use_user_bank {
              self.write_user_reg(i, val);
            } else {
              self.regs[i as usize] = val;
            }
          } else {
            let val = if i == 15 {
              self.reg(15).wrapping_add(4)
            } else if i == rn && rn_in_list && !rn_is_lowest && w {
              final_addr
            } else if use_user_bank {
              self.read_user_reg(i)
            } else {
              self.reg(i)
            };
            bus.write32(addr & !3, val);
          }
          addr = addr.wrapping_add(4);
        }
        if w && !(l && rlist & (1 << rn) != 0) {
          self.regs[rn as usize] = final_addr;
        }
        if l { reg_count + 2 } else { reg_count + 1 }
      }

      fn arm_branch(&mut self, opcode: u32) -> u32 {
        let link = opcode & (1 << 24) != 0;
        let offset = ((opcode & 0x00FF_FFFF) as i32) << 8 >> 6;
        if link {
          self.regs[14] = self.regs[15].wrapping_sub(4);
        }
        let target = (self.regs[15] as i32).wrapping_add(offset) as u32;
        self.branch(target);
        3
      }

      fn arm_branch_exchange(&mut self, opcode: u32) -> u32 {
        let rm = (opcode & 0xF) as u8;
        let addr = self.reg(rm);
        self.branch_exchange(addr);
        3
      }

      fn arm_swap(&mut self, bus: &mut Bus, opcode: u32) -> u32 {
        let b = opcode & (1 << 22) != 0;
        let rn = ((opcode >> 16) & 0xF) as u8;
        let rd = ((opcode >> 12) & 0xF) as u8;
        let rm = (opcode & 0xF) as u8;
        let addr = self.reg(rn);
        if b {
          let old = bus.read8(addr) as u32;
          bus.write8(addr, self.reg(rm) as u8);
          self.regs[rd as usize] = old;
        } else {
          let aligned = addr & !3;
          let old = bus.read32(aligned);
          let rotation = (addr & 3) * 8;
          let old_rotated = old.rotate_right(rotation);
          bus.write32(aligned, self.reg(rm));
          self.regs[rd as usize] = old_rotated;
        }
        4
      }

      fn arm_mrs(&mut self, opcode: u32) -> u32 {
        let spsr = opcode & (1 << 22) != 0;
        let rd = ((opcode >> 12) & 0xF) as u8;
        let psr = if spsr { self.spsr() } else { self.cpsr };
        self.regs[rd as usize] = psr.bits;
        1
      }

      fn arm_msr(&mut self, opcode: u32) -> u32 {
        let i = opcode & (1 << 25) != 0;
        let spsr = opcode & (1 << 22) != 0;
        let field_mask = (opcode >> 16) & 0xF;
        let mut mask = 0u32;
        if field_mask & 1 != 0 {
          mask |= 0x0000_00FF;
        }
        if field_mask & 2 != 0 {
          mask |= 0x0000_FF00;
        }
        if field_mask & 4 != 0 {
          mask |= 0x00FF_0000;
        }
        if field_mask & 8 != 0 {
          mask |= 0xFF00_0000;
        }
        if self.cpsr.mode() == super::CpuMode::User {
          mask &= 0xFF00_0000;
        }
        let val = if i {
          let imm = opcode & 0xFF;
          let rotate = ((opcode >> 8) & 0xF) * 2;
          imm.rotate_right(rotate)
        } else {
          let rm = (opcode & 0xF) as u8;
          self.reg(rm)
        };
        if spsr {
          let mut psr = self.spsr();
          psr.bits = (psr.bits & !mask) | (val & mask);
          self.set_spsr(psr);
        } else {
          let old_mode = self.cpsr.mode();
          let new_bits = (self.cpsr.bits & !mask) | (val & mask);
          let new_mode = super::Psr { bits: new_bits }.mode();
          if old_mode != new_mode {
            self.switch_mode(new_mode);
          }
          self.cpsr.bits = new_bits;
        }
        1
      }

      fn arm_swi(&mut self, opcode: u32) -> u32 {
        let comment = (opcode >> 16) & 0xFF;
        self.pending_swi = Some(comment as u8);
        3
      }

      fn arm_undefined(&mut self, _opcode: u32) -> u32 {
        eprintln!(
          "ARM undefined instruction: 0x{:08X} at PC=0x{:08X}",
          _opcode,
          self.regs[15].wrapping_sub(8)
        );
        1
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      fn make_cpu_bus() -> (Cpu, Bus) {
        let cpu = Cpu::new_post_bios();
        let bus = Bus::new(None, vec![0; 256]);
        (cpu, bus)
      }

      #[test]
      fn test_arm_mov_immediate() {
        let (mut cpu, mut bus) = make_cpu_bus();
        let opcode: u32 = 0xE3A0_002A;
        cpu.execute_arm(&mut bus, opcode);
        assert_eq!(cpu.regs[0], 42);
      }

      #[test]
      fn test_arm_add() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[1] = 10;
        cpu.regs[2] = 20;
        let opcode: u32 = 0xE081_0002;
        cpu.execute_arm(&mut bus, opcode);
        assert_eq!(cpu.regs[0], 30);
      }

      #[test]
      fn test_arm_sub_with_flags() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[1] = 5;
        cpu.regs[2] = 5;
        let opcode: u32 = 0xE051_0002;
        cpu.execute_arm(&mut bus, opcode);
        assert_eq!(cpu.regs[0], 0);
        assert!(cpu.cpsr.z());
        assert!(cpu.cpsr.c());
      }

      #[test]
      fn test_arm_cmp() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 10;
        let opcode: u32 = 0xE350_000A;
        cpu.execute_arm(&mut bus, opcode);
        assert!(cpu.cpsr.z());
      }

      #[test]
      fn test_arm_branch() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[15] = 0x0800_0008;
        cpu.pipeline_flushed = false;
        let opcode: u32 = 0xEA00_003E;
        cpu.execute_arm(&mut bus, opcode);
        assert_eq!(cpu.regs[15], 0x0800_0100);
      }

      #[test]
      fn test_arm_str_ldr() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 0xDEAD_BEEF;
        cpu.regs[1] = 0x0200_0000;
        let opcode_str: u32 = 0xE581_0000;
        cpu.execute_arm(&mut bus, opcode_str);
        let opcode_ldr: u32 = 0xE591_2000;
        cpu.execute_arm(&mut bus, opcode_ldr);
        assert_eq!(cpu.regs[2], 0xDEAD_BEEF);
      }

      #[test]
      fn test_arm_multiply() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 7;
        cpu.regs[1] = 6;
        let opcode: u32 = 0xE002_0190;
        cpu.execute_arm(&mut bus, opcode);
        assert_eq!(cpu.regs[2], 42);
      }
    }
  }
  pub mod thumb {
    use super::Cpu;
    use super::alu::{ShiftType, add_with_carry, barrel_shift, sub_with_carry};
    use crate::bus::Bus;
    impl Cpu {
      pub fn execute_thumb(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        match opcode >> 8 {
          0x00..=0x17 => self.thumb_shift_imm(opcode),
          0x18..=0x1B => self.thumb_add_sub_reg(opcode),
          0x1C..=0x1F => self.thumb_add_sub_imm(opcode),
          0x20..=0x27 => self.thumb_mov_imm(opcode),
          0x28..=0x2F => self.thumb_cmp_imm(opcode),
          0x30..=0x37 => self.thumb_add_imm(opcode),
          0x38..=0x3F => self.thumb_sub_imm(opcode),
          0x40..=0x43 => self.thumb_alu(bus, opcode),
          0x44..=0x47 => self.thumb_hi_reg_bx(opcode),
          0x48..=0x4F => self.thumb_ldr_pc(bus, opcode),
          0x50..=0x5F => self.thumb_load_store_reg(bus, opcode),
          0x60..=0x7F => self.thumb_load_store_imm(bus, opcode),
          0x80..=0x8F => self.thumb_load_store_half(bus, opcode),
          0x90..=0x9F => self.thumb_load_store_sp(bus, opcode),
          0xA0..=0xAF => self.thumb_load_address(opcode),
          0xB0 => self.thumb_add_sp(opcode),
          0xB4..=0xB5 => self.thumb_push(bus, opcode),
          0xBC..=0xBD => self.thumb_pop(bus, opcode),
          0xC0..=0xC7 => self.thumb_stmia(bus, opcode),
          0xC8..=0xCF => self.thumb_ldmia(bus, opcode),
          0xD0..=0xDD => self.thumb_cond_branch(opcode),
          0xDF => self.thumb_swi(opcode),
          0xE0..=0xE7 => self.thumb_branch(opcode),
          0xF0..=0xF7 => self.thumb_bl_prefix(opcode),
          0xF8..=0xFF => self.thumb_bl_suffix(bus, opcode),
          _ => {
            eprintln!(
              "THUMB undefined: 0x{:04X} at PC=0x{:08X}",
              opcode,
              self.regs[15].wrapping_sub(4)
            );
            1
          }
        }
      }

      fn thumb_shift_imm(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 11) & 3;
        let offset = ((opcode >> 6) & 0x1F) as u8;
        let rs = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let shift_type = match op {
          0 => ShiftType::Lsl,
          1 => ShiftType::Lsr,
          2 => ShiftType::Asr,
          _ => unreachable!(),
        };
        let (result, carry) = barrel_shift(self.reg(rs), shift_type, offset, self.cpsr.c(), true);
        self.regs[rd as usize] = result;
        self.cpsr.set_nz(result);
        self.cpsr.set_c(carry);
        1
      }

      fn thumb_add_sub_reg(&mut self, opcode: u16) -> u32 {
        let sub = opcode & (1 << 9) != 0;
        let rn = ((opcode >> 6) & 7) as u8;
        let rs = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let a = self.reg(rs);
        let b = self.reg(rn);
        let (result, carry, overflow) = if sub {
          sub_with_carry(a, b, true)
        } else {
          add_with_carry(a, b, false)
        };
        self.regs[rd as usize] = result;
        self.cpsr.set_nz(result);
        self.cpsr.set_c(carry);
        self.cpsr.set_v(overflow);
        1
      }

      fn thumb_add_sub_imm(&mut self, opcode: u16) -> u32 {
        let sub = opcode & (1 << 9) != 0;
        let imm = ((opcode >> 6) & 7) as u32;
        let rs = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let a = self.reg(rs);
        let (result, carry, overflow) = if sub {
          sub_with_carry(a, imm, true)
        } else {
          add_with_carry(a, imm, false)
        };
        self.regs[rd as usize] = result;
        self.cpsr.set_nz(result);
        self.cpsr.set_c(carry);
        self.cpsr.set_v(overflow);
        1
      }

      fn thumb_mov_imm(&mut self, opcode: u16) -> u32 {
        let rd = ((opcode >> 8) & 7) as u8;
        let imm = (opcode & 0xFF) as u32;
        self.regs[rd as usize] = imm;
        self.cpsr.set_nz(imm);
        1
      }

      fn thumb_cmp_imm(&mut self, opcode: u16) -> u32 {
        let rd = ((opcode >> 8) & 7) as u8;
        let imm = (opcode & 0xFF) as u32;
        let (result, carry, overflow) = sub_with_carry(self.reg(rd), imm, true);
        self.cpsr.set_nz(result);
        self.cpsr.set_c(carry);
        self.cpsr.set_v(overflow);
        1
      }

      fn thumb_add_imm(&mut self, opcode: u16) -> u32 {
        let rd = ((opcode >> 8) & 7) as u8;
        let imm = (opcode & 0xFF) as u32;
        let (result, carry, overflow) = add_with_carry(self.reg(rd), imm, false);
        self.regs[rd as usize] = result;
        self.cpsr.set_nz(result);
        self.cpsr.set_c(carry);
        self.cpsr.set_v(overflow);
        1
      }

      fn thumb_sub_imm(&mut self, opcode: u16) -> u32 {
        let rd = ((opcode >> 8) & 7) as u8;
        let imm = (opcode & 0xFF) as u32;
        let (result, carry, overflow) = sub_with_carry(self.reg(rd), imm, true);
        self.regs[rd as usize] = result;
        self.cpsr.set_nz(result);
        self.cpsr.set_c(carry);
        self.cpsr.set_v(overflow);
        1
      }

      fn thumb_alu(&mut self, _bus: &mut Bus, opcode: u16) -> u32 {
        let op = (opcode >> 6) & 0xF;
        let rs = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let a = self.reg(rd);
        let b = self.reg(rs);
        match op {
          0x0 => {
            let result = a & b;
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
          }
          0x1 => {
            let result = a ^ b;
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
          }
          0x2 => {
            let (result, carry) = barrel_shift(a, ShiftType::Lsl, b as u8, self.cpsr.c(), false);
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
          }
          0x3 => {
            let (result, carry) = barrel_shift(a, ShiftType::Lsr, b as u8, self.cpsr.c(), false);
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
          }
          0x4 => {
            let (result, carry) = barrel_shift(a, ShiftType::Asr, b as u8, self.cpsr.c(), false);
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
          }
          0x5 => {
            let (result, carry, overflow) = add_with_carry(a, b, self.cpsr.c());
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
          }
          0x6 => {
            let (result, carry, overflow) = sub_with_carry(a, b, self.cpsr.c());
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
          }
          0x7 => {
            let (result, carry) = barrel_shift(a, ShiftType::Ror, b as u8, self.cpsr.c(), false);
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
          }
          0x8 => {
            let result = a & b;
            self.cpsr.set_nz(result);
          }
          0x9 => {
            let (result, carry, overflow) = sub_with_carry(0, b, true);
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
          }
          0xA => {
            let (result, carry, overflow) = sub_with_carry(a, b, true);
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
          }
          0xB => {
            let (result, carry, overflow) = add_with_carry(a, b, false);
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
          }
          0xC => {
            let result = a | b;
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
          }
          0xD => {
            let result = a.wrapping_mul(b);
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
          }
          0xE => {
            let result = a & !b;
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
          }
          0xF => {
            let result = !b;
            self.regs[rd as usize] = result;
            self.cpsr.set_nz(result);
          }
          _ => unreachable!(),
        }
        1
      }

      fn thumb_hi_reg_bx(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 8) & 3;
        let h1 = (opcode >> 7) & 1;
        let h2 = (opcode >> 6) & 1;
        let rs = (((h2 << 3) | ((opcode >> 3) & 7)) & 0xF) as u8;
        let rd = (((h1 << 3) | (opcode & 7)) & 0xF) as u8;
        match op {
          0 => {
            let result = self.reg(rd).wrapping_add(self.reg(rs));
            if rd == 15 {
              self.branch(result & !1);
            } else {
              self.regs[rd as usize] = result;
            }
          }
          1 => {
            let (result, carry, overflow) = sub_with_carry(self.reg(rd), self.reg(rs), true);
            self.cpsr.set_nz(result);
            self.cpsr.set_c(carry);
            self.cpsr.set_v(overflow);
          }
          2 => {
            let val = self.reg(rs);
            if rd == 15 {
              self.branch(val & !1);
            } else {
              self.regs[rd as usize] = val;
            }
          }
          3 => {
            let addr = self.reg(rs);
            self.branch_exchange(addr);
          }
          _ => unreachable!(),
        }
        if ((op == 0 || op == 2) && rd == 15) || op == 3 {
          3
        } else {
          1
        }
      }

      fn thumb_ldr_pc(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let rd = ((opcode >> 8) & 7) as u8;
        let offset = ((opcode & 0xFF) as u32) << 2;
        let addr = (self.regs[15] & !3).wrapping_add(offset);
        let val = bus.read32(addr & !3);
        self.regs[rd as usize] = val;
        3
      }

      fn thumb_load_store_reg(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let op = (opcode >> 10) & 3;
        let ro = ((opcode >> 6) & 7) as u8;
        let rb = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let addr = self.reg(rb).wrapping_add(self.reg(ro));
        match (opcode >> 9) & 7 {
          0b000 => {
            bus.write32(addr & !3, self.reg(rd));
          }
          0b001 => {
            bus.write16(addr & !1, self.reg(rd) as u16);
          }
          0b010 => {
            bus.write8(addr, self.reg(rd) as u8);
          }
          0b011 => {
            let val = bus.read8(addr) as i8 as i32 as u32;
            self.regs[rd as usize] = val;
          }
          0b100 => {
            let val = bus.read32(addr & !3);
            let rotation = (addr & 3) * 8;
            self.regs[rd as usize] = val.rotate_right(rotation);
          }
          0b101 => {
            let val = bus.read16(addr & !1) as u32;
            self.regs[rd as usize] = if addr & 1 != 0 {
              val.rotate_right(8)
            } else {
              val
            };
          }
          0b110 => {
            self.regs[rd as usize] = bus.read8(addr) as u32;
          }
          0b111 => {
            if addr & 1 != 0 {
              self.regs[rd as usize] = bus.read8(addr) as i8 as i32 as u32;
            } else {
              self.regs[rd as usize] = bus.read16(addr) as i16 as i32 as u32;
            }
          }
          _ => unreachable!(),
        }
        let _ = op;
        2
      }

      fn thumb_load_store_imm(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let b = opcode & (1 << 12) != 0;
        let l = opcode & (1 << 11) != 0;
        let offset = ((opcode >> 6) & 0x1F) as u32;
        let rb = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let base = self.reg(rb);
        let addr = if b {
          base.wrapping_add(offset)
        } else {
          base.wrapping_add(offset << 2)
        };
        if l {
          if b {
            self.regs[rd as usize] = bus.read8(addr) as u32;
          } else {
            let val = bus.read32(addr & !3);
            let rotation = (addr & 3) * 8;
            self.regs[rd as usize] = val.rotate_right(rotation);
          }
        } else {
          if b {
            bus.write8(addr, self.reg(rd) as u8);
          } else {
            bus.write32(addr & !3, self.reg(rd));
          }
        }
        2
      }

      fn thumb_load_store_half(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let l = opcode & (1 << 11) != 0;
        let offset = (((opcode >> 6) & 0x1F) as u32) << 1;
        let rb = ((opcode >> 3) & 7) as u8;
        let rd = (opcode & 7) as u8;
        let addr = self.reg(rb).wrapping_add(offset);
        if l {
          let val = bus.read16(addr & !1) as u32;
          self.regs[rd as usize] = if addr & 1 != 0 {
            val.rotate_right(8)
          } else {
            val
          };
        } else {
          bus.write16(addr & !1, self.reg(rd) as u16);
        }
        2
      }

      fn thumb_load_store_sp(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let l = opcode & (1 << 11) != 0;
        let rd = ((opcode >> 8) & 7) as u8;
        let offset = ((opcode & 0xFF) as u32) << 2;
        let addr = self.regs[13].wrapping_add(offset);
        if l {
          let val = bus.read32(addr & !3);
          let rotation = (addr & 3) * 8;
          self.regs[rd as usize] = val.rotate_right(rotation);
        } else {
          bus.write32(addr & !3, self.reg(rd));
        }
        2
      }

      fn thumb_load_address(&mut self, opcode: u16) -> u32 {
        let sp = opcode & (1 << 11) != 0;
        let rd = ((opcode >> 8) & 7) as u8;
        let offset = ((opcode & 0xFF) as u32) << 2;
        if sp {
          self.regs[rd as usize] = self.regs[13].wrapping_add(offset);
        } else {
          let pc = self.regs[15] & !2;
          self.regs[rd as usize] = pc.wrapping_add(offset);
        }
        1
      }

      fn thumb_add_sp(&mut self, opcode: u16) -> u32 {
        let negative = opcode & (1 << 7) != 0;
        let offset = ((opcode & 0x7F) as u32) << 2;
        if negative {
          self.regs[13] = self.regs[13].wrapping_sub(offset);
        } else {
          self.regs[13] = self.regs[13].wrapping_add(offset);
        }
        1
      }

      fn thumb_push(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let lr = opcode & (1 << 8) != 0;
        let rlist = opcode & 0xFF;
        let reg_count = rlist.count_ones() + lr as u32;
        let mut addr = self.regs[13].wrapping_sub(reg_count * 4);
        self.regs[13] = addr;
        for i in 0..8u8 {
          if rlist & (1 << i) != 0 {
            bus.write32(addr, self.reg(i));
            addr = addr.wrapping_add(4);
          }
        }
        if lr {
          bus.write32(addr, self.regs[14]);
        }
        reg_count + 1
      }

      fn thumb_pop(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let pc = opcode & (1 << 8) != 0;
        let rlist = opcode & 0xFF;
        let mut addr = self.regs[13];
        for i in 0..8u8 {
          if rlist & (1 << i) != 0 {
            self.regs[i as usize] = bus.read32(addr);
            addr = addr.wrapping_add(4);
          }
        }
        if pc {
          let val = bus.read32(addr);
          addr = addr.wrapping_add(4);
          self.branch(val & !1);
        }
        self.regs[13] = addr;
        let reg_count = rlist.count_ones() + pc as u32;
        reg_count + 2
      }

      fn thumb_stmia(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let rb = ((opcode >> 8) & 7) as u8;
        let rlist = opcode & 0xFF;
        let base = self.reg(rb);
        let mut addr = base;
        if rlist == 0 {
          bus.write32(addr, self.reg(15).wrapping_add(2));
          self.regs[rb as usize] = base.wrapping_add(0x40);
          return 2;
        }
        let rb_in_list = rlist & (1 << rb) != 0;
        let rb_is_lowest = rb_in_list && (rlist & ((1 << rb) - 1)) == 0;
        let final_addr = base.wrapping_add(rlist.count_ones() * 4);
        for i in 0..8u8 {
          if rlist & (1 << i) != 0 {
            let val = if i == rb && rb_in_list && !rb_is_lowest {
              final_addr
            } else {
              self.reg(i)
            };
            bus.write32(addr, val);
            addr = addr.wrapping_add(4);
          }
        }
        self.regs[rb as usize] = addr;
        rlist.count_ones() + 1
      }

      fn thumb_ldmia(&mut self, bus: &mut Bus, opcode: u16) -> u32 {
        let rb = ((opcode >> 8) & 7) as u8;
        let rlist = opcode & 0xFF;
        let base = self.reg(rb);
        let mut addr = base;
        if rlist == 0 {
          let val = bus.read32(addr);
          self.regs[rb as usize] = base.wrapping_add(0x40);
          self.branch(val & !1);
          return 5;
        }
        for i in 0..8u8 {
          if rlist & (1 << i) != 0 {
            self.regs[i as usize] = bus.read32(addr);
            addr = addr.wrapping_add(4);
          }
        }
        if rlist & (1 << rb) == 0 {
          self.regs[rb as usize] = addr;
        }
        rlist.count_ones() + 2
      }

      fn thumb_cond_branch(&mut self, opcode: u16) -> u32 {
        let cond = (opcode >> 8) & 0xF;
        if !self.check_condition(cond as u32) {
          return 1;
        }
        let offset = ((opcode & 0xFF) as i8 as i32) << 1;
        let target = (self.regs[15] as i32).wrapping_add(offset) as u32;
        self.branch(target);
        3
      }

      fn thumb_swi(&mut self, opcode: u16) -> u32 {
        let comment = (opcode & 0xFF) as u8;
        self.pending_swi = Some(comment);
        3
      }

      fn thumb_branch(&mut self, opcode: u16) -> u32 {
        let offset = (((opcode & 0x7FF) as i32) << 21) >> 20;
        let target = (self.regs[15] as i32).wrapping_add(offset) as u32;
        self.branch(target);
        3
      }

      fn thumb_bl_prefix(&mut self, opcode: u16) -> u32 {
        let offset = (((opcode & 0x7FF) as i32) << 21) >> 9;
        self.regs[14] = (self.regs[15] as i32).wrapping_add(offset) as u32;
        1
      }

      fn thumb_bl_suffix(&mut self, _bus: &mut Bus, opcode: u16) -> u32 {
        let offset = ((opcode & 0x7FF) as u32) << 1;
        let next_instr = self.regs[15].wrapping_sub(2);
        let target = self.regs[14].wrapping_add(offset);
        self.regs[14] = next_instr | 1;
        self.branch(target);
        4
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      fn make_cpu_bus() -> (Cpu, Bus) {
        let mut cpu = Cpu::new_post_bios();
        cpu.cpsr.set_thumb(true);
        let bus = Bus::new(None, vec![0; 256]);
        (cpu, bus)
      }

      #[test]
      fn test_thumb_mov_imm() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.execute_thumb(&mut bus, 0x202A);
        assert_eq!(cpu.regs[0], 42);
      }

      #[test]
      fn test_thumb_add_imm() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 10;
        cpu.execute_thumb(&mut bus, 0x3005);
        assert_eq!(cpu.regs[0], 15);
      }

      #[test]
      fn test_thumb_sub_imm() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 10;
        cpu.execute_thumb(&mut bus, 0x3805);
        assert_eq!(cpu.regs[0], 5);
      }

      #[test]
      fn test_thumb_cmp_sets_flags() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 42;
        cpu.execute_thumb(&mut bus, 0x282A);
        assert!(cpu.cpsr.z());
      }

      #[test]
      fn test_thumb_lsl() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[1] = 1;
        cpu.execute_thumb(&mut bus, 0x0108);
        assert_eq!(cpu.regs[0], 16);
      }

      #[test]
      fn test_thumb_push_pop() {
        let (mut cpu, mut bus) = make_cpu_bus();
        cpu.regs[0] = 0xAAAA;
        cpu.regs[1] = 0xBBBB;
        cpu.regs[13] = 0x0300_0100;
        cpu.execute_thumb(&mut bus, 0xB403);
        assert_eq!(cpu.regs[13], 0x0300_00F8);
        cpu.regs[0] = 0;
        cpu.regs[1] = 0;
        cpu.execute_thumb(&mut bus, 0xBC03);
        assert_eq!(cpu.regs[0], 0xAAAA);
        assert_eq!(cpu.regs[1], 0xBBBB);
        assert_eq!(cpu.regs[13], 0x0300_0100);
      }
    }
  }
  use crate::bus::Bus;
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum CpuMode {
    User = 0x10,
    Fiq = 0x11,
    Irq = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1B,
    System = 0x1F,
  }

  impl CpuMode {
    pub fn from_bits(bits: u32) -> Self {
      match bits & 0x1F {
        0x10 => CpuMode::User,
        0x11 => CpuMode::Fiq,
        0x12 => CpuMode::Irq,
        0x13 => CpuMode::Supervisor,
        0x17 => CpuMode::Abort,
        0x1B => CpuMode::Undefined,
        0x1F => CpuMode::System,
        _ => CpuMode::User,
      }
    }

    pub fn bank_index(self) -> usize {
      match self {
        CpuMode::User | CpuMode::System => 0,
        CpuMode::Fiq => 1,
        CpuMode::Irq => 2,
        CpuMode::Supervisor => 3,
        CpuMode::Abort => 4,
        CpuMode::Undefined => 5,
      }
    }

    pub fn has_spsr(self) -> bool {
      !matches!(self, CpuMode::User | CpuMode::System)
    }
  }

  #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
  pub struct Psr {
    pub bits: u32,
  }

  impl Psr {
    pub fn new(mode: CpuMode) -> Self {
      Psr {
        bits: mode as u32 | (1 << 7) | (1 << 6),
      }
    }

    #[inline]
    pub fn n(self) -> bool {
      self.bits >> 31 != 0
    }
    #[inline]
    pub fn z(self) -> bool {
      (self.bits >> 30) & 1 != 0
    }
    #[inline]
    pub fn c(self) -> bool {
      (self.bits >> 29) & 1 != 0
    }
    #[inline]
    pub fn v(self) -> bool {
      (self.bits >> 28) & 1 != 0
    }
    #[inline]
    pub fn irq_disabled(self) -> bool {
      (self.bits >> 7) & 1 != 0
    }
    #[inline]
    pub fn fiq_disabled(self) -> bool {
      (self.bits >> 6) & 1 != 0
    }
    #[inline]
    pub fn thumb(self) -> bool {
      (self.bits >> 5) & 1 != 0
    }
    #[inline]
    pub fn mode(self) -> CpuMode {
      CpuMode::from_bits(self.bits)
    }

    #[inline]
    pub fn set_n(&mut self, v: bool) {
      self.bits = (self.bits & !(1 << 31)) | ((v as u32) << 31);
    }
    #[inline]
    pub fn set_z(&mut self, v: bool) {
      self.bits = (self.bits & !(1 << 30)) | ((v as u32) << 30);
    }
    #[inline]
    pub fn set_c(&mut self, v: bool) {
      self.bits = (self.bits & !(1 << 29)) | ((v as u32) << 29);
    }
    #[inline]
    pub fn set_v(&mut self, v: bool) {
      self.bits = (self.bits & !(1 << 28)) | ((v as u32) << 28);
    }

    #[inline]
    pub fn set_nz(&mut self, result: u32) {
      self.set_n(result >> 31 != 0);
      self.set_z(result == 0);
    }

    pub fn set_thumb(&mut self, v: bool) {
      self.bits = (self.bits & !(1 << 5)) | ((v as u32) << 5);
    }
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct BankedRegisters {
    pub(crate) sp: [u32; 6],
    pub(crate) lr: [u32; 6],
    fiq_r8_r12: [u32; 5],
    usr_r8_r12: [u32; 5],
    spsr: [Psr; 5],
  }

  impl BankedRegisters {
    pub fn new() -> Self {
      BankedRegisters {
        sp: [0; 6],
        lr: [0; 6],
        fiq_r8_r12: [0; 5],
        usr_r8_r12: [0; 5],
        spsr: [Psr { bits: 0 }; 5],
      }
    }
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Cpu {
    pub regs: [u32; 16],
    pub cpsr: Psr,
    pub(crate) banked: BankedRegisters,
    pipeline: [u32; 2],
    pub pipeline_flushed: bool,
    pub halted: bool,
    #[serde(default)]
    pub intrwait_mask: u16,
    pub(crate) pending_swi: Option<u8>,
  }

  impl Cpu {
    pub fn new() -> Self {
      let mut cpu = Cpu {
        regs: [0; 16],
        cpsr: Psr::new(CpuMode::Supervisor),
        banked: BankedRegisters::new(),
        pipeline: [0; 2],
        pipeline_flushed: true,
        halted: false,
        intrwait_mask: 0,
        pending_swi: None,
      };
      cpu.regs[15] = 0x0000_0000;
      cpu
    }

    pub fn new_post_bios() -> Self {
      let mut cpu = Cpu::new();
      cpu.cpsr = Psr::new(CpuMode::System);
      cpu.cpsr.bits &= !(1 << 7);
      cpu.cpsr.bits &= !(1 << 6);
      cpu.regs[13] = 0x0300_7F00;
      cpu.banked.sp[CpuMode::Irq.bank_index()] = 0x0300_7FA0;
      cpu.banked.sp[CpuMode::Supervisor.bank_index()] = 0x0300_7FE0;
      cpu.regs[15] = 0x0800_0000;
      cpu.pipeline_flushed = true;
      cpu
    }

    pub fn step(&mut self, bus: &mut Bus) -> u32 {
      bus.last_pc = if self.cpsr.thumb() {
        self.regs[15].wrapping_sub(4)
      } else {
        self.regs[15].wrapping_sub(8)
      };
      if self.pipeline_flushed {
        self.refill_pipeline(bus);
      }
      let mut irq_entry_cycles = 0u32;
      if bus.interrupt.has_pending() && !self.cpsr.irq_disabled() {
        self.handle_interrupt(bus);
        self.halted = false;
        irq_entry_cycles = 3;
      }
      if self.halted {
        bus.tick_backup(1);
        return 1;
      }
      if self.pipeline_flushed {
        self.refill_pipeline(bus);
      }
      let prior_mem_cycles = bus.take_mem_cycles();
      let pre_pc = self.regs[15];
      let cycles = if self.cpsr.thumb() {
        self.step_thumb(bus)
      } else {
        self.step_arm(bus)
      };
      if pre_pc < 0x0000_4000 && self.regs[15] >= 0x0000_4000 {
        bus.bios_latch = match pre_pc {
          0x2C => 0xE25E_F004,
          0x34 => 0xE55E_C002,
          _ => bus.bios_latch,
        };
      }
      let mem_cycles = bus.take_mem_cycles();
      let total = cycles + prior_mem_cycles + mem_cycles + irq_entry_cycles;
      bus.tick_backup(total);
      total
    }

    fn step_arm(&mut self, bus: &mut Bus) -> u32 {
      let opcode = self.pipeline[0];
      if !self.check_condition(opcode >> 28) {
        self.advance_arm_pipeline(bus);
        return 1;
      }
      let cycles = self.execute_arm(bus, opcode);
      if !self.pipeline_flushed {
        self.advance_arm_pipeline(bus);
      }
      cycles
    }

    fn step_thumb(&mut self, bus: &mut Bus) -> u32 {
      let opcode = self.pipeline[0] as u16;
      let cycles = self.execute_thumb(bus, opcode);
      if !self.pipeline_flushed {
        self.advance_thumb_pipeline(bus);
      }
      cycles
    }

    #[inline]
    fn advance_arm_pipeline(&mut self, bus: &mut Bus) {
      self.pipeline[0] = self.pipeline[1];
      self.pipeline[1] = bus.read32(self.regs[15]);
      self.regs[15] = self.regs[15].wrapping_add(4);
    }

    #[inline]
    fn advance_thumb_pipeline(&mut self, bus: &mut Bus) {
      self.pipeline[0] = self.pipeline[1];
      self.pipeline[1] = bus.read16(self.regs[15]) as u32;
      self.regs[15] = self.regs[15].wrapping_add(2);
    }

    fn refill_pipeline(&mut self, bus: &mut Bus) {
      bus.break_sequential();
      if self.cpsr.thumb() {
        let pc = self.regs[15] & !1;
        bus.last_pc = pc;
        self.pipeline[0] = bus.read16(pc) as u32;
        self.pipeline[1] = bus.read16(pc + 2) as u32;
        self.regs[15] = pc + 4;
      } else {
        let pc = self.regs[15] & !3;
        bus.last_pc = pc;
        self.pipeline[0] = bus.read32(pc);
        self.pipeline[1] = bus.read32(pc + 4);
        self.regs[15] = pc + 8;
      }
      self.pipeline_flushed = false;
    }

    #[inline]
    fn check_condition(&self, cond: u32) -> bool {
      const TABLE: [u16; 16] = [
        0xF0F0, 0x0F0F, 0xCCCC, 0x3333, 0xFF00, 0x00FF, 0xAAAA, 0x5555, 0x0C0C, 0xF3F3, 0xAA55,
        0x55AA, 0x0A05, 0xF5FA, 0xFFFF, 0xFFFF,
      ];
      let flags = ((self.cpsr.bits >> 28) & 0xF) as usize;
      TABLE[(cond & 0xF) as usize] & (1 << flags) != 0
    }

    pub fn switch_mode(&mut self, new_mode: CpuMode) {
      let old_mode = self.cpsr.mode();
      if old_mode == new_mode {
        return;
      }
      let old_bank = old_mode.bank_index();
      self.banked.sp[old_bank] = self.regs[13];
      self.banked.lr[old_bank] = self.regs[14];
      if old_mode == CpuMode::Fiq {
        self.banked.fiq_r8_r12.copy_from_slice(&self.regs[8..13]);
        self.regs[8..13].copy_from_slice(&self.banked.usr_r8_r12);
      } else if new_mode == CpuMode::Fiq {
        self.banked.usr_r8_r12.copy_from_slice(&self.regs[8..13]);
        self.regs[8..13].copy_from_slice(&self.banked.fiq_r8_r12);
      }
      let new_bank = new_mode.bank_index();
      self.regs[13] = self.banked.sp[new_bank];
      self.regs[14] = self.banked.lr[new_bank];
      self.cpsr.bits = (self.cpsr.bits & !0x1F) | (new_mode as u32);
    }

    pub fn spsr(&self) -> Psr {
      let mode = self.cpsr.mode();
      if mode.has_spsr() {
        let index = match mode {
          CpuMode::Fiq => 0,
          CpuMode::Irq => 1,
          CpuMode::Supervisor => 2,
          CpuMode::Abort => 3,
          CpuMode::Undefined => 4,
          _ => return self.cpsr,
        };
        self.banked.spsr[index]
      } else {
        self.cpsr
      }
    }

    pub fn set_spsr(&mut self, psr: Psr) {
      let mode = self.cpsr.mode();
      if mode.has_spsr() {
        let index = match mode {
          CpuMode::Fiq => 0,
          CpuMode::Irq => 1,
          CpuMode::Supervisor => 2,
          CpuMode::Abort => 3,
          CpuMode::Undefined => 4,
          _ => return,
        };
        self.banked.spsr[index] = psr;
      }
    }

    fn handle_interrupt(&mut self, bus: &mut Bus) {
      bus.bios_latch = 0xE25E_F004;
      let return_addr = if self.cpsr.thumb() {
        self.regs[15]
      } else {
        self.regs[15].wrapping_sub(4)
      };
      let saved_cpsr = self.cpsr;
      self.switch_mode(CpuMode::Irq);
      self.set_spsr(saved_cpsr);
      self.regs[14] = return_addr;
      self.cpsr.set_thumb(false);
      self.cpsr.bits |= 1 << 7;
      self.regs[15] = 0x0000_0018;
      self.pipeline_flushed = true;
    }

    pub fn software_interrupt(&mut self, _comment: u32) {
      let return_addr = if self.cpsr.thumb() {
        self.regs[15].wrapping_sub(2)
      } else {
        self.regs[15].wrapping_sub(4)
      };
      let saved_cpsr = self.cpsr;
      self.switch_mode(CpuMode::Supervisor);
      self.set_spsr(saved_cpsr);
      self.regs[14] = return_addr;
      self.cpsr.set_thumb(false);
      self.cpsr.bits |= 1 << 7;
      self.regs[15] = 0x0000_0008;
      self.pipeline_flushed = true;
    }

    #[inline]
    pub fn branch(&mut self, addr: u32) {
      self.regs[15] = addr;
      self.pipeline_flushed = true;
    }

    #[inline]
    pub fn branch_exchange(&mut self, addr: u32) {
      self.cpsr.set_thumb(addr & 1 != 0);
      self.regs[15] = addr & !1;
      self.pipeline_flushed = true;
    }

    #[inline]
    pub fn reg(&self, r: u8) -> u32 {
      self.regs[r as usize & 0xF]
    }

    #[inline]
    pub fn set_reg(&mut self, r: u8, val: u32) {
      let r = r as usize & 0xF;
      if r == 15 {
        self.branch(val & !1);
      } else {
        self.regs[r] = val;
      }
    }

    pub fn read_user_reg(&self, r: u8) -> u32 {
      let r = r as usize & 0xF;
      let mode = self.cpsr.mode();
      match r {
        0..=7 | 15 => self.regs[r],
        8..=12 => {
          if mode == CpuMode::Fiq {
            self.banked.usr_r8_r12[r - 8]
          } else {
            self.regs[r]
          }
        }
        13 => {
          if mode.bank_index() == 0 {
            self.regs[13]
          } else {
            self.banked.sp[0]
          }
        }
        14 => {
          if mode.bank_index() == 0 {
            self.regs[14]
          } else {
            self.banked.lr[0]
          }
        }
        _ => unreachable!(),
      }
    }

    pub fn write_user_reg(&mut self, r: u8, val: u32) {
      let r = r as usize & 0xF;
      let mode = self.cpsr.mode();
      match r {
        0..=7 => self.regs[r] = val,
        15 => self.regs[r] = val,
        8..=12 => {
          if mode == CpuMode::Fiq {
            self.banked.usr_r8_r12[r - 8] = val;
          } else {
            self.regs[r] = val;
          }
        }
        13 => {
          if mode.bank_index() == 0 {
            self.regs[13] = val;
          } else {
            self.banked.sp[0] = val;
          }
        }
        14 => {
          if mode.bank_index() == 0 {
            self.regs[14] = val;
          } else {
            self.banked.lr[0] = val;
          }
        }
        _ => unreachable!(),
      }
    }

    pub fn set_reg_with_flags(&mut self, r: u8, val: u32, s: bool) {
      let r_idx = r as usize & 0xF;
      if r_idx == 15 {
        if s {
          let spsr = self.spsr();
          let new_mode = spsr.mode();
          self.switch_mode(new_mode);
          self.cpsr = spsr;
        }
        self.branch(val & !1);
      } else {
        self.regs[r_idx] = val;
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn test_psr_flags() {
      let mut psr = Psr { bits: 0 };
      psr.set_n(true);
      assert!(psr.n());
      psr.set_z(true);
      assert!(psr.z());
      psr.set_c(true);
      assert!(psr.c());
      psr.set_v(true);
      assert!(psr.v());
      psr.set_nz(0);
      assert!(!psr.n());
      assert!(psr.z());
      psr.set_nz(0x80000000);
      assert!(psr.n());
      assert!(!psr.z());
    }

    #[test]
    fn test_condition_codes() {
      let cpu = Cpu::new();
      assert!(cpu.check_condition(0xE));
    }

    #[test]
    fn condition_codes_match_arm_truth_table_for_all_flag_states() {
      fn expected(cond: u32, n: bool, z: bool, c: bool, v: bool) -> bool {
        match cond {
          0x0 => z,
          0x1 => !z,
          0x2 => c,
          0x3 => !c,
          0x4 => n,
          0x5 => !n,
          0x6 => v,
          0x7 => !v,
          0x8 => c && !z,
          0x9 => !c || z,
          0xA => n == v,
          0xB => n != v,
          0xC => !z && n == v,
          0xD => z || n != v,
          0xE | 0xF => true,
          _ => unreachable!(),
        }
      }
      for flags in 0..16u32 {
        let mut cpu = Cpu::new();
        cpu.cpsr.bits = flags << 28;
        let n = flags & 8 != 0;
        let z = flags & 4 != 0;
        let c = flags & 2 != 0;
        let v = flags & 1 != 0;
        for cond in 0..16 {
          assert_eq!(cpu.check_condition(cond), expected(cond, n, z, c, v));
        }
      }
    }

    #[test]
    fn test_mode_switching() {
      let mut cpu = Cpu::new();
      cpu.cpsr = Psr::new(CpuMode::System);
      cpu.regs[13] = 0x1234;
      cpu.regs[14] = 0x5678;
      cpu.switch_mode(CpuMode::Irq);
      assert_eq!(cpu.cpsr.mode(), CpuMode::Irq);
      assert_eq!(cpu.regs[13], 0);
      assert_eq!(cpu.regs[14], 0);
      cpu.switch_mode(CpuMode::System);
      assert_eq!(cpu.regs[13], 0x1234);
      assert_eq!(cpu.regs[14], 0x5678);
    }

    #[test]
    fn test_sp_irq_preserved_with_nested_irq() {
      use crate::bus::Bus;
      let mut cpu = Cpu::new();
      cpu.cpsr = Psr::new(CpuMode::Irq);
      cpu.cpsr.bits |= 1 << 7;
      cpu.regs[13] = 0x03007F74;
      cpu.banked.sp[CpuMode::System.bank_index()] = 0x03007E20;
      let mut bus = Bus::new(None, vec![0u8; 0x100]);
      cpu.regs[3] = 0x4000_001F;
      cpu.execute_arm(&mut bus, 0xE129_F003);
      assert_eq!(cpu.cpsr.mode(), CpuMode::System);
      cpu.handle_interrupt(&mut bus);
      assert_eq!(cpu.cpsr.mode(), CpuMode::Irq);
      assert_eq!(
        cpu.regs[13], 0x03007F74,
        "nested IRQ entry: SP_irq should be outer handler's pushed value"
      );
      cpu.regs[13] -= 24;
      let saved_spsr = cpu.spsr();
      cpu.switch_mode(saved_spsr.mode());
      cpu.cpsr = saved_spsr;
      assert_eq!(cpu.cpsr.mode(), CpuMode::System);
      cpu.regs[3] = 0x4000_0092;
      cpu.execute_arm(&mut bus, 0xE129_F003);
      assert_eq!(cpu.cpsr.mode(), CpuMode::Irq);
      assert_eq!(
        cpu.regs[13], 0x03007F5C,
        "after nested IRQ + outer MSR back: SP_irq wrong"
      );
    }

    #[test]
    fn test_sp_irq_preserved_through_msr_round_trip() {
      use crate::bus::Bus;
      let mut cpu = Cpu::new();
      cpu.cpsr = Psr::new(CpuMode::Irq);
      cpu.cpsr.bits |= 1 << 7;
      cpu.regs[13] = 0x03007F74;
      cpu.banked.sp[CpuMode::System.bank_index()] = 0x03007E20;
      let mut bus = Bus::new(None, vec![0u8; 0x100]);
      cpu.regs[3] = 0x4000_001F;
      cpu.execute_arm(&mut bus, 0xE129_F003);
      assert_eq!(cpu.cpsr.mode(), CpuMode::System);
      assert_eq!(cpu.regs[13], 0x03007E20);
      cpu.regs[13] = 0x03007D00;
      cpu.regs[3] = 0x4000_0092;
      cpu.execute_arm(&mut bus, 0xE129_F003);
      assert_eq!(cpu.cpsr.mode(), CpuMode::Irq);
      assert_eq!(
        cpu.regs[13], 0x03007F74,
        "SP_irq corrupted across IRQ→System→IRQ via MSR"
      );
    }

    #[test]
    fn test_sp_irq_preserved_across_round_trip() {
      let mut cpu = Cpu::new();
      cpu.cpsr = Psr::new(CpuMode::Irq);
      cpu.regs[13] = 0x03007F74;
      cpu.banked.sp[CpuMode::System.bank_index()] = 0x03007E20;
      cpu.switch_mode(CpuMode::System);
      assert_eq!(cpu.cpsr.mode(), CpuMode::System);
      assert_eq!(cpu.regs[13], 0x03007E20);
      cpu.regs[13] = 0x03007D00;
      cpu.switch_mode(CpuMode::Irq);
      assert_eq!(cpu.cpsr.mode(), CpuMode::Irq);
      assert_eq!(
        cpu.regs[13], 0x03007F74,
        "SP_irq corrupted across IRQ→System→IRQ round trip"
      );
    }
  }
}

pub mod bus {
  pub mod io_regs {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IoRegisters {
      pub dispcnt: u16,
      pub green_swap: u16,
      pub dispstat: u16,
      pub vcount: u16,
      pub bgcnt: [u16; 4],
      pub bg_ofs: [[u16; 2]; 4],
      pub bg2_affine: [u16; 4],
      pub bg2x_latch: i32,
      pub bg2y_latch: i32,
      pub bg3_affine: [u16; 4],
      pub bg3x_latch: i32,
      pub bg3y_latch: i32,
      pub winh: [u16; 2],
      pub winv: [u16; 2],
      pub winin: u16,
      pub winout: u16,
      pub mosaic: u16,
      pub bldcnt: u16,
      pub bldalpha: u16,
      pub bldy: u16,
      pub sound_regs: Vec<u8>,
      pub siomulti: [u16; 4],
      pub siocnt: u16,
      pub siomlt_send: u16,
      pub rcnt: u16,
      pub waitcnt: u16,
      pub postflg: u8,
      pub haltcnt: u8,
    }

    impl IoRegisters {
      pub fn new() -> Self {
        IoRegisters {
          dispcnt: 0,
          green_swap: 0,
          dispstat: 0,
          vcount: 0,
          bgcnt: [0; 4],
          bg_ofs: [[0; 2]; 4],
          bg2_affine: [0; 4],
          bg2x_latch: 0,
          bg2y_latch: 0,
          bg3_affine: [0; 4],
          bg3x_latch: 0,
          bg3y_latch: 0,
          winh: [0; 2],
          winv: [0; 2],
          winin: 0,
          winout: 0,
          mosaic: 0,
          bldcnt: 0,
          bldalpha: 0,
          bldy: 0,
          sound_regs: vec![0; 0x50],
          siomulti: [0; 4],
          siocnt: 0,
          siomlt_send: 0,
          rcnt: 0,
          waitcnt: 0,
          postflg: 0,
          haltcnt: 0,
        }
      }

      fn sign_extend_28(val: u32) -> i32 {
        if val & (1 << 27) != 0 {
          (val | 0xF000_0000) as i32
        } else {
          val as i32
        }
      }

      pub fn write_bg_ref_low(&mut self, bg: usize, coord: usize, val: u16) {
        let latch = match (bg, coord) {
          (2, 0) => &mut self.bg2x_latch,
          (2, 1) => &mut self.bg2y_latch,
          (3, 0) => &mut self.bg3x_latch,
          (3, 1) => &mut self.bg3y_latch,
          _ => return,
        };
        *latch = (*latch & !0xFFFF) | val as i32;
        *latch = Self::sign_extend_28(*latch as u32);
      }

      pub fn write_bg_ref_high(&mut self, bg: usize, coord: usize, val: u16) {
        let latch = match (bg, coord) {
          (2, 0) => &mut self.bg2x_latch,
          (2, 1) => &mut self.bg2y_latch,
          (3, 0) => &mut self.bg3x_latch,
          (3, 1) => &mut self.bg3y_latch,
          _ => return,
        };
        *latch = (*latch & 0xFFFF) | ((val as i32) << 16);
        *latch = Self::sign_extend_28(*latch as u32);
      }

      pub fn write_siocnt_no_cable(&mut self, val: u16) {
        self.siocnt = (val & !0x0080) | 0x000C;
      }
    }
  }
  use crate::apu::Apu;
  use crate::backup::{self, BackupMedia};
  use crate::dma::DmaController;
  use crate::interrupt::InterruptController;
  use crate::keypad::Keypad;
  use crate::ppu::Ppu;
  use crate::rtc::Rtc;
  use crate::timer::Timers;
  use io_regs::IoRegisters;
  use serde::{Deserialize, Serialize};

  #[derive(Serialize, Deserialize)]
  pub struct Bus {
    bios: Vec<u8>,
    ewram: Vec<u8>,
    pub(crate) iwram: Vec<u8>,
    pub io: IoRegisters,
    pub ppu: Ppu,
    pub apu: Apu,
    pub palette: Vec<u8>,
    pub vram: Vec<u8>,
    pub oam: Vec<u8>,
    rom: Vec<u8>,
    pub backup: BackupMedia,
    pub dma: DmaController,
    pub timers: Timers,
    pub interrupt: InterruptController,
    pub keypad: Keypad,
    pub rtc: Rtc,
    last_read: u32,
    pub bios_latch: u32,
    pub has_bios: bool,
    pub halt_requested: bool,
    pub last_pc: u32,
    #[serde(skip)]
    pub mem_access_cycles: u32,
    #[serde(skip)]
    pub last_access_end: u32,
    #[serde(skip)]
    pub now: u64,
  }

  impl Bus {
    pub fn new(bios: Option<Vec<u8>>, rom: Vec<u8>) -> Self {
      let has_bios = bios.is_some();
      let bios_data = bios.unwrap_or_else(make_hle_bios);
      let backup = backup::detect_backup_type(&rom);
      let mut rtc = Rtc::new();
      rtc.enabled = Rtc::detect(&rom);
      Bus {
        bios: bios_data,
        ewram: vec![0; 0x40000],
        iwram: vec![0; 0x8000],
        io: IoRegisters::new(),
        ppu: Ppu::new(),
        apu: Apu::new(),
        palette: vec![0; 0x400],
        vram: vec![0; 0x18000],
        oam: vec![0; 0x400],
        rom,
        backup,
        dma: DmaController::new(),
        timers: Timers::new(),
        interrupt: InterruptController::new(),
        keypad: Keypad::new(),
        rtc,
        last_read: 0,
        bios_latch: 0xE129_F000,
        has_bios,
        halt_requested: false,
        last_pc: 0,
        mem_access_cycles: 0,
        last_access_end: 0xFFFFFFFF,
        now: 0,
      }
    }

    #[inline]
    pub fn break_sequential(&mut self) {
      self.last_access_end = 0xFFFFFFFF;
    }

    #[inline]
    pub fn add_mem_cycles(&mut self, _addr: u32, _width_bytes: u32) {}
    #[inline]
    pub fn take_mem_cycles(&mut self) -> u32 {
      std::mem::replace(&mut self.mem_access_cycles, 0)
    }

    #[inline]
    fn is_eeprom_region(&self, addr: u32) -> bool {
      matches!(self.backup, BackupMedia::Eeprom(_))
        && addr >> 24 == 0x0D
        && (self.rom.len() <= 16 * 1024 * 1024 || (addr & 0x01FF_FFFF) >= 0x01FF_FF00)
    }

    #[inline]
    pub fn read8(&mut self, addr: u32) -> u8 {
      self.add_mem_cycles(addr, 1);
      let val = match addr >> 24 {
        0x00 => self.read_bios(addr),
        0x02 => self.ewram[(addr & 0x3FFFF) as usize],
        0x03 => self.iwram[(addr & 0x7FFF) as usize],
        0x04 => self.read_io8(addr),
        0x05 => self.palette[(addr & 0x3FF) as usize],
        0x06 => self.read_vram8(addr),
        0x07 => self.oam[(addr & 0x3FF) as usize],
        0x0D if self.is_eeprom_region(addr) => self.backup.peek(addr & 0xFFFF),
        0x08..=0x0D => self.read_rom8(addr),
        0x0E..=0x0F => self.backup.read(addr & 0xFFFF),
        _ => (self.last_read & 0xFF) as u8,
      };
      self.last_read = val as u32;
      val
    }

    #[inline]
    pub fn read16(&mut self, addr: u32) -> u16 {
      let addr = addr & !1;
      self.add_mem_cycles(addr, 2);
      let val = match addr >> 24 {
        0x00 => {
          let lo = self.read_bios(addr) as u16;
          let hi = self.read_bios(addr + 1) as u16;
          lo | (hi << 8)
        }
        0x02 => {
          let base = (addr & 0x3FFFF) as usize;
          u16::from_le_bytes([self.ewram[base], self.ewram[base + 1]])
        }
        0x03 => {
          let base = (addr & 0x7FFF) as usize;
          u16::from_le_bytes([self.iwram[base], self.iwram[base + 1]])
        }
        0x04 => self.read_io16(addr),
        0x05 => {
          let base = (addr & 0x3FF) as usize;
          u16::from_le_bytes([self.palette[base], self.palette[base + 1]])
        }
        0x06 => self.read_vram16(addr),
        0x07 => {
          let base = (addr & 0x3FF) as usize;
          u16::from_le_bytes([self.oam[base], self.oam[base + 1]])
        }
        0x0D if self.is_eeprom_region(addr) => self.backup.read(addr & 0xFFFF) as u16,
        0x08..=0x0D => self.read_rom16(addr),
        0x0E..=0x0F => {
          let b = self.backup.read(addr & 0xFFFF) as u16;
          if matches!(self.backup, BackupMedia::Eeprom(_)) {
            b
          } else {
            b | (b << 8)
          }
        }
        _ => self.last_read as u16,
      };
      self.last_read = val as u32;
      val
    }

    #[inline]
    pub fn read32(&mut self, addr: u32) -> u32 {
      let addr = addr & !3;
      self.add_mem_cycles(addr, 4);
      let val = match addr >> 24 {
        0x02 => {
          let base = (addr & 0x3FFFF) as usize;
          u32::from_le_bytes([
            self.ewram[base],
            self.ewram[base + 1],
            self.ewram[base + 2],
            self.ewram[base + 3],
          ])
        }
        0x03 => {
          let base = (addr & 0x7FFF) as usize;
          u32::from_le_bytes([
            self.iwram[base],
            self.iwram[base + 1],
            self.iwram[base + 2],
            self.iwram[base + 3],
          ])
        }
        0x04 => {
          let lo = self.read_io16(addr) as u32;
          let hi = self.read_io16(addr + 2) as u32;
          lo | (hi << 16)
        }
        0x05 => {
          let base = (addr & 0x3FF) as usize;
          u32::from_le_bytes([
            self.palette[base],
            self.palette[base + 1],
            self.palette[base + 2],
            self.palette[base + 3],
          ])
        }
        0x06 => {
          let lo = self.read_vram16(addr) as u32;
          let hi = self.read_vram16(addr + 2) as u32;
          lo | (hi << 16)
        }
        0x07 => {
          let base = (addr & 0x3FF) as usize;
          u32::from_le_bytes([
            self.oam[base],
            self.oam[base + 1],
            self.oam[base + 2],
            self.oam[base + 3],
          ])
        }
        0x0D if self.is_eeprom_region(addr) => self.backup.read(addr & 0xFFFF) as u32,
        0x08..=0x0D => {
          let lo = self.read_rom16(addr) as u32;
          let hi = self.read_rom16(addr + 2) as u32;
          lo | (hi << 16)
        }
        0x0E..=0x0F => {
          let b = self.backup.read(addr & 0xFFFF) as u32;
          if matches!(self.backup, BackupMedia::Eeprom(_)) {
            b
          } else {
            b | (b << 8) | (b << 16) | (b << 24)
          }
        }
        _ => {
          let lo = self.read16(addr) as u32;
          let hi = self.read16(addr + 2) as u32;
          lo | (hi << 16)
        }
      };
      self.last_read = val;
      val
    }

    #[inline]
    pub fn write8(&mut self, addr: u32, val: u8) {
      self.add_mem_cycles(addr, 1);
      match addr >> 24 {
        0x02 => self.ewram[(addr & 0x3FFFF) as usize] = val,
        0x03 => self.iwram[(addr & 0x7FFF) as usize] = val,
        0x04 => self.write_io8(addr, val),
        0x05 => {
          let base = (addr & 0x3FE) as usize;
          self.palette[base] = val;
          self.palette[base + 1] = val;
        }
        0x06 => {
          let offset = self.vram_addr(addr);
          if offset + 1 < self.vram.len() {
            self.vram[offset] = val;
            self.vram[offset + 1] = val;
          }
        }
        0x07 => {}
        0x0D if self.is_eeprom_region(addr) => self.backup.write(addr & 0xFFFF, val),
        0x08..=0x0D => {
          if self.rtc.enabled {
            let rel = addr & 0x01FF_FFFF;
            let reg_addr = rel & !1;
            let reg_off = match reg_addr {
              0xC4 => Some(0u32),
              0xC6 => Some(2),
              0xC8 => Some(4),
              _ => None,
            };
            if let Some(off) = reg_off {
              let cur = self.rtc.read_reg(off);
              let new = if rel & 1 == 0 {
                (cur & 0xFF00) | val as u16
              } else {
                (cur & 0x00FF) | ((val as u16) << 8)
              };
              self.rtc.write_reg(off, new);
            }
          }
        }
        0x0E..=0x0F => self.backup.write(addr & 0xFFFF, val),
        _ => {}
      }
    }

    #[inline]
    pub fn write16(&mut self, addr: u32, val: u16) {
      self.add_mem_cycles(addr, 2);
      let addr = addr & !1;
      let bytes = val.to_le_bytes();
      match addr >> 24 {
        0x02 => {
          let base = (addr & 0x3FFFF) as usize;
          self.ewram[base] = bytes[0];
          self.ewram[base + 1] = bytes[1];
        }
        0x03 => {
          let base = (addr & 0x7FFF) as usize;
          self.iwram[base] = bytes[0];
          self.iwram[base + 1] = bytes[1];
        }
        0x04 => self.write_io16(addr, val),
        0x05 => {
          let base = (addr & 0x3FF) as usize;
          self.palette[base] = bytes[0];
          self.palette[base + 1] = bytes[1];
        }
        0x06 => {
          let offset = self.vram_addr(addr);
          if offset + 1 < self.vram.len() {
            self.vram[offset] = bytes[0];
            self.vram[offset + 1] = bytes[1];
          }
        }
        0x07 => {
          let base = (addr & 0x3FF) as usize;
          self.oam[base] = bytes[0];
          self.oam[base + 1] = bytes[1];
        }
        0x0D if self.is_eeprom_region(addr) => self.backup.write(addr & 0xFFFF, val as u8),
        0x08..=0x0D if self.rtc.enabled => {
          let rel = addr & 0x01FF_FFFE;
          match rel {
            0xC4 => self.rtc.write_reg(0, val),
            0xC6 => self.rtc.write_reg(2, val),
            0xC8 => self.rtc.write_reg(4, val),
            _ => {}
          }
        }
        0x08..=0x0D => {}
        _ => {}
      }
    }

    #[inline]
    pub fn write32(&mut self, addr: u32, val: u32) {
      self.add_mem_cycles(addr, 4);
      let addr = addr & !3;
      let bytes = val.to_le_bytes();
      match addr >> 24 {
        0x02 => {
          let base = (addr & 0x3FFFF) as usize;
          self.ewram[base..base + 4].copy_from_slice(&bytes);
        }
        0x03 => {
          let base = (addr & 0x7FFF) as usize;
          self.iwram[base..base + 4].copy_from_slice(&bytes);
        }
        0x04 => {
          self.write_io16(addr, val as u16);
          self.write_io16(addr + 2, (val >> 16) as u16);
        }
        0x05 => {
          let base = (addr & 0x3FF) as usize;
          self.palette[base..base + 4].copy_from_slice(&bytes);
        }
        0x06 => {
          let offset = self.vram_addr(addr);
          if offset + 3 < self.vram.len() {
            self.vram[offset..offset + 4].copy_from_slice(&bytes);
          }
        }
        0x07 => {
          let base = (addr & 0x3FF) as usize;
          self.oam[base..base + 4].copy_from_slice(&bytes);
        }
        _ => {
          self.write16(addr, val as u16);
          self.write16(addr + 2, (val >> 16) as u16);
        }
      }
    }

    pub fn iwram_mut(&mut self) -> &mut [u8] {
      &mut self.iwram
    }

    pub fn peek8(&self, addr: u32) -> u8 {
      match addr >> 24 {
        0x02 => self.ewram[(addr & 0x3FFFF) as usize],
        0x03 => self.iwram[(addr & 0x7FFF) as usize],
        0x0D if self.is_eeprom_region(addr) => self.backup.peek(addr & 0xFFFF),
        0x08..=0x0D => {
          let offset = (addr & 0x01FF_FFFF) as usize;
          if offset < self.rom.len() {
            self.rom[offset]
          } else {
            0
          }
        }
        _ => 0,
      }
    }

    pub fn backup_busy(&self) -> bool {
      self.backup.is_busy()
    }

    pub fn tick_backup(&mut self, cycles: u32) {
      self.backup.tick(cycles);
    }

    pub fn run_dma(&mut self, channel_id: usize) -> (u32, bool) {
      use crate::dma::AddrControl;
      let ch = &self.dma.channels[channel_id];
      if !ch.is_enabled() || !ch.active {
        return (0, false);
      }
      let ch = &self.dma.channels[channel_id];
      let word32 = ch.is_word_transfer();
      let word_size: u32 = if word32 { 4 } else { 2 };
      let count = ch.internal_count;
      let irq_on_done = ch.irq_enabled();
      let is_repeat = ch.repeat() && ch.timing_mode() != crate::dma::DmaTiming::Immediate;
      let src_step: i32 = match ch.src_addr_control() {
        AddrControl::Increment | AddrControl::IncrementReload => word_size as i32,
        AddrControl::Decrement => -(word_size as i32),
        AddrControl::Fixed => 0,
      };
      let dst_step: i32 = match ch.dst_addr_control() {
        AddrControl::Increment | AddrControl::IncrementReload => word_size as i32,
        AddrControl::Decrement => -(word_size as i32),
        AddrControl::Fixed => 0,
      };
      let is_fifo =
        (channel_id == 1 || channel_id == 2) && ch.timing_mode() == crate::dma::DmaTiming::Special;
      let mut src = ch.internal_sad;
      let mut dst = ch.internal_dad;
      if !word32
        && self.is_eeprom_region(dst)
        && let BackupMedia::Eeprom(e) = &mut self.backup
      {
        e.hint_transfer_bits(count);
      }
      if is_fifo {
        for _ in 0..4 {
          let val = self.read32(src);
          self.write32(dst, val);
          src = src.wrapping_add(4);
        }
        self.dma.channels[channel_id].internal_sad = src;
        return (4, irq_on_done);
      }
      for _ in 0..count {
        if word32 {
          let val = self.read32(src & !3);
          self.write32(dst & !3, val);
        } else {
          let val = self.read16(src & !1);
          self.write16(dst & !1, val);
        }
        src = (src as i32).wrapping_add(src_step) as u32;
        dst = (dst as i32).wrapping_add(dst_step) as u32;
      }
      self.dma.channels[channel_id].internal_sad = src;
      self.dma.channels[channel_id].internal_dad = dst;
      if is_repeat {
        self.dma.channels[channel_id].reload_for_repeat(channel_id);
      } else {
        self.dma.channels[channel_id].control &= !(1 << 15);
        self.dma.channels[channel_id].active = false;
      }
      (count, irq_on_done)
    }

    pub fn write_dma_control(&mut self, channel_id: usize, value: u16) -> Option<usize> {
      self.dma.write_control(channel_id, value, self.now)
    }

    fn read_bios(&mut self, addr: u32) -> u8 {
      if self.last_pc < 0x0000_4000 {
        let index = (addr & 0x3FFF) as usize;
        if index + 3 < self.bios.len() {
          let word_idx = index & !3;
          self.bios_latch = u32::from_le_bytes([
            self.bios[word_idx],
            self.bios[word_idx + 1],
            self.bios[word_idx + 2],
            self.bios[word_idx + 3],
          ]);
          self.bios[index]
        } else if index < self.bios.len() {
          self.bios[index]
        } else {
          0
        }
      } else {
        let shift = (addr & 3) * 8;
        ((self.bios_latch >> shift) & 0xFF) as u8
      }
    }

    fn read_vram8(&self, addr: u32) -> u8 {
      let offset = self.vram_addr(addr);
      if offset < self.vram.len() {
        self.vram[offset]
      } else {
        0
      }
    }

    fn read_vram16(&self, addr: u32) -> u16 {
      let offset = self.vram_addr(addr & !1);
      if offset + 1 < self.vram.len() {
        u16::from_le_bytes([self.vram[offset], self.vram[offset + 1]])
      } else {
        0
      }
    }

    fn vram_addr(&self, addr: u32) -> usize {
      let offset = (addr & 0x1FFFF) as usize;
      if offset >= 0x18000 {
        offset - 0x8000
      } else {
        offset
      }
    }

    fn read_rom8(&self, addr: u32) -> u8 {
      if self.rtc.enabled {
        let rel = addr & 0x01FF_FFFF;
        match rel {
          0xC4 => return self.rtc.read_reg(0) as u8,
          0xC5 => return (self.rtc.read_reg(0) >> 8) as u8,
          0xC6 => return self.rtc.read_reg(2) as u8,
          0xC7 => return (self.rtc.read_reg(2) >> 8) as u8,
          0xC8 => return self.rtc.read_reg(4) as u8,
          0xC9 => return (self.rtc.read_reg(4) >> 8) as u8,
          _ => {}
        }
      }
      let offset = (addr & 0x01FF_FFFF) as usize;
      if offset < self.rom.len() {
        self.rom[offset]
      } else {
        ((offset >> 1) & 0xFF) as u8
      }
    }

    fn read_rom16(&self, addr: u32) -> u16 {
      if self.rtc.enabled {
        let rel = addr & 0x01FF_FFFE;
        match rel {
          0xC4 => return self.rtc.read_reg(0),
          0xC6 => return self.rtc.read_reg(2),
          0xC8 => return self.rtc.read_reg(4),
          _ => {}
        }
      }
      let offset = (addr & 0x01FF_FFFE) as usize;
      if offset + 1 < self.rom.len() {
        u16::from_le_bytes([self.rom[offset], self.rom[offset + 1]])
      } else {
        (offset >> 1) as u16
      }
    }

    fn read_io8(&mut self, addr: u32) -> u8 {
      let val16 = self.read_io16(addr & !1);
      if addr & 1 == 0 {
        val16 as u8
      } else {
        (val16 >> 8) as u8
      }
    }

    fn read_io16(&mut self, addr: u32) -> u16 {
      match addr & 0x3FF {
        0x000 => self.io.dispcnt,
        0x002 => self.io.green_swap,
        0x004 => self.io.dispstat,
        0x006 => self.io.vcount,
        0x008 => self.io.bgcnt[0],
        0x00A => self.io.bgcnt[1],
        0x00C => self.io.bgcnt[2],
        0x00E => self.io.bgcnt[3],
        0x010..=0x01E => 0,
        0x020..=0x03E => 0,
        0x040 => self.io.winh[0],
        0x042 => self.io.winh[1],
        0x044 => self.io.winv[0],
        0x046 => self.io.winv[1],
        0x048 => self.io.winin,
        0x04A => self.io.winout,
        0x04C => self.io.mosaic,
        0x050 => self.io.bldcnt,
        0x052 => self.io.bldalpha,
        0x054 => 0,
        0x060..=0x0A8 => {
          let offset = (addr & 0x3FF) - 0x60;
          self.apu.read_reg(offset as u16)
        }
        0x0B0..=0x0DE => 0,
        0x100 => self.timers.read_counter(0),
        0x102 => self.timers.timers[0].control,
        0x104 => self.timers.read_counter(1),
        0x106 => self.timers.timers[1].control,
        0x108 => self.timers.read_counter(2),
        0x10A => self.timers.timers[2].control,
        0x10C => self.timers.read_counter(3),
        0x10E => self.timers.timers[3].control,
        0x120 => self.io.siomulti[0],
        0x122 => self.io.siomulti[1],
        0x124 => self.io.siomulti[2],
        0x126 => self.io.siomulti[3],
        0x128 => self.io.siocnt,
        0x12A => self.io.siomlt_send,
        0x134 => self.io.rcnt,
        0x130 => self.keypad.read_keyinput(),
        0x132 => self.keypad.keycnt,
        0x200 => self.interrupt.read_ie(),
        0x202 => self.interrupt.read_if(),
        0x208 => self.interrupt.read_ime(),
        0x204 => self.io.waitcnt,
        0x300 => self.io.postflg as u16,
        _ => 0,
      }
    }

    fn write_io8(&mut self, addr: u32, val: u8) {
      if addr & 0x3FF == 0x301 {
        self.io.haltcnt = val;
        self.halt_requested = true;
        return;
      }
      let aligned = addr & !1;
      let current = self.read_io16(aligned);
      let new_val = if addr & 1 == 0 {
        (current & 0xFF00) | val as u16
      } else {
        (current & 0x00FF) | ((val as u16) << 8)
      };
      self.write_io16(aligned, new_val);
    }

    fn write_io16(&mut self, addr: u32, val: u16) {
      match addr & 0x3FF {
        0x000 => self.io.dispcnt = val,
        0x002 => self.io.green_swap = val,
        0x004 => {
          self.io.dispstat = (self.io.dispstat & 0x07) | (val & !0x07);
        }
        0x006 => {}
        0x008 => self.io.bgcnt[0] = val,
        0x00A => self.io.bgcnt[1] = val,
        0x00C => self.io.bgcnt[2] = val,
        0x00E => self.io.bgcnt[3] = val,
        0x010 => self.io.bg_ofs[0][0] = val & 0x1FF,
        0x012 => self.io.bg_ofs[0][1] = val & 0x1FF,
        0x014 => self.io.bg_ofs[1][0] = val & 0x1FF,
        0x016 => self.io.bg_ofs[1][1] = val & 0x1FF,
        0x018 => self.io.bg_ofs[2][0] = val & 0x1FF,
        0x01A => self.io.bg_ofs[2][1] = val & 0x1FF,
        0x01C => self.io.bg_ofs[3][0] = val & 0x1FF,
        0x01E => self.io.bg_ofs[3][1] = val & 0x1FF,
        0x020 => self.io.bg2_affine[0] = val,
        0x022 => self.io.bg2_affine[1] = val,
        0x024 => self.io.bg2_affine[2] = val,
        0x026 => self.io.bg2_affine[3] = val,
        0x028 => {
          self.io.write_bg_ref_low(2, 0, val);
          self.ppu.reload_bg_ref(2, 0, &self.io);
        }
        0x02A => {
          self.io.write_bg_ref_high(2, 0, val);
          self.ppu.reload_bg_ref(2, 0, &self.io);
        }
        0x02C => {
          self.io.write_bg_ref_low(2, 1, val);
          self.ppu.reload_bg_ref(2, 1, &self.io);
        }
        0x02E => {
          self.io.write_bg_ref_high(2, 1, val);
          self.ppu.reload_bg_ref(2, 1, &self.io);
        }
        0x030 => self.io.bg3_affine[0] = val,
        0x032 => self.io.bg3_affine[1] = val,
        0x034 => self.io.bg3_affine[2] = val,
        0x036 => self.io.bg3_affine[3] = val,
        0x038 => {
          self.io.write_bg_ref_low(3, 0, val);
          self.ppu.reload_bg_ref(3, 0, &self.io);
        }
        0x03A => {
          self.io.write_bg_ref_high(3, 0, val);
          self.ppu.reload_bg_ref(3, 0, &self.io);
        }
        0x03C => {
          self.io.write_bg_ref_low(3, 1, val);
          self.ppu.reload_bg_ref(3, 1, &self.io);
        }
        0x03E => {
          self.io.write_bg_ref_high(3, 1, val);
          self.ppu.reload_bg_ref(3, 1, &self.io);
        }
        0x040 => self.io.winh[0] = val,
        0x042 => self.io.winh[1] = val,
        0x044 => self.io.winv[0] = val,
        0x046 => self.io.winv[1] = val,
        0x048 => self.io.winin = val,
        0x04A => self.io.winout = val,
        0x04C => self.io.mosaic = val,
        0x050 => self.io.bldcnt = val,
        0x052 => self.io.bldalpha = val,
        0x054 => self.io.bldy = val,
        0x060..=0x0A8 => {
          let offset = (addr & 0x3FF) - 0x60;
          self.apu.write_reg(offset as u16, val);
        }
        0x0B0 => self.dma.channels[0].sad = (self.dma.channels[0].sad & 0xFFFF0000) | val as u32,
        0x0B2 => {
          self.dma.channels[0].sad = (self.dma.channels[0].sad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0B4 => self.dma.channels[0].dad = (self.dma.channels[0].dad & 0xFFFF0000) | val as u32,
        0x0B6 => {
          self.dma.channels[0].dad = (self.dma.channels[0].dad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0B8 => self.dma.channels[0].count = val,
        0x0BA => {
          if let Some(_ch) = self.write_dma_control(0, val) {
            self.run_dma(0);
          }
        }
        0x0BC => self.dma.channels[1].sad = (self.dma.channels[1].sad & 0xFFFF0000) | val as u32,
        0x0BE => {
          self.dma.channels[1].sad = (self.dma.channels[1].sad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0C0 => self.dma.channels[1].dad = (self.dma.channels[1].dad & 0xFFFF0000) | val as u32,
        0x0C2 => {
          self.dma.channels[1].dad = (self.dma.channels[1].dad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0C4 => self.dma.channels[1].count = val,
        0x0C6 => {
          if let Some(_ch) = self.write_dma_control(1, val) {
            self.run_dma(1);
          }
        }
        0x0C8 => self.dma.channels[2].sad = (self.dma.channels[2].sad & 0xFFFF0000) | val as u32,
        0x0CA => {
          self.dma.channels[2].sad = (self.dma.channels[2].sad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0CC => self.dma.channels[2].dad = (self.dma.channels[2].dad & 0xFFFF0000) | val as u32,
        0x0CE => {
          self.dma.channels[2].dad = (self.dma.channels[2].dad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0D0 => self.dma.channels[2].count = val,
        0x0D2 => {
          if let Some(_ch) = self.write_dma_control(2, val) {
            self.run_dma(2);
          }
        }
        0x0D4 => self.dma.channels[3].sad = (self.dma.channels[3].sad & 0xFFFF0000) | val as u32,
        0x0D6 => {
          self.dma.channels[3].sad = (self.dma.channels[3].sad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0D8 => self.dma.channels[3].dad = (self.dma.channels[3].dad & 0xFFFF0000) | val as u32,
        0x0DA => {
          self.dma.channels[3].dad = (self.dma.channels[3].dad & 0x0000FFFF) | ((val as u32) << 16)
        }
        0x0DC => self.dma.channels[3].count = val,
        0x0DE => {
          if let Some(_ch) = self.write_dma_control(3, val) {
            self.run_dma(3);
          }
        }
        0x100 => self.timers.write_reload(0, val),
        0x102 => self.timers.write_control(0, val),
        0x104 => self.timers.write_reload(1, val),
        0x106 => self.timers.write_control(1, val),
        0x108 => self.timers.write_reload(2, val),
        0x10A => self.timers.write_control(2, val),
        0x10C => self.timers.write_reload(3, val),
        0x10E => self.timers.write_control(3, val),
        0x120 => self.io.siomulti[0] = val,
        0x122 => self.io.siomulti[1] = val,
        0x124 => self.io.siomulti[2] = val,
        0x126 => self.io.siomulti[3] = val,
        0x128 => {
          self.io.write_siocnt_no_cable(val);
          if val & 0x4080 == 0x4080 {
            self.interrupt.request_irq(crate::interrupt::Irq::Serial);
          }
        }
        0x12A => self.io.siomlt_send = val,
        0x132 => self.keypad.keycnt = val,
        0x134 => self.io.rcnt = val,
        0x200 => self.interrupt.write_ie(val),
        0x202 => self.interrupt.write_if(val),
        0x208 => self.interrupt.write_ime(val),
        0x204 => self.io.waitcnt = val,
        0x300 => self.io.postflg = val as u8,
        _ => {}
      }
    }
  }

  fn make_hle_bios() -> Vec<u8> {
    let mut bios = vec![0u8; 0x4000];
    let stub: [(u32, u32); 6] = [
      (0x18, 0xE92D500F),
      (0x1C, 0xE3A00404),
      (0x20, 0xE28FE000),
      (0x24, 0xE510F004),
      (0x28, 0xE8BD500F),
      (0x2C, 0xE25EF004),
    ];
    for (addr, opcode) in stub.iter() {
      let a = *addr as usize;
      let bytes = opcode.to_le_bytes();
      bios[a] = bytes[0];
      bios[a + 1] = bytes[1];
      bios[a + 2] = bytes[2];
      bios[a + 3] = bytes[3];
    }
    let post_latch = 0xE3A02004u32.to_le_bytes();
    bios[0x30] = post_latch[0];
    bios[0x31] = post_latch[1];
    bios[0x32] = post_latch[2];
    bios[0x33] = post_latch[3];
    bios
  }
}

pub mod ppu {
  pub mod bg {
    use crate::SCREEN_WIDTH;
    use crate::bus::io_regs::IoRegisters;
    use crate::ppu::PixelInfo;
    const TEXT_SCREEN_SIZES: [(u32, u32); 4] = [(32, 32), (64, 32), (32, 64), (64, 64)];
    const AFFINE_SCREEN_SIZES: [u32; 4] = [16, 32, 64, 128];

    #[derive(Clone, Copy)]
    pub struct BgCtx<'a> {
      pub io: &'a IoRegisters,
      pub palette: &'a [u8],
      pub vram: &'a [u8],
    }

    pub fn render_text_bg_line(
      bg: usize,
      line: u16,
      io: &IoRegisters,
      palette: &[u8],
      vram: &[u8],
      output: &mut [Option<PixelInfo>; 240],
    ) {
      let bgcnt = io.bgcnt[bg];
      let priority = (bgcnt & 3) as u8;
      let char_base = (((bgcnt >> 2) & 3) as usize) * 0x4000;
      let mosaic = bgcnt & (1 << 6) != 0;
      let bpp8 = bgcnt & (1 << 7) != 0;
      let screen_base = (((bgcnt >> 8) & 0x1F) as usize) * 0x800;
      let size_idx = ((bgcnt >> 14) & 3) as usize;
      let (screen_w, screen_h) = TEXT_SCREEN_SIZES[size_idx];
      let scroll_x = io.bg_ofs[bg][0] as u32;
      let scroll_y = io.bg_ofs[bg][1] as u32;
      let y = if mosaic {
        let mos_h = ((io.mosaic >> 4) & 0xF) as u32 + 1;
        ((line as u32 + scroll_y) / mos_h) * mos_h
      } else {
        line as u32 + scroll_y
      };
      let tile_y = (y >> 3) & (screen_h - 1);
      let pixel_y = y & 7;
      for (screen_x, out) in output.iter_mut().enumerate().take(SCREEN_WIDTH) {
        let x = if mosaic {
          let mos_w = (io.mosaic & 0xF) as u32 + 1;
          (((screen_x as u32 + scroll_x) / mos_w) * mos_w) % (screen_w * 8)
        } else {
          (screen_x as u32 + scroll_x) % (screen_w * 8)
        };
        let tile_x = x >> 3;
        let pixel_x = x & 7;
        let screen_block_offset = get_text_screen_block_offset(tile_x, tile_y, screen_w, screen_h);
        let map_addr =
          screen_base + screen_block_offset + (((tile_y & 31) << 5) + (tile_x & 31)) as usize * 2;
        if map_addr + 1 >= vram.len() {
          *out = None;
          continue;
        }
        let map_entry = u16::from_le_bytes([vram[map_addr], vram[map_addr + 1]]);
        let tile_num = (map_entry & 0x3FF) as usize;
        let h_flip = map_entry & (1 << 10) != 0;
        let v_flip = map_entry & (1 << 11) != 0;
        let pal_num = ((map_entry >> 12) & 0xF) as usize;
        let px = if h_flip { 7 - pixel_x } else { pixel_x };
        let py = if v_flip { 7 - pixel_y } else { pixel_y };
        let color_index = if bpp8 {
          let tile_addr = char_base + tile_num * 64 + py as usize * 8 + px as usize;
          if tile_addr < vram.len() {
            vram[tile_addr] as usize
          } else {
            0
          }
        } else {
          let tile_addr = char_base + tile_num * 32 + py as usize * 4 + ((px as usize) >> 1);
          if tile_addr < vram.len() {
            let byte = vram[tile_addr];
            if px & 1 == 0 {
              (byte & 0x0F) as usize
            } else {
              (byte >> 4) as usize
            }
          } else {
            0
          }
        };
        if color_index == 0 {
          *out = None;
          continue;
        }
        let pal_offset = if bpp8 {
          color_index * 2
        } else {
          (pal_num * 16 + color_index) * 2
        };
        if pal_offset + 1 < palette.len() {
          let color = u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]) & 0x7FFF;
          *out = Some(PixelInfo {
            color,
            priority,
            layer: bg as u8,
            semi_transparent: false,
          });
        } else {
          *out = None;
        }
      }
    }

    fn get_text_screen_block_offset(
      tile_x: u32,
      tile_y: u32,
      screen_w: u32,
      _screen_h: u32,
    ) -> usize {
      let block_x = tile_x >> 5;
      let block_y = tile_y >> 5;
      let block_index = match screen_w {
        64 => block_x + block_y * 2,
        32 => block_y,
        _ => 0,
      };
      (block_index as usize) * 0x800
    }

    pub fn render_affine_bg_line(
      bg: usize,
      ctx: BgCtx<'_>,
      ref_x: i32,
      ref_y: i32,
      output: &mut [Option<PixelInfo>; 240],
    ) {
      let io = ctx.io;
      let palette = ctx.palette;
      let vram = ctx.vram;
      let bgcnt = io.bgcnt[bg];
      let priority = (bgcnt & 3) as u8;
      let char_base = (((bgcnt >> 2) & 3) as usize) * 0x4000;
      let screen_base = (((bgcnt >> 8) & 0x1F) as usize) * 0x800;
      let wrap = bgcnt & (1 << 13) != 0;
      let size_idx = ((bgcnt >> 14) & 3) as usize;
      let map_size = AFFINE_SCREEN_SIZES[size_idx];
      let pixel_size = map_size * 8;
      let (pa, _pb, pc, _pd) = get_affine_params(bg, io);
      let mut tex_x = ref_x;
      let mut tex_y = ref_y;
      for out in output.iter_mut().take(SCREEN_WIDTH) {
        let sx = tex_x >> 8;
        let sy = tex_y >> 8;
        let (fx, fy) = if wrap {
          (
            ((sx % pixel_size as i32) + pixel_size as i32) as u32 & (pixel_size - 1),
            ((sy % pixel_size as i32) + pixel_size as i32) as u32 & (pixel_size - 1),
          )
        } else {
          if sx < 0 || sy < 0 || sx >= pixel_size as i32 || sy >= pixel_size as i32 {
            *out = None;
            tex_x += pa as i32;
            tex_y += pc as i32;
            continue;
          }
          (sx as u32, sy as u32)
        };
        let tile_x = fx >> 3;
        let tile_y = fy >> 3;
        let pixel_x = (fx & 7) as usize;
        let pixel_y = (fy & 7) as usize;
        let map_addr = screen_base + (tile_y * map_size + tile_x) as usize;
        let tile_num = if map_addr < vram.len() {
          vram[map_addr] as usize
        } else {
          0
        };
        let tile_addr = char_base + tile_num * 64 + pixel_y * 8 + pixel_x;
        let color_index = if tile_addr < vram.len() {
          vram[tile_addr] as usize
        } else {
          0
        };
        if color_index == 0 {
          *out = None;
        } else {
          let pal_offset = color_index * 2;
          if pal_offset + 1 < palette.len() {
            let color = u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]) & 0x7FFF;
            *out = Some(PixelInfo {
              color,
              priority,
              layer: bg as u8,
              semi_transparent: false,
            });
          } else {
            *out = None;
          }
        }
        tex_x += pa as i32;
        tex_y += pc as i32;
      }
    }

    pub fn get_affine_params(bg: usize, io: &IoRegisters) -> (i16, i16, i16, i16) {
      if bg == 2 {
        (
          io.bg2_affine[0] as i16,
          io.bg2_affine[1] as i16,
          io.bg2_affine[2] as i16,
          io.bg2_affine[3] as i16,
        )
      } else {
        (
          io.bg3_affine[0] as i16,
          io.bg3_affine[1] as i16,
          io.bg3_affine[2] as i16,
          io.bg3_affine[3] as i16,
        )
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_text_bg_basic() {
        let mut io = IoRegisters::new();
        io.bgcnt[0] = 0x0000;
        let mut palette = vec![0u8; 0x400];
        palette[2] = 0x1F;
        palette[3] = 0x00;
        let mut vram = vec![0u8; 0x18000];
        vram[0] = 1;
        vram[1] = 0;
        let tile_offset = 32;
        vram[tile_offset] = 0x01;
        let mut output = [None; 240];
        render_text_bg_line(0, 0, &io, &palette, &vram, &mut output);
        assert!(output[0].is_some());
        assert_eq!(output[0].unwrap().color, 0x001F);
        assert!(output[1].is_none());
      }

      #[test]
      fn text_bg_4bpp_uses_low_then_high_nibble() {
        let mut io = IoRegisters::new();
        io.bgcnt[0] = 0x0000;
        let mut palette = vec![0u8; 0x400];
        palette[2] = 0x1F;
        palette[4] = 0xE0;
        palette[5] = 0x03;
        let mut vram = vec![0u8; 0x18000];
        vram[0] = 1;
        vram[32] = 0x21;
        let mut output = [None; 240];
        render_text_bg_line(0, 0, &io, &palette, &vram, &mut output);
        assert_eq!(output[0].unwrap().color, 0x001F);
        assert_eq!(output[1].unwrap().color, 0x03E0);
      }

      #[test]
      fn test_text_bg_hflip() {
        let mut io = IoRegisters::new();
        io.bgcnt[0] = 0x0000;
        let mut palette = vec![0u8; 0x400];
        palette[2] = 0x1F;
        let mut vram = vec![0u8; 0x18000];
        vram[0] = 1;
        vram[1] = 0x04;
        let tile_offset = 32;
        vram[tile_offset] = 0x01;
        let mut output = [None; 240];
        render_text_bg_line(0, 0, &io, &palette, &vram, &mut output);
        assert!(output[0].is_none());
        assert!(output[7].is_some());
        assert_eq!(output[7].unwrap().color, 0x001F);
      }

      #[test]
      fn test_text_bg_scrolling() {
        let mut io = IoRegisters::new();
        io.bgcnt[0] = 0x0000;
        io.bg_ofs[0][0] = 4;
        let mut palette = vec![0u8; 0x400];
        palette[2] = 0x1F;
        let mut vram = vec![0u8; 0x18000];
        vram[0] = 1;
        vram[1] = 0;
        let tile_offset = 32;
        vram[tile_offset] = 0x01;
        let mut output = [None; 240];
        render_text_bg_line(0, 0, &io, &palette, &vram, &mut output);
        assert!(output[0].is_none());
      }
    }
  }
  pub mod obj {
    use crate::SCREEN_WIDTH;
    use crate::bus::io_regs::IoRegisters;
    use crate::ppu::PixelInfo;
    const OBJ_SIZES: [[(u32, u32); 4]; 3] = [
      [(8, 8), (16, 16), (32, 32), (64, 64)],
      [(16, 8), (32, 8), (32, 16), (64, 32)],
      [(8, 16), (8, 32), (16, 32), (32, 64)],
    ];
    #[allow(dead_code)]
    struct ObjAttr {
      y: i32,
      x: i32,
      mode: u8,
      gfx_mode: u8,
      mosaic: bool,
      bpp8: bool,
      shape: u8,
      size: u8,
      h_flip: bool,
      v_flip: bool,
      affine_param: u8,
      tile_num: u16,
      priority: u8,
      palette: u8,
      width: u32,
      height: u32,
    }

    fn parse_obj(oam: &[u8], index: usize) -> ObjAttr {
      let base = index * 8;
      let attr0 = u16::from_le_bytes([oam[base], oam[base + 1]]);
      let attr1 = u16::from_le_bytes([oam[base + 2], oam[base + 3]]);
      let attr2 = u16::from_le_bytes([oam[base + 4], oam[base + 5]]);
      let y = (attr0 & 0xFF) as i32;
      let mode = ((attr0 >> 8) & 3) as u8;
      let gfx_mode = ((attr0 >> 10) & 3) as u8;
      let mosaic = attr0 & (1 << 12) != 0;
      let bpp8 = attr0 & (1 << 13) != 0;
      let shape = ((attr0 >> 14) & 3) as u8;
      let x_raw = (attr1 & 0x1FF) as i32;
      let x = if x_raw >= 256 { x_raw - 512 } else { x_raw };
      let is_affine = mode == 1 || mode == 3;
      let h_flip = if !is_affine {
        attr1 & (1 << 12) != 0
      } else {
        false
      };
      let v_flip = if !is_affine {
        attr1 & (1 << 13) != 0
      } else {
        false
      };
      let affine_param = if is_affine {
        ((attr1 >> 9) & 0x1F) as u8
      } else {
        0
      };
      let size = ((attr1 >> 14) & 3) as u8;
      let tile_num = attr2 & 0x3FF;
      let priority = ((attr2 >> 10) & 3) as u8;
      let palette = ((attr2 >> 12) & 0xF) as u8;
      let shape_idx = (shape as usize).min(2);
      let size_idx = (size as usize).min(3);
      let (width, height) = OBJ_SIZES[shape_idx][size_idx];
      ObjAttr {
        y,
        x,
        mode,
        gfx_mode,
        mosaic,
        bpp8,
        shape,
        size,
        h_flip,
        v_flip,
        affine_param,
        tile_num,
        priority,
        palette,
        width,
        height,
      }
    }

    fn read_affine_params(oam: &[u8], group: u8) -> (i16, i16, i16, i16) {
      let base = group as usize * 32;
      let pa = i16::from_le_bytes([oam[base + 6], oam[base + 7]]);
      let pb = i16::from_le_bytes([oam[base + 14], oam[base + 15]]);
      let pc = i16::from_le_bytes([oam[base + 22], oam[base + 23]]);
      let pd = i16::from_le_bytes([oam[base + 30], oam[base + 31]]);
      (pa, pb, pc, pd)
    }

    pub fn render_obj_line(
      line: u16,
      io: &IoRegisters,
      palette: &[u8],
      vram: &[u8],
      oam: &[u8],
      output: &mut [Option<PixelInfo>; 240],
    ) {
      let mapping_1d = io.dispcnt & (1 << 6) != 0;
      let obj_vram_base: usize = 0x10000;
      let obj_pal_base: usize = 0x200;
      for i in 0..128 {
        let obj = parse_obj(oam, i);
        if obj.mode == 2 {
          continue;
        }
        if obj.gfx_mode == 2 {
          continue;
        }
        let is_affine = obj.mode == 1 || obj.mode == 3;
        let double_size = obj.mode == 3;
        let (bound_w, bound_h) = if double_size {
          (obj.width * 2, obj.height * 2)
        } else {
          (obj.width, obj.height)
        };
        let obj_y = if obj.y >= 160 && obj.y < 256 {
          obj.y - 256
        } else {
          obj.y
        };
        let local_y = line as i32 - obj_y;
        if local_y < 0 || local_y >= bound_h as i32 {
          continue;
        }
        for lx in 0..bound_w as i32 {
          let screen_x = obj.x + lx;
          if screen_x < 0 || screen_x >= SCREEN_WIDTH as i32 {
            continue;
          }
          let sx = screen_x as usize;
          if let Some(existing) = &output[sx]
            && existing.layer == 4
            && existing.priority <= obj.priority
          {
            continue;
          }
          let (tex_x, tex_y) = if is_affine {
            let (pa, pb, pc, pd) = read_affine_params(oam, obj.affine_param);
            let half_w = obj.width as i32 / 2;
            let half_h = obj.height as i32 / 2;
            let cx = lx - bound_w as i32 / 2;
            let cy = local_y - bound_h as i32 / 2;
            let tx = ((pa as i32 * cx + pb as i32 * cy) >> 8) + half_w;
            let ty = ((pc as i32 * cx + pd as i32 * cy) >> 8) + half_h;
            if tx < 0 || ty < 0 || tx >= obj.width as i32 || ty >= obj.height as i32 {
              continue;
            }
            (tx as u32, ty as u32)
          } else {
            let tx = if obj.h_flip {
              obj.width - 1 - lx as u32
            } else {
              lx as u32
            };
            let ty = if obj.v_flip {
              obj.height - 1 - local_y as u32
            } else {
              local_y as u32
            };
            (tx, ty)
          };
          let tile_x = tex_x >> 3;
          let tile_y = tex_y >> 3;
          let pixel_x = (tex_x & 7) as usize;
          let pixel_y = (tex_y & 7) as usize;
          let tile_offset = if mapping_1d {
            let base_tile = obj.tile_num as u32;
            let tile_idx = if obj.bpp8 {
              base_tile + tile_y * (obj.width >> 3) * 2 + tile_x * 2
            } else {
              base_tile + tile_y * (obj.width >> 3) + tile_x
            };
            tile_idx as usize
          } else {
            let base_tile = obj.tile_num as u32;
            let tile_idx = if obj.bpp8 {
              base_tile + tile_y * 32 + tile_x * 2
            } else {
              base_tile + tile_y * 32 + tile_x
            };
            tile_idx as usize
          };
          let color_index = if obj.bpp8 {
            let addr = obj_vram_base + tile_offset * 32 + pixel_y * 8 + pixel_x;
            if addr < vram.len() {
              vram[addr] as usize
            } else {
              0
            }
          } else {
            let addr = obj_vram_base + tile_offset * 32 + pixel_y * 4 + (pixel_x >> 1);
            if addr < vram.len() {
              let byte = vram[addr];
              if pixel_x & 1 == 0 {
                (byte & 0x0F) as usize
              } else {
                (byte >> 4) as usize
              }
            } else {
              0
            }
          };
          if color_index == 0 {
            continue;
          }
          let pal_offset = if obj.bpp8 {
            obj_pal_base + color_index * 2
          } else {
            obj_pal_base + (obj.palette as usize * 16 + color_index) * 2
          };
          if pal_offset + 1 < palette.len() {
            let color = u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]) & 0x7FFF;
            output[sx] = Some(PixelInfo {
              color,
              priority: obj.priority,
              layer: 4,
              semi_transparent: obj.gfx_mode == 1,
            });
          }
        }
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_obj_sizes() {
        assert_eq!(OBJ_SIZES[0][0], (8, 8));
        assert_eq!(OBJ_SIZES[0][3], (64, 64));
        assert_eq!(OBJ_SIZES[1][0], (16, 8));
        assert_eq!(OBJ_SIZES[2][3], (32, 64));
      }

      #[test]
      fn test_parse_obj_disabled() {
        let mut oam = vec![0u8; 0x400];
        oam[1] = 0x02;
        let obj = parse_obj(&oam, 0);
        assert_eq!(obj.mode, 2);
      }

      #[test]
      fn test_render_simple_sprite() {
        let mut io = IoRegisters::new();
        io.dispcnt = 0x1040;
        let mut palette = vec![0u8; 0x400];
        let pal_addr = 0x200 + 2;
        palette[pal_addr] = 0xE0;
        palette[pal_addr + 1] = 0x03;
        let mut vram = vec![0u8; 0x18000];
        let tile_addr = 0x10000;
        vram[tile_addr] = 0x01;
        let mut oam = vec![0u8; 0x400];
        oam[0] = 0;
        oam[1] = 0;
        oam[2] = 0;
        oam[3] = 0;
        oam[4] = 0;
        oam[5] = 0;
        let mut output = [None; 240];
        render_obj_line(0, &io, &palette, &vram, &oam, &mut output);
        assert!(output[0].is_some());
        assert_eq!(output[0].unwrap().color, 0x03E0);
        assert_eq!(output[0].unwrap().layer, 4);
      }

      #[test]
      fn render_sprite_4bpp_uses_low_then_high_nibble() {
        let mut io = IoRegisters::new();
        io.dispcnt = 0x1040;
        let mut palette = vec![0u8; 0x400];
        let pal_addr = 0x200;
        palette[pal_addr + 2] = 0x1F;
        palette[pal_addr + 4] = 0xE0;
        palette[pal_addr + 5] = 0x03;
        let mut vram = vec![0u8; 0x18000];
        vram[0x10000] = 0x21;
        let oam = vec![0u8; 0x400];
        let mut output = [None; 240];
        render_obj_line(0, &io, &palette, &vram, &oam, &mut output);
        assert_eq!(output[0].unwrap().color, 0x001F);
        assert_eq!(output[1].unwrap().color, 0x03E0);
      }
    }
  }
  pub mod window {
    use crate::bus::io_regs::IoRegisters;

    #[derive(Debug, Clone, Copy)]
    pub struct WindowFlags {
      pub bg_enable: [bool; 4],
      pub obj_enable: bool,
      pub effects_enable: bool,
    }

    impl WindowFlags {
      fn from_bits(bits: u8) -> Self {
        WindowFlags {
          bg_enable: [
            bits & (1 << 0) != 0,
            bits & (1 << 1) != 0,
            bits & (1 << 2) != 0,
            bits & (1 << 3) != 0,
          ],
          obj_enable: bits & (1 << 4) != 0,
          effects_enable: bits & (1 << 5) != 0,
        }
      }

      pub fn all_enabled() -> Self {
        WindowFlags {
          bg_enable: [true; 4],
          obj_enable: true,
          effects_enable: true,
        }
      }
    }

    pub fn compute_window_line(
      line: u16,
      io: &IoRegisters,
      obj_window_line: &[bool; 240],
    ) -> Option<[WindowFlags; 240]> {
      let dispcnt = io.dispcnt;
      let win0_enabled = dispcnt & (1 << 13) != 0;
      let win1_enabled = dispcnt & (1 << 14) != 0;
      let objwin_enabled = dispcnt & (1 << 15) != 0;
      if !win0_enabled && !win1_enabled && !objwin_enabled {
        return None;
      }
      let win0_flags = WindowFlags::from_bits((io.winin & 0x3F) as u8);
      let win1_flags = WindowFlags::from_bits(((io.winin >> 8) & 0x3F) as u8);
      let outside_flags = WindowFlags::from_bits((io.winout & 0x3F) as u8);
      let objwin_flags = WindowFlags::from_bits(((io.winout >> 8) & 0x3F) as u8);
      let win0_y1 = io.winv[0] >> 8;
      let win0_y2 = io.winv[0] & 0xFF;
      let win0_in_v = in_wrapped_range(line, win0_y1, win0_y2);
      let win1_y1 = io.winv[1] >> 8;
      let win1_y2 = io.winv[1] & 0xFF;
      let win1_in_v = in_wrapped_range(line, win1_y1, win1_y2);
      let win0_x1 = io.winh[0] >> 8;
      let win0_x2 = io.winh[0] & 0xFF;
      let win1_x1 = io.winh[1] >> 8;
      let win1_x2 = io.winh[1] & 0xFF;
      let mut result = [outside_flags; 240];
      if objwin_enabled {
        for (dst, active) in result.iter_mut().zip(obj_window_line) {
          if *active {
            *dst = objwin_flags;
          }
        }
      }
      if win1_enabled && win1_in_v {
        fill_window_range(&mut result, win1_x1, win1_x2, win1_flags);
      }
      if win0_enabled && win0_in_v {
        fill_window_range(&mut result, win0_x1, win0_x2, win0_flags);
      }
      Some(result)
    }

    fn fill_window_range(line: &mut [WindowFlags; 240], start: u16, end: u16, flags: WindowFlags) {
      let start = start as usize;
      let end = end as usize;
      if start <= end {
        line[start.min(240)..end.min(240)].fill(flags);
      } else {
        line[start.min(240)..].fill(flags);
        line[..end.min(240)].fill(flags);
      }
    }

    fn in_wrapped_range(coord: u16, start: u16, end: u16) -> bool {
      if start <= end {
        coord >= start && coord < end
      } else {
        coord >= start || coord < end
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_window_flags_from_bits() {
        let flags = WindowFlags::from_bits(0x3F);
        assert!(flags.bg_enable[0]);
        assert!(flags.bg_enable[3]);
        assert!(flags.obj_enable);
        assert!(flags.effects_enable);
        let flags = WindowFlags::from_bits(0x00);
        assert!(!flags.bg_enable[0]);
        assert!(!flags.obj_enable);
        assert!(!flags.effects_enable);
      }

      #[test]
      fn test_no_windows_returns_none() {
        let io = IoRegisters::new();
        let obj_win = [false; 240];
        assert!(compute_window_line(0, &io, &obj_win).is_none());
      }

      #[test]
      fn test_win0_basic() {
        let mut io = IoRegisters::new();
        io.dispcnt = 1 << 13;
        io.winh[0] = (10 << 8) | 50;
        io.winv[0] = 100;
        io.winin = 0x3F;
        io.winout = 0x00;
        let obj_win = [false; 240];
        let result = compute_window_line(5, &io, &obj_win).unwrap();
        assert!(result[20].bg_enable[0]);
        assert!(result[20].effects_enable);
        assert!(!result[5].bg_enable[0]);
        assert!(!result[5].effects_enable);
      }

      #[test]
      fn test_window_range_wrap() {
        assert!(in_wrapped_range(250, 200, 50));
        assert!(in_wrapped_range(10, 200, 50));
        assert!(!in_wrapped_range(100, 200, 50));
      }

      #[test]
      fn window_priority_is_win0_then_win1_then_obj_then_outside() {
        let mut io = IoRegisters::new();
        io.dispcnt = (1 << 13) | (1 << 14) | (1 << 15);
        io.winh[0] = (20 << 8) | 40;
        io.winh[1] = (30 << 8) | 50;
        io.winv[0] = 160;
        io.winv[1] = 160;
        io.winin = 0x0201;
        io.winout = 0x0804;
        let mut obj_win = [false; 240];
        obj_win[10] = true;
        obj_win[35] = true;
        obj_win[55] = true;
        let result = compute_window_line(5, &io, &obj_win).unwrap();
        assert!(result[10].bg_enable[3]);
        assert!(result[25].bg_enable[0]);
        assert!(result[35].bg_enable[0]);
        assert!(result[45].bg_enable[1]);
        assert!(result[55].bg_enable[3]);
        assert!(result[70].bg_enable[2]);
      }
    }
  }
  pub mod effects {

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BlendMode {
      None = 0,
      Alpha = 1,
      BrightnessIncrease = 2,
      BrightnessDecrease = 3,
    }

    impl BlendMode {
      pub fn from_bldcnt(bldcnt: u16) -> Self {
        match (bldcnt >> 6) & 3 {
          0 => BlendMode::None,
          1 => BlendMode::Alpha,
          2 => BlendMode::BrightnessIncrease,
          3 => BlendMode::BrightnessDecrease,
          _ => unreachable!(),
        }
      }
    }

    pub fn is_first_target(bldcnt: u16, layer: u8) -> bool {
      if layer > 5 {
        return false;
      }
      bldcnt & (1 << layer) != 0
    }

    pub fn is_second_target(bldcnt: u16, layer: u8) -> bool {
      if layer > 5 {
        return false;
      }
      bldcnt & (1 << (8 + layer)) != 0
    }

    pub fn alpha_blend(color1: u16, color2: u16, eva: u16, evb: u16) -> u16 {
      let eva = eva.min(16);
      let evb = evb.min(16);
      let r1 = (color1 & 0x1F) as u32;
      let g1 = ((color1 >> 5) & 0x1F) as u32;
      let b1 = ((color1 >> 10) & 0x1F) as u32;
      let r2 = (color2 & 0x1F) as u32;
      let g2 = ((color2 >> 5) & 0x1F) as u32;
      let b2 = ((color2 >> 10) & 0x1F) as u32;
      let r = ((r1 * eva as u32 + r2 * evb as u32) >> 4).min(31);
      let g = ((g1 * eva as u32 + g2 * evb as u32) >> 4).min(31);
      let b = ((b1 * eva as u32 + b2 * evb as u32) >> 4).min(31);
      (r as u16) | ((g as u16) << 5) | ((b as u16) << 10)
    }

    pub fn brightness_increase(color: u16, evy: u16) -> u16 {
      let evy = evy.min(16);
      let r = (color & 0x1F) as u32;
      let g = ((color >> 5) & 0x1F) as u32;
      let b = ((color >> 10) & 0x1F) as u32;
      let r = r + (((31 - r) * evy as u32) >> 4);
      let g = g + (((31 - g) * evy as u32) >> 4);
      let b = b + (((31 - b) * evy as u32) >> 4);
      (r as u16) | ((g as u16) << 5) | ((b as u16) << 10)
    }

    pub fn brightness_decrease(color: u16, evy: u16) -> u16 {
      let evy = evy.min(16);
      let r = (color & 0x1F) as u32;
      let g = ((color >> 5) & 0x1F) as u32;
      let b = ((color >> 10) & 0x1F) as u32;
      let r = r - ((r * evy as u32) >> 4);
      let g = g - ((g * evy as u32) >> 4);
      let b = b - ((b * evy as u32) >> 4);
      (r as u16) | ((g as u16) << 5) | ((b as u16) << 10)
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_alpha_blend_equal() {
        let red = 0x001F;
        let blue = 0x7C00;
        let result = alpha_blend(red, blue, 8, 8);
        let r = result & 0x1F;
        let b = (result >> 10) & 0x1F;
        assert_eq!(r, 15);
        assert_eq!(b, 15);
      }

      #[test]
      fn test_alpha_blend_full_first() {
        let red = 0x001F;
        let blue = 0x7C00;
        let result = alpha_blend(red, blue, 16, 0);
        assert_eq!(result, red);
      }

      #[test]
      fn test_alpha_blend_clamped() {
        let white = 0x7FFF;
        let result = alpha_blend(white, white, 16, 16);
        assert_eq!(result, 0x7FFF);
      }

      #[test]
      fn test_brightness_increase() {
        let black = 0x0000;
        let result = brightness_increase(black, 16);
        assert_eq!(result, 0x7FFF);
        let color = 16;
        let result = brightness_increase(color, 8);
        assert_eq!(result & 0x1F, 23);
      }

      #[test]
      fn test_brightness_decrease() {
        let white = 0x7FFF;
        let result = brightness_decrease(white, 16);
        assert_eq!(result, 0x0000);
        let color = 20;
        let result = brightness_decrease(color, 8);
        assert_eq!(result & 0x1F, 10);
      }

      #[test]
      fn test_target_flags() {
        let bldcnt: u16 = 0x0011;
        assert!(is_first_target(bldcnt, 0));
        assert!(is_first_target(bldcnt, 4));
        assert!(!is_first_target(bldcnt, 1));
        let bldcnt2: u16 = 0x0200;
        assert!(is_second_target(bldcnt2, 1));
        assert!(!is_second_target(bldcnt2, 0));
        let bldcnt3: u16 = 0x0020;
        assert!(is_first_target(bldcnt3, 5));
      }
    }
  }
  use crate::SCREEN_WIDTH;
  use crate::bus::io_regs::IoRegisters;
  use effects::{
    BlendMode, alpha_blend, brightness_decrease, brightness_increase, is_first_target,
    is_second_target,
  };
  use serde::{Deserialize, Serialize};
  use window::WindowFlags;

  #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
  pub struct PixelInfo {
    pub color: u16,
    pub priority: u8,
    pub layer: u8,
    pub semi_transparent: bool,
  }
  type PixelLine = [Option<PixelInfo>; SCREEN_WIDTH];

  #[derive(Clone, Copy)]
  struct RenderCtx<'a> {
    io: &'a IoRegisters,
    palette: &'a [u8],
    vram: &'a [u8],
    oam: &'a [u8],
  }

  #[derive(Clone, Copy)]
  struct BlendState {
    bldcnt: u16,
    mode: BlendMode,
    eva: u16,
    evb: u16,
    evy: u16,
  }

  impl BlendState {
    fn from_io(io: &IoRegisters) -> Self {
      BlendState {
        bldcnt: io.bldcnt,
        mode: BlendMode::from_bldcnt(io.bldcnt),
        eva: (io.bldalpha & 0x1F).min(16),
        evb: ((io.bldalpha >> 8) & 0x1F).min(16),
        evy: (io.bldy & 0x1F).min(16),
      }
    }
  }
  struct ScanlineLayers {
    bg: [Option<PixelLine>; 4],
    obj: PixelLine,
    backdrop: u16,
  }

  #[derive(Serialize, Deserialize)]
  pub struct Ppu {
    pub bg2_ref_x: i32,
    pub bg2_ref_y: i32,
    pub bg3_ref_x: i32,
    pub bg3_ref_y: i32,
  }

  impl Ppu {
    pub fn new() -> Self {
      Ppu {
        bg2_ref_x: 0,
        bg2_ref_y: 0,
        bg3_ref_x: 0,
        bg3_ref_y: 0,
      }
    }

    pub fn render_scanline(
      &mut self,
      line: u16,
      io: &IoRegisters,
      palette: &[u8],
      vram: &[u8],
      oam: &[u8],
      framebuffer: &mut [u16],
    ) {
      let dispcnt = io.dispcnt;
      let mode = dispcnt & 0x07;
      let forced_blank = dispcnt & (1 << 7) != 0;
      let row_start = line as usize * SCREEN_WIDTH;
      let row = &mut framebuffer[row_start..row_start + SCREEN_WIDTH];
      let ctx = RenderCtx {
        io,
        palette,
        vram,
        oam,
      };
      if forced_blank {
        row.fill(0x7FFF);
        return;
      }
      match mode {
        0 => self.render_tile_mode(line, ctx, row, &[0, 1, 2, 3], &[]),
        1 => self.render_tile_mode(line, ctx, row, &[0, 1], &[2]),
        2 => self.render_tile_mode(line, ctx, row, &[], &[2, 3]),
        3 => self.render_mode3(line, vram, row),
        4 => self.render_mode4(line, io, palette, vram, row),
        5 => self.render_mode5(line, io, vram, row),
        _ => {
          row.fill(0);
        }
      }
      if mode == 1 || mode == 2 {
        self.advance_affine_refs(io);
      }
    }

    fn render_tile_mode(
      &mut self,
      line: u16,
      ctx: RenderCtx<'_>,
      row: &mut [u16],
      text_bgs: &[usize],
      affine_bgs: &[usize],
    ) {
      let dispcnt = ctx.io.dispcnt;
      let backdrop = if ctx.palette.len() >= 2 {
        u16::from_le_bytes([ctx.palette[0], ctx.palette[1]]) & 0x7FFF
      } else {
        0
      };
      let mut layers = ScanlineLayers {
        bg: [None, None, None, None],
        obj: [None; SCREEN_WIDTH],
        backdrop,
      };
      for &bgi in text_bgs {
        if dispcnt & (1 << (8 + bgi)) != 0 {
          let mut line_buf = [None; SCREEN_WIDTH];
          bg::render_text_bg_line(bgi, line, ctx.io, ctx.palette, ctx.vram, &mut line_buf);
          layers.bg[bgi] = Some(line_buf);
        }
      }
      for &bgi in affine_bgs {
        if dispcnt & (1 << (8 + bgi)) != 0 {
          let mut line_buf = [None; SCREEN_WIDTH];
          let (ref_x, ref_y) = if bgi == 2 {
            (self.bg2_ref_x, self.bg2_ref_y)
          } else {
            (self.bg3_ref_x, self.bg3_ref_y)
          };
          bg::render_affine_bg_line(
            bgi,
            bg::BgCtx {
              io: ctx.io,
              palette: ctx.palette,
              vram: ctx.vram,
            },
            ref_x,
            ref_y,
            &mut line_buf,
          );
          layers.bg[bgi] = Some(line_buf);
        }
      }
      let obj_window_mask = [false; 240];
      let obj_enabled = dispcnt & (1 << 12) != 0;
      if obj_enabled {
        obj::render_obj_line(
          line,
          ctx.io,
          ctx.palette,
          ctx.vram,
          ctx.oam,
          &mut layers.obj,
        );
      }
      let window_line = window::compute_window_line(line, ctx.io, &obj_window_mask);
      let all_windows = [WindowFlags::all_enabled(); SCREEN_WIDTH];
      let windows = window_line.as_ref().unwrap_or(&all_windows);
      let blend = BlendState::from_io(ctx.io);
      for (x, (dst, win_flags)) in row
        .iter_mut()
        .zip(windows.iter())
        .enumerate()
        .take(SCREEN_WIDTH)
      {
        *dst = self.composite_pixel_with_effects(x, &layers, win_flags, blend);
      }
    }

    fn composite_pixel_with_effects(
      &self,
      x: usize,
      layers: &ScanlineLayers,
      win_flags: &WindowFlags,
      blend: BlendState,
    ) -> u16 {
      let mut top: Option<PixelInfo> = None;
      let mut second: Option<PixelInfo> = None;
      let mut try_insert = |px: PixelInfo| {
        let dominated = match &top {
          None => false,
          Some(t) => {
            if px.priority < t.priority {
              false
            } else if px.priority == t.priority {
              if px.layer == 4 && t.layer != 4 {
                false
              } else if px.layer != 4 && t.layer == 4 {
                true
              } else {
                px.layer >= t.layer
              }
            } else {
              true
            }
          }
        };
        if !dominated {
          second = top;
          top = Some(px);
        } else if second.is_none()
          || second.is_some_and(|s| {
            px.priority < s.priority || (px.priority == s.priority && px.layer < s.layer)
          })
        {
          second = Some(px);
        }
      };
      if win_flags.obj_enable
        && let Some(obj_px) = &layers.obj[x]
      {
        try_insert(*obj_px);
      }
      for (bgi, line_buf) in layers.bg.iter().enumerate() {
        if !win_flags.bg_enable[bgi] {
          continue;
        }
        if let Some(line_buf) = line_buf
          && let Some(px) = &line_buf[x]
        {
          try_insert(*px);
        }
      }
      let (top_color, top_layer, is_semi_transparent) = match &top {
        Some(px) => (px.color, px.layer, px.semi_transparent),
        None => (layers.backdrop, 5, false),
      };
      let (second_color, second_layer) = match &second {
        Some(px) => (px.color, px.layer),
        None => (layers.backdrop, 5),
      };
      if !win_flags.effects_enable {
        return top_color;
      }
      if is_semi_transparent && is_second_target(blend.bldcnt, second_layer) {
        return alpha_blend(top_color, second_color, blend.eva, blend.evb);
      }
      match blend.mode {
        BlendMode::None => top_color,
        BlendMode::Alpha => {
          if is_first_target(blend.bldcnt, top_layer)
            && is_second_target(blend.bldcnt, second_layer)
          {
            alpha_blend(top_color, second_color, blend.eva, blend.evb)
          } else {
            top_color
          }
        }
        BlendMode::BrightnessIncrease => {
          if is_first_target(blend.bldcnt, top_layer) {
            brightness_increase(top_color, blend.evy)
          } else {
            top_color
          }
        }
        BlendMode::BrightnessDecrease => {
          if is_first_target(blend.bldcnt, top_layer) {
            brightness_decrease(top_color, blend.evy)
          } else {
            top_color
          }
        }
      }
    }

    fn advance_affine_refs(&mut self, io: &IoRegisters) {
      let (_, pb2, _, pd2) = bg::get_affine_params(2, io);
      self.bg2_ref_x += pb2 as i32;
      self.bg2_ref_y += pd2 as i32;
      let (_, pb3, _, pd3) = bg::get_affine_params(3, io);
      self.bg3_ref_x += pb3 as i32;
      self.bg3_ref_y += pd3 as i32;
    }

    pub fn on_vblank(&mut self, io: &IoRegisters) {
      self.bg2_ref_x = io.bg2x_latch;
      self.bg2_ref_y = io.bg2y_latch;
      self.bg3_ref_x = io.bg3x_latch;
      self.bg3_ref_y = io.bg3y_latch;
    }

    pub fn reload_bg_ref(&mut self, bg: usize, coord: usize, io: &IoRegisters) {
      match (bg, coord) {
        (2, 0) => self.bg2_ref_x = io.bg2x_latch,
        (2, 1) => self.bg2_ref_y = io.bg2y_latch,
        (3, 0) => self.bg3_ref_x = io.bg3x_latch,
        (3, 1) => self.bg3_ref_y = io.bg3y_latch,
        _ => {}
      }
    }

    fn render_mode3(&self, line: u16, vram: &[u8], row: &mut [u16]) {
      let vram_row = line as usize * SCREEN_WIDTH * 2;
      for (dst, px) in row
        .iter_mut()
        .zip(vram.get(vram_row..).unwrap_or(&[]).chunks_exact(2))
        .take(SCREEN_WIDTH)
      {
        *dst = u16::from_le_bytes([px[0], px[1]]) & 0x7FFF;
      }
    }

    fn render_mode4(
      &self,
      line: u16,
      io: &IoRegisters,
      palette: &[u8],
      vram: &[u8],
      row: &mut [u16],
    ) {
      let frame_base = if io.dispcnt & (1 << 4) != 0 {
        0xA000
      } else {
        0
      };
      let vram_row = frame_base + line as usize * SCREEN_WIDTH;
      for (dst, &idx) in row
        .iter_mut()
        .zip(vram.get(vram_row..).unwrap_or(&[]).iter())
        .take(SCREEN_WIDTH)
      {
        let idx = idx as usize;
        let po = idx * 2;
        let color = u16::from_le_bytes([palette[po], palette[po + 1]]);
        *dst = color & 0x7FFF;
      }
    }

    fn render_mode5(&self, line: u16, io: &IoRegisters, vram: &[u8], row: &mut [u16]) {
      let frame_base = if io.dispcnt & (1 << 4) != 0 {
        0xA000
      } else {
        0
      };
      if line >= 128 {
        row.fill(0);
        return;
      }
      let vram_row = frame_base + line as usize * 160 * 2;
      let (visible, blank) = row.split_at_mut(160);
      for (dst, px) in visible
        .iter_mut()
        .zip(vram.get(vram_row..).unwrap_or(&[]).chunks_exact(2))
      {
        *dst = u16::from_le_bytes([px[0], px[1]]) & 0x7FFF;
      }
      blank.fill(0);
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    fn layers(bg: [Option<PixelLine>; 4], obj: PixelLine, backdrop: u16) -> ScanlineLayers {
      ScanlineLayers { bg, obj, backdrop }
    }

    fn blend(bldcnt: u16, mode: BlendMode, eva: u16, evb: u16, evy: u16) -> BlendState {
      BlendState {
        bldcnt,
        mode,
        eva,
        evb,
        evy,
      }
    }

    #[test]
    fn test_composite_backdrop_only() {
      let ppu = Ppu::new();
      let bg_lines: [Option<[Option<PixelInfo>; 240]>; 4] = [None, None, None, None];
      let obj_line = [None; 240];
      let win = WindowFlags::all_enabled();
      let layers = layers(bg_lines, obj_line, 0x7C00);
      let color =
        ppu.composite_pixel_with_effects(0, &layers, &win, blend(0, BlendMode::None, 0, 0, 0));
      assert_eq!(color, 0x7C00);
    }

    #[test]
    fn test_composite_bg_over_backdrop() {
      let ppu = Ppu::new();
      let mut bg0 = [None; 240];
      bg0[0] = Some(PixelInfo {
        color: 0x001F,
        priority: 0,
        layer: 0,
        semi_transparent: false,
      });
      let bg_lines = [Some(bg0), None, None, None];
      let obj_line = [None; 240];
      let win = WindowFlags::all_enabled();
      let layers = layers(bg_lines, obj_line, 0x7C00);
      let color =
        ppu.composite_pixel_with_effects(0, &layers, &win, blend(0, BlendMode::None, 0, 0, 0));
      assert_eq!(color, 0x001F);
    }

    #[test]
    fn test_composite_obj_over_bg_same_priority() {
      let ppu = Ppu::new();
      let mut bg0 = [None; 240];
      bg0[0] = Some(PixelInfo {
        color: 0x001F,
        priority: 0,
        layer: 0,
        semi_transparent: false,
      });
      let bg_lines = [Some(bg0), None, None, None];
      let mut obj_line = [None; 240];
      obj_line[0] = Some(PixelInfo {
        color: 0x03E0,
        priority: 0,
        layer: 4,
        semi_transparent: false,
      });
      let win = WindowFlags::all_enabled();
      let layers = layers(bg_lines, obj_line, 0);
      let color =
        ppu.composite_pixel_with_effects(0, &layers, &win, blend(0, BlendMode::None, 0, 0, 0));
      assert_eq!(color, 0x03E0);
    }

    #[test]
    fn test_alpha_blend_bg_layers() {
      let ppu = Ppu::new();
      let mut bg0 = [None; 240];
      bg0[0] = Some(PixelInfo {
        color: 0x001F,
        priority: 0,
        layer: 0,
        semi_transparent: false,
      });
      let mut bg1 = [None; 240];
      bg1[0] = Some(PixelInfo {
        color: 0x7C00,
        priority: 1,
        layer: 1,
        semi_transparent: false,
      });
      let bg_lines = [Some(bg0), Some(bg1), None, None];
      let obj_line = [None; 240];
      let win = WindowFlags::all_enabled();
      let bldcnt: u16 = (1 << 6) | (1 << 0) | (1 << 9);
      let layers = layers(bg_lines, obj_line, 0);
      let color = ppu.composite_pixel_with_effects(
        0,
        &layers,
        &win,
        blend(bldcnt, BlendMode::Alpha, 8, 8, 0),
      );
      let r = color & 0x1F;
      let b = (color >> 10) & 0x1F;
      assert_eq!(r, 15);
      assert_eq!(b, 15);
    }

    #[test]
    fn test_brightness_increase_on_first_target() {
      let ppu = Ppu::new();
      let mut bg0 = [None; 240];
      bg0[0] = Some(PixelInfo {
        color: 0x0000,
        priority: 0,
        layer: 0,
        semi_transparent: false,
      });
      let bg_lines = [Some(bg0), None, None, None];
      let obj_line = [None; 240];
      let win = WindowFlags::all_enabled();
      let bldcnt: u16 = (2 << 6) | (1 << 0);
      let layers = layers(bg_lines, obj_line, 0);
      let color = ppu.composite_pixel_with_effects(
        0,
        &layers,
        &win,
        blend(bldcnt, BlendMode::BrightnessIncrease, 0, 0, 16),
      );
      assert_eq!(color, 0x7FFF);
    }

    #[test]
    fn test_window_hides_layer() {
      let ppu = Ppu::new();
      let mut bg0 = [None; 240];
      bg0[0] = Some(PixelInfo {
        color: 0x001F,
        priority: 0,
        layer: 0,
        semi_transparent: false,
      });
      let bg_lines = [Some(bg0), None, None, None];
      let obj_line = [None; 240];
      let win = WindowFlags {
        bg_enable: [false, true, true, true],
        obj_enable: true,
        effects_enable: true,
      };
      let layers = layers(bg_lines, obj_line, 0x7C00);
      let color =
        ppu.composite_pixel_with_effects(0, &layers, &win, blend(0, BlendMode::None, 0, 0, 0));
      assert_eq!(color, 0x7C00);
    }

    #[test]
    fn test_semi_transparent_obj_always_blends() {
      let ppu = Ppu::new();
      let mut bg0 = [None; 240];
      bg0[0] = Some(PixelInfo {
        color: 0x7C00,
        priority: 1,
        layer: 0,
        semi_transparent: false,
      });
      let bg_lines = [Some(bg0), None, None, None];
      let mut obj_line = [None; 240];
      obj_line[0] = Some(PixelInfo {
        color: 0x001F,
        priority: 0,
        layer: 4,
        semi_transparent: true,
      });
      let win = WindowFlags::all_enabled();
      let bldcnt: u16 = 1 << 8;
      let layers = layers(bg_lines, obj_line, 0);
      let color = ppu.composite_pixel_with_effects(
        0,
        &layers,
        &win,
        blend(bldcnt, BlendMode::Alpha, 8, 8, 0),
      );
      let r = color & 0x1F;
      let b = (color >> 10) & 0x1F;
      assert_eq!(r, 15);
      assert_eq!(b, 15);
    }

    #[test]
    fn mode4_palette_index_zero_renders_palette_zero_color() {
      let ppu = Ppu::new();
      let io = IoRegisters::new();
      let mut palette = vec![0u8; 0x400];
      palette[0..2].copy_from_slice(&0x001Fu16.to_le_bytes());
      palette[2..4].copy_from_slice(&0x03E0u16.to_le_bytes());
      let mut vram = vec![0u8; 0x18000];
      vram[0] = 0;
      vram[1] = 1;
      let mut row = [0u16; SCREEN_WIDTH];
      ppu.render_mode4(0, &io, &palette, &vram, &mut row);
      assert_eq!(row[0], 0x001F);
      assert_eq!(row[1], 0x03E0);
    }
  }
}

pub mod apu {
  pub mod psg {
    use serde::{Deserialize, Serialize};
    const DUTY_TABLE: [[u8; 8]; 4] = [
      [0, 0, 0, 0, 0, 0, 0, 1],
      [1, 0, 0, 0, 0, 0, 0, 1],
      [1, 0, 0, 0, 0, 1, 1, 1],
      [0, 1, 1, 1, 1, 1, 1, 0],
    ];

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel1 {
      pub enabled: bool,
      pub sweep_period: u8,
      pub sweep_negate: bool,
      pub sweep_shift: u8,
      sweep_timer: u8,
      sweep_shadow: u16,
      sweep_enabled: bool,
      pub duty: u8,
      pub length_load: u8,
      length_counter: u16,
      pub length_enabled: bool,
      pub envelope_init: u8,
      pub envelope_dir: bool,
      pub envelope_period: u8,
      envelope_volume: u8,
      envelope_timer: u8,
      pub frequency: u16,
      timer: u32,
      duty_pos: u8,
    }

    impl Channel1 {
      pub fn new() -> Self {
        Channel1 {
          enabled: false,
          sweep_period: 0,
          sweep_negate: false,
          sweep_shift: 0,
          sweep_timer: 0,
          sweep_shadow: 0,
          sweep_enabled: false,
          duty: 0,
          length_load: 0,
          length_counter: 0,
          length_enabled: false,
          envelope_init: 0,
          envelope_dir: false,
          envelope_period: 0,
          envelope_volume: 0,
          envelope_timer: 0,
          frequency: 0,
          timer: 0,
          duty_pos: 0,
        }
      }

      pub fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
          self.length_counter = 64;
        }
        self.timer = (2048 - self.frequency as u32) * 16;
        self.envelope_volume = self.envelope_init;
        self.envelope_timer = self.envelope_period;
        self.sweep_shadow = self.frequency;
        self.sweep_timer = if self.sweep_period > 0 {
          self.sweep_period
        } else {
          8
        };
        self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;
        if self.sweep_shift > 0 {
          let _ = self.calc_sweep_freq();
        }
        if self.envelope_init == 0 && !self.envelope_dir {
          self.enabled = false;
        }
      }

      fn calc_sweep_freq(&mut self) -> u16 {
        let delta = self.sweep_shadow >> self.sweep_shift;
        let new_freq = if self.sweep_negate {
          self.sweep_shadow.wrapping_sub(delta)
        } else {
          self.sweep_shadow + delta
        };
        if new_freq > 2047 {
          self.enabled = false;
        }
        new_freq
      }

      pub fn clock_sweep(&mut self) {
        if self.sweep_timer > 0 {
          self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
          self.sweep_timer = if self.sweep_period > 0 {
            self.sweep_period
          } else {
            8
          };
          if self.sweep_enabled && self.sweep_period > 0 {
            let new_freq = self.calc_sweep_freq();
            if new_freq <= 2047 && self.sweep_shift > 0 {
              self.sweep_shadow = new_freq;
              self.frequency = new_freq;
              let _ = self.calc_sweep_freq();
            }
          }
        }
      }

      pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
          self.length_counter -= 1;
          if self.length_counter == 0 {
            self.enabled = false;
          }
        }
      }

      pub fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
          return;
        }
        if self.envelope_timer > 0 {
          self.envelope_timer -= 1;
        }
        if self.envelope_timer == 0 {
          self.envelope_timer = self.envelope_period;
          if self.envelope_dir && self.envelope_volume < 15 {
            self.envelope_volume += 1;
          } else if !self.envelope_dir && self.envelope_volume > 0 {
            self.envelope_volume -= 1;
          }
        }
      }

      pub fn tick(&mut self) -> i16 {
        if !self.enabled {
          return 0;
        }
        if self.timer > 0 {
          self.timer -= 1;
        }
        if self.timer == 0 {
          self.timer = (2048 - self.frequency as u32) * 16;
          self.duty_pos = (self.duty_pos + 1) & 7;
        }
        self.output()
      }

      pub fn output(&self) -> i16 {
        if !self.enabled {
          return 0;
        }
        let wave = DUTY_TABLE[self.duty as usize & 3][self.duty_pos as usize];
        if wave != 0 {
          self.envelope_volume as i16
        } else {
          -(self.envelope_volume as i16)
        }
      }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel2 {
      pub enabled: bool,
      pub duty: u8,
      pub length_load: u8,
      length_counter: u16,
      pub length_enabled: bool,
      pub envelope_init: u8,
      pub envelope_dir: bool,
      pub envelope_period: u8,
      envelope_volume: u8,
      envelope_timer: u8,
      pub frequency: u16,
      timer: u32,
      duty_pos: u8,
    }

    impl Channel2 {
      pub fn new() -> Self {
        Channel2 {
          enabled: false,
          duty: 0,
          length_load: 0,
          length_counter: 0,
          length_enabled: false,
          envelope_init: 0,
          envelope_dir: false,
          envelope_period: 0,
          envelope_volume: 0,
          envelope_timer: 0,
          frequency: 0,
          timer: 0,
          duty_pos: 0,
        }
      }

      pub fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
          self.length_counter = 64;
        }
        self.timer = (2048 - self.frequency as u32) * 16;
        self.envelope_volume = self.envelope_init;
        self.envelope_timer = self.envelope_period;
        if self.envelope_init == 0 && !self.envelope_dir {
          self.enabled = false;
        }
      }

      pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
          self.length_counter -= 1;
          if self.length_counter == 0 {
            self.enabled = false;
          }
        }
      }

      pub fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
          return;
        }
        if self.envelope_timer > 0 {
          self.envelope_timer -= 1;
        }
        if self.envelope_timer == 0 {
          self.envelope_timer = self.envelope_period;
          if self.envelope_dir && self.envelope_volume < 15 {
            self.envelope_volume += 1;
          } else if !self.envelope_dir && self.envelope_volume > 0 {
            self.envelope_volume -= 1;
          }
        }
      }

      pub fn tick(&mut self) -> i16 {
        if !self.enabled {
          return 0;
        }
        if self.timer > 0 {
          self.timer -= 1;
        }
        if self.timer == 0 {
          self.timer = (2048 - self.frequency as u32) * 16;
          self.duty_pos = (self.duty_pos + 1) & 7;
        }
        self.output()
      }

      pub fn output(&self) -> i16 {
        if !self.enabled {
          return 0;
        }
        let wave = DUTY_TABLE[self.duty as usize & 3][self.duty_pos as usize];
        if wave != 0 {
          self.envelope_volume as i16
        } else {
          -(self.envelope_volume as i16)
        }
      }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel3 {
      pub enabled: bool,
      pub dac_enabled: bool,
      pub length_load: u16,
      length_counter: u16,
      pub length_enabled: bool,
      pub volume_code: u8,
      pub force_75: bool,
      pub frequency: u16,
      pub wave_ram: [u8; 32],
      pub bank_select: u8,
      pub dimension: bool,
      timer: u32,
      sample_pos: u8,
    }

    impl Channel3 {
      pub fn new() -> Self {
        Channel3 {
          enabled: false,
          dac_enabled: false,
          length_load: 0,
          length_counter: 0,
          length_enabled: false,
          volume_code: 0,
          force_75: false,
          frequency: 0,
          wave_ram: [0; 32],
          bank_select: 0,
          dimension: false,
          timer: 0,
          sample_pos: 0,
        }
      }

      pub fn trigger(&mut self) {
        self.enabled = self.dac_enabled;
        if self.length_counter == 0 {
          self.length_counter = 256;
        }
        self.timer = (2048 - self.frequency as u32) * 8;
        self.sample_pos = 0;
      }

      pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
          self.length_counter -= 1;
          if self.length_counter == 0 {
            self.enabled = false;
          }
        }
      }

      pub fn tick(&mut self) -> i16 {
        if !self.enabled || !self.dac_enabled {
          return 0;
        }
        if self.timer > 0 {
          self.timer -= 1;
        }
        if self.timer == 0 {
          self.timer = (2048 - self.frequency as u32) * 8;
          let total_samples = if self.dimension { 64 } else { 32 };
          self.sample_pos = (self.sample_pos + 1) & (total_samples - 1);
        }
        self.output()
      }

      pub fn output(&self) -> i16 {
        if !self.enabled || !self.dac_enabled {
          return 0;
        }
        let pos = if !self.dimension {
          let bank_offset = if self.bank_select == 0 { 16 } else { 0 };
          bank_offset + self.sample_pos as usize
        } else {
          self.sample_pos as usize
        };
        let byte_idx = pos >> 1;
        let sample = if pos & 1 == 0 {
          (self.wave_ram[byte_idx] >> 4) & 0xF
        } else {
          self.wave_ram[byte_idx] & 0xF
        };
        let shifted = match self.volume_code {
          0 => 0,
          1 => sample,
          2 => sample >> 1,
          3 => sample >> 2,
          _ => sample,
        };
        let shifted = if self.force_75 {
          (sample * 3) >> 2
        } else {
          shifted
        };
        shifted as i16 - 8
      }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel4 {
      pub enabled: bool,
      pub length_load: u8,
      length_counter: u16,
      pub length_enabled: bool,
      pub envelope_init: u8,
      pub envelope_dir: bool,
      pub envelope_period: u8,
      envelope_volume: u8,
      envelope_timer: u8,
      pub clock_shift: u8,
      pub width_mode: bool,
      pub divisor_code: u8,
      timer: u32,
      lfsr: u16,
    }

    impl Channel4 {
      pub fn new() -> Self {
        Channel4 {
          enabled: false,
          length_load: 0,
          length_counter: 0,
          length_enabled: false,
          envelope_init: 0,
          envelope_dir: false,
          envelope_period: 0,
          envelope_volume: 0,
          envelope_timer: 0,
          clock_shift: 0,
          width_mode: false,
          divisor_code: 0,
          timer: 0,
          lfsr: 0x7FFF,
        }
      }

      fn divisor(&self) -> u32 {
        match self.divisor_code & 7 {
          0 => 8,
          n => (n as u32) * 16,
        }
      }

      pub fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
          self.length_counter = 64;
        }
        self.timer = (self.divisor() << self.clock_shift) * 4;
        self.envelope_volume = self.envelope_init;
        self.envelope_timer = self.envelope_period;
        self.lfsr = 0x7FFF;
        if self.envelope_init == 0 && !self.envelope_dir {
          self.enabled = false;
        }
      }

      pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
          self.length_counter -= 1;
          if self.length_counter == 0 {
            self.enabled = false;
          }
        }
      }

      pub fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
          return;
        }
        if self.envelope_timer > 0 {
          self.envelope_timer -= 1;
        }
        if self.envelope_timer == 0 {
          self.envelope_timer = self.envelope_period;
          if self.envelope_dir && self.envelope_volume < 15 {
            self.envelope_volume += 1;
          } else if !self.envelope_dir && self.envelope_volume > 0 {
            self.envelope_volume -= 1;
          }
        }
      }

      pub fn tick(&mut self) -> i16 {
        if !self.enabled {
          return 0;
        }
        if self.timer > 0 {
          self.timer -= 1;
        }
        if self.timer == 0 {
          self.timer = (self.divisor() << self.clock_shift) * 4;
          let xor_bit = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
          self.lfsr >>= 1;
          self.lfsr |= xor_bit << 14;
          if self.width_mode {
            self.lfsr = (self.lfsr & !(1 << 6)) | (xor_bit << 6);
          }
        }
        self.output()
      }

      pub fn output(&self) -> i16 {
        if !self.enabled {
          return 0;
        }
        if self.lfsr & 1 == 0 {
          self.envelope_volume as i16
        } else {
          -(self.envelope_volume as i16)
        }
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_ch1_trigger_and_tick() {
        let mut ch = Channel1::new();
        ch.duty = 2;
        ch.frequency = 2000;
        ch.envelope_init = 15;
        ch.envelope_dir = false;
        ch.envelope_period = 0;
        ch.trigger();
        assert!(ch.enabled);
        let mut found_nonzero = false;
        for _ in 0..1000 {
          let s = ch.tick();
          if s != 0 {
            found_nonzero = true;
            break;
          }
        }
        assert!(found_nonzero);
      }

      #[test]
      fn test_ch4_noise() {
        let mut ch = Channel4::new();
        ch.envelope_init = 15;
        ch.clock_shift = 0;
        ch.divisor_code = 1;
        ch.trigger();
        assert!(ch.enabled);
        let mut samples = std::collections::HashSet::new();
        for _ in 0..4000 {
          samples.insert(ch.tick());
        }
        assert!(samples.len() > 1, "Noise should produce varied output");
      }

      #[test]
      fn ch1_full_waveform_period_matches_gba_spec() {
        let mut ch = Channel1::new();
        ch.duty = 2;
        ch.frequency = 1024;
        ch.envelope_init = 15;
        ch.envelope_dir = false;
        ch.envelope_period = 0;
        ch.length_enabled = false;
        ch.trigger();
        let mut last = ch.tick();
        let mut first_h_to_l = 0u64;
        for c in 2u64..400_000 {
          let v = ch.tick();
          if v < last && last > 0 {
            first_h_to_l = c;
            last = v;
            break;
          }
          last = v;
        }
        assert!(first_h_to_l != 0, "never saw first H→L transition");
        let mut second_h_to_l = 0u64;
        for c in first_h_to_l + 1..1_000_000 {
          let v = ch.tick();
          if v < last && last > 0 {
            second_h_to_l = c;
            break;
          }
          last = v;
        }
        let waveform_cycles = second_h_to_l - first_h_to_l;
        let expected = (2048u64 - 1024) * 16 * 8;
        assert_eq!(
          waveform_cycles, expected,
          "ch1 full-waveform period: got {} CPU cycles, spec wants {} for F=1024",
          waveform_cycles, expected,
        );
      }

      #[test]
      fn test_length_counter_disables() {
        let mut ch = Channel2::new();
        ch.duty = 2;
        ch.frequency = 2000;
        ch.envelope_init = 15;
        ch.length_enabled = true;
        ch.trigger();
        ch.length_counter = 2;
        ch.clock_length();
        assert!(ch.enabled);
        ch.clock_length();
        assert!(!ch.enabled);
      }
    }
  }
  pub mod fifo {
    use serde::{Deserialize, Serialize};
    const FIFO_CAPACITY: usize = 32;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FifoChannel {
      buffer: [i8; FIFO_CAPACITY],
      read_pos: usize,
      write_pos: usize,
      pub count: usize,
      pub current_sample: i8,
      pub timer_select: u8,
      pub enable_left: bool,
      pub enable_right: bool,
      pub volume_full: bool,
    }

    impl FifoChannel {
      pub fn new() -> Self {
        FifoChannel {
          buffer: [0; FIFO_CAPACITY],
          read_pos: 0,
          write_pos: 0,
          count: 0,
          current_sample: 0,
          timer_select: 0,
          enable_left: false,
          enable_right: false,
          volume_full: false,
        }
      }

      pub fn reset(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
        self.count = 0;
        self.current_sample = 0;
      }

      pub fn write32(&mut self, value: u32) {
        let bytes = value.to_le_bytes();
        for &byte in &bytes {
          self.push_byte(byte as i8);
        }
      }

      pub fn write16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.push_byte(bytes[0] as i8);
        self.push_byte(bytes[1] as i8);
      }

      fn push_byte(&mut self, sample: i8) {
        if self.count < FIFO_CAPACITY {
          self.buffer[self.write_pos] = sample;
          self.write_pos = (self.write_pos + 1) % FIFO_CAPACITY;
          self.count += 1;
        }
      }

      pub fn pop_sample(&mut self) -> bool {
        if self.count > 0 {
          self.current_sample = self.buffer[self.read_pos];
          self.read_pos = (self.read_pos + 1) % FIFO_CAPACITY;
          self.count -= 1;
        }
        self.count <= 16
      }

      pub fn output(&self) -> i16 {
        let sample = self.current_sample as i16;
        if self.volume_full { sample } else { sample / 2 }
      }

      pub fn len(&self) -> usize {
        self.count
      }

      pub fn is_empty(&self) -> bool {
        self.count == 0
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_fifo_write_and_pop() {
        let mut fifo = FifoChannel::new();
        fifo.volume_full = true;
        fifo.write32(0x01020304);
        assert_eq!(fifo.len(), 4);
        fifo.pop_sample();
        assert_eq!(fifo.current_sample, 0x04);
        fifo.pop_sample();
        assert_eq!(fifo.current_sample, 0x03);
        fifo.pop_sample();
        assert_eq!(fifo.current_sample, 0x02);
        fifo.pop_sample();
        assert_eq!(fifo.current_sample, 0x01);
        assert_eq!(fifo.len(), 0);
      }

      #[test]
      fn test_fifo_reset() {
        let mut fifo = FifoChannel::new();
        fifo.write32(0xDEADBEEF);
        assert_eq!(fifo.len(), 4);
        fifo.reset();
        assert_eq!(fifo.len(), 0);
      }

      #[test]
      fn test_fifo_refill_request() {
        let mut fifo = FifoChannel::new();
        for i in 0..8 {
          fifo.write32(i);
        }
        assert_eq!(fifo.len(), 32);
        for _ in 0..15 {
          let needs_refill = fifo.pop_sample();
          assert!(!needs_refill);
        }
        let needs_refill = fifo.pop_sample();
        assert!(needs_refill);
      }
    }
  }
  use fifo::FifoChannel;
  use psg::{Channel1, Channel2, Channel3, Channel4};
  use serde::{Deserialize, Serialize};
  pub const OUTPUT_SAMPLE_RATE: u32 = 48_000;
  pub const CPU_CLOCK_HZ: u32 = 16_777_216;
  pub const CYCLES_PER_FRAME_SEQ: u32 = 32_768;

  #[derive(Serialize, Deserialize)]
  pub struct Apu {
    pub ch1: Channel1,
    pub ch2: Channel2,
    pub ch3: Channel3,
    pub ch4: Channel4,
    pub fifo_a: FifoChannel,
    pub fifo_b: FifoChannel,
    pub master_enable: bool,
    pub psg_volume_right: u8,
    pub psg_volume_left: u8,
    pub psg_enable_right: [bool; 4],
    pub psg_enable_left: [bool; 4],
    pub psg_master_volume: u8,
    pub bias_level: u16,
    frame_seq_step: u8,
    frame_seq_counter: u32,
    sample_frac: u64,
    accum_left: i64,
    accum_right: i64,
    accum_count: u32,
    lpf_left: [i32; 1],
    lpf_right: [i32; 1],
    #[serde(default)]
    hp_in_l: i64,
    #[serde(default)]
    hp_out_l: i64,
    #[serde(default)]
    hp_in_r: i64,
    #[serde(default)]
    hp_out_r: i64,
    #[serde(default)]
    emph_prev_l: i64,
    #[serde(default)]
    emph_prev_r: i64,
    pub sample_buffer: Vec<i16>,
    pub sample_buffer_max: usize,
  }

  impl Apu {
    pub fn new() -> Self {
      Apu {
        ch1: Channel1::new(),
        ch2: Channel2::new(),
        ch3: Channel3::new(),
        ch4: Channel4::new(),
        fifo_a: FifoChannel::new(),
        fifo_b: FifoChannel::new(),
        master_enable: false,
        psg_volume_right: 7,
        psg_volume_left: 7,
        psg_enable_right: [false; 4],
        psg_enable_left: [false; 4],
        psg_master_volume: 2,
        bias_level: 0x200,
        frame_seq_step: 0,
        frame_seq_counter: 0,
        sample_frac: 0,
        accum_left: 0,
        accum_right: 0,
        accum_count: 0,
        lpf_left: [0; 1],
        lpf_right: [0; 1],
        hp_in_l: 0,
        hp_out_l: 0,
        hp_in_r: 0,
        hp_out_r: 0,
        emph_prev_l: 0,
        emph_prev_r: 0,
        sample_buffer: Vec::with_capacity(8192),
        sample_buffer_max: 16384,
      }
    }

    pub fn tick(&mut self, cycles: u32) -> (bool, bool) {
      let psg_active = self.ch1.enabled || self.ch2.enabled || self.ch3.enabled || self.ch4.enabled;
      if !psg_active {
        self.tick_fast(cycles);
        return (false, false);
      }
      for _ in 0..cycles {
        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();
        self.ch4.tick();
        self.frame_seq_counter += 1;
        if self.frame_seq_counter >= CYCLES_PER_FRAME_SEQ {
          self.frame_seq_counter -= CYCLES_PER_FRAME_SEQ;
          self.clock_frame_sequencer();
        }
        if self.master_enable {
          let (l, r) = self.current_mix();
          self.accum_left += l as i64;
          self.accum_right += r as i64;
        }
        self.accum_count += 1;
        self.sample_frac += OUTPUT_SAMPLE_RATE as u64;
        if self.sample_frac >= CPU_CLOCK_HZ as u64 {
          self.sample_frac -= CPU_CLOCK_HZ as u64;
          self.emit_sample();
        }
      }
      (false, false)
    }

    fn tick_fast(&mut self, cycles: u32) {
      let (l, r) = if self.master_enable {
        self.current_mix()
      } else {
        (0, 0)
      };
      let l = l as i64;
      let r = r as i64;
      let mut remaining = cycles as u64;
      while remaining > 0 {
        let cpu_clock = CPU_CLOCK_HZ as u64;
        let out_rate = OUTPUT_SAMPLE_RATE as u64;
        let to_next_emit = (cpu_clock - self.sample_frac).div_ceil(out_rate);
        let chunk = remaining.min(to_next_emit);
        self.accum_left += l * chunk as i64;
        self.accum_right += r * chunk as i64;
        self.accum_count += chunk as u32;
        self.sample_frac += out_rate * chunk;
        if self.sample_frac >= cpu_clock {
          self.sample_frac -= cpu_clock;
          self.emit_sample();
        }
        remaining -= chunk;
      }
      self.frame_seq_counter += cycles;
      while self.frame_seq_counter >= CYCLES_PER_FRAME_SEQ {
        self.frame_seq_counter -= CYCLES_PER_FRAME_SEQ;
        self.clock_frame_sequencer();
      }
    }

    fn current_mix(&self) -> (i32, i32) {
      let ch1 = self.ch1.output() as i32;
      let ch2 = self.ch2.output() as i32;
      let ch3 = self.ch3.output() as i32;
      let ch4 = self.ch4.output() as i32;
      let l = self.psg_enable_left.map(i32::from);
      let r = self.psg_enable_right.map(i32::from);
      let mut psg_left = ch1 * l[0] + ch2 * l[1] + ch3 * l[2] + ch4 * l[3];
      let mut psg_right = ch1 * r[0] + ch2 * r[1] + ch3 * r[2] + ch4 * r[3];
      psg_left = psg_left * (self.psg_volume_left as i32 + 1) / 8;
      psg_right = psg_right * (self.psg_volume_right as i32 + 1) / 8;
      let psg_ratio = match self.psg_master_volume {
        0 => 1,
        1 => 2,
        _ => 4,
      };
      psg_left = psg_left * psg_ratio / 4;
      psg_right = psg_right * psg_ratio / 4;
      let fifo_a = self.fifo_a.output() as i32;
      let fifo_b = self.fifo_b.output() as i32;
      let left = psg_left
        + fifo_a * i32::from(self.fifo_a.enable_left)
        + fifo_b * i32::from(self.fifo_b.enable_left);
      let right = psg_right
        + fifo_a * i32::from(self.fifo_a.enable_right)
        + fifo_b * i32::from(self.fifo_b.enable_right);
      (left, right)
    }

    fn emit_sample(&mut self) {
      if self.accum_count == 0 {
        self.push_pair(0, 0);
        return;
      }
      let n = self.accum_count as i64;
      let avg_left = (self.accum_left / n) as i32;
      let avg_right = (self.accum_right / n) as i32;
      self.accum_left = 0;
      self.accum_right = 0;
      self.accum_count = 0;
      let scaled_left = avg_left * 200;
      let scaled_right = avg_right * 200;
      let (hp_left, hp_right) = self.dc_block(scaled_left as i64, scaled_right as i64);
      let (em_left, em_right) = self.hf_emphasis(hp_left, hp_right);
      let left_out = em_left.clamp(-32768, 32767) as i16;
      let right_out = em_right.clamp(-32768, 32767) as i16;
      self.push_pair(left_out, right_out);
    }

    fn dc_block(&mut self, xl: i64, xr: i64) -> (i64, i64) {
      const R_NUM: i64 = 8187;
      const R_DEN: i64 = 8192;
      let yl = xl - self.hp_in_l + self.hp_out_l * R_NUM / R_DEN;
      let yr = xr - self.hp_in_r + self.hp_out_r * R_NUM / R_DEN;
      self.hp_in_l = xl;
      self.hp_out_l = yl;
      self.hp_in_r = xr;
      self.hp_out_r = yr;
      (yl, yr)
    }

    fn hf_emphasis(&mut self, xl: i64, xr: i64) -> (i64, i64) {
      const K_NUM: i64 = 512;
      const K_DEN: i64 = 1024;
      let yl = xl + K_NUM * (xl - self.emph_prev_l) / K_DEN;
      let yr = xr + K_NUM * (xr - self.emph_prev_r) / K_DEN;
      self.emph_prev_l = xl;
      self.emph_prev_r = xr;
      (yl, yr)
    }

    fn push_pair(&mut self, left: i16, right: i16) {
      if self.sample_buffer.len() < self.sample_buffer_max {
        self.sample_buffer.push(left);
        self.sample_buffer.push(right);
      }
    }

    pub fn on_timer_overflow(&mut self, timer_id: u8) -> (bool, bool) {
      let mut fifo_a_refill = false;
      let mut fifo_b_refill = false;
      if self.fifo_a.timer_select == timer_id {
        fifo_a_refill = self.fifo_a.pop_sample();
      }
      if self.fifo_b.timer_select == timer_id {
        fifo_b_refill = self.fifo_b.pop_sample();
      }
      (fifo_a_refill, fifo_b_refill)
    }

    fn clock_frame_sequencer(&mut self) {
      match self.frame_seq_step {
        0 => {
          self.ch1.clock_length();
          self.ch2.clock_length();
          self.ch3.clock_length();
          self.ch4.clock_length();
        }
        2 => {
          self.ch1.clock_length();
          self.ch2.clock_length();
          self.ch3.clock_length();
          self.ch4.clock_length();
          self.ch1.clock_sweep();
        }
        4 => {
          self.ch1.clock_length();
          self.ch2.clock_length();
          self.ch3.clock_length();
          self.ch4.clock_length();
        }
        6 => {
          self.ch1.clock_length();
          self.ch2.clock_length();
          self.ch3.clock_length();
          self.ch4.clock_length();
          self.ch1.clock_sweep();
        }
        7 => {
          self.ch1.clock_envelope();
          self.ch2.clock_envelope();
          self.ch4.clock_envelope();
        }
        _ => {}
      }
      self.frame_seq_step = (self.frame_seq_step + 1) & 7;
    }

    pub fn drain_samples(&mut self, out: &mut [i16]) -> usize {
      let available = self.sample_buffer.len().min(out.len());
      out[..available].copy_from_slice(&self.sample_buffer[..available]);
      let remaining = self.sample_buffer.len() - available;
      if remaining == 0 {
        self.sample_buffer.clear();
      } else if available > 0 {
        self.sample_buffer.copy_within(available.., 0);
        self.sample_buffer.truncate(remaining);
      }
      available
    }

    pub fn write_reg(&mut self, offset: u16, value: u16) {
      match offset {
        0x00 => {
          self.ch1.sweep_shift = (value & 7) as u8;
          self.ch1.sweep_negate = value & (1 << 3) != 0;
          self.ch1.sweep_period = ((value >> 4) & 7) as u8;
        }
        0x02 => {
          self.ch1.length_load = (value & 0x3F) as u8;
          self.ch1.duty = ((value >> 6) & 3) as u8;
          self.ch1.envelope_period = (value >> 8 & 7) as u8;
          self.ch1.envelope_dir = value & (1 << 11) != 0;
          self.ch1.envelope_init = ((value >> 12) & 0xF) as u8;
        }
        0x04 => {
          self.ch1.frequency = value & 0x7FF;
          self.ch1.length_enabled = value & (1 << 14) != 0;
          if value & (1 << 15) != 0 {
            self.ch1.trigger();
          }
        }
        0x08 => {
          self.ch2.length_load = (value & 0x3F) as u8;
          self.ch2.duty = ((value >> 6) & 3) as u8;
          self.ch2.envelope_period = (value >> 8 & 7) as u8;
          self.ch2.envelope_dir = value & (1 << 11) != 0;
          self.ch2.envelope_init = ((value >> 12) & 0xF) as u8;
        }
        0x0C => {
          self.ch2.frequency = value & 0x7FF;
          self.ch2.length_enabled = value & (1 << 14) != 0;
          if value & (1 << 15) != 0 {
            self.ch2.trigger();
          }
        }
        0x10 => {
          self.ch3.dimension = value & (1 << 5) != 0;
          self.ch3.bank_select = ((value >> 6) & 1) as u8;
          self.ch3.dac_enabled = value & (1 << 7) != 0;
        }
        0x12 => {
          self.ch3.length_load = value & 0xFF;
          self.ch3.volume_code = ((value >> 13) & 3) as u8;
          self.ch3.force_75 = value & (1 << 15) != 0;
        }
        0x14 => {
          self.ch3.frequency = value & 0x7FF;
          self.ch3.length_enabled = value & (1 << 14) != 0;
          if value & (1 << 15) != 0 {
            self.ch3.trigger();
          }
        }
        0x18 => {
          self.ch4.length_load = (value & 0x3F) as u8;
          self.ch4.envelope_period = (value >> 8 & 7) as u8;
          self.ch4.envelope_dir = value & (1 << 11) != 0;
          self.ch4.envelope_init = ((value >> 12) & 0xF) as u8;
        }
        0x1C => {
          self.ch4.divisor_code = (value & 7) as u8;
          self.ch4.width_mode = value & (1 << 3) != 0;
          self.ch4.clock_shift = ((value >> 4) & 0xF) as u8;
          self.ch4.length_enabled = value & (1 << 14) != 0;
          if value & (1 << 15) != 0 {
            self.ch4.trigger();
          }
        }
        0x20 => {
          self.psg_volume_right = (value & 7) as u8;
          self.psg_volume_left = ((value >> 4) & 7) as u8;
          for i in 0..4 {
            self.psg_enable_right[i] = value & (1 << (8 + i)) != 0;
            self.psg_enable_left[i] = value & (1 << (12 + i)) != 0;
          }
        }
        0x22 => {
          self.psg_master_volume = (value & 3) as u8;
          self.fifo_a.volume_full = value & (1 << 2) != 0;
          self.fifo_b.volume_full = value & (1 << 3) != 0;
          self.fifo_a.enable_right = value & (1 << 8) != 0;
          self.fifo_a.enable_left = value & (1 << 9) != 0;
          self.fifo_a.timer_select = ((value >> 10) & 1) as u8;
          if value & (1 << 11) != 0 {
            self.fifo_a.reset();
          }
          self.fifo_b.enable_right = value & (1 << 12) != 0;
          self.fifo_b.enable_left = value & (1 << 13) != 0;
          self.fifo_b.timer_select = ((value >> 14) & 1) as u8;
          if value & (1 << 15) != 0 {
            self.fifo_b.reset();
          }
        }
        0x24 => {
          self.master_enable = value & (1 << 7) != 0;
          if !self.master_enable {
            self.ch1.enabled = false;
            self.ch2.enabled = false;
            self.ch3.enabled = false;
            self.ch4.enabled = false;
          }
        }
        0x28 => {
          self.bias_level = value & 0x3FF;
        }
        0x30..=0x3F => {
          let idx = (offset - 0x30) as usize * 2;
          let bytes = value.to_le_bytes();
          if idx < 32 {
            self.ch3.wave_ram[idx] = bytes[0];
          }
          if idx + 1 < 32 {
            self.ch3.wave_ram[idx + 1] = bytes[1];
          }
        }
        0x40 => self.fifo_a.write16(value),
        0x42 => self.fifo_a.write16(value),
        0x44 => self.fifo_b.write16(value),
        0x46 => self.fifo_b.write16(value),
        _ => {}
      }
    }

    pub fn read_reg(&self, offset: u16) -> u16 {
      match offset {
        0x00 => {
          (self.ch1.sweep_shift as u16)
            | ((self.ch1.sweep_negate as u16) << 3)
            | ((self.ch1.sweep_period as u16) << 4)
        }
        0x02 => {
          ((self.ch1.duty as u16) << 6)
            | ((self.ch1.envelope_period as u16) << 8)
            | ((self.ch1.envelope_dir as u16) << 11)
            | ((self.ch1.envelope_init as u16) << 12)
        }
        0x04 => self.ch1.frequency | ((self.ch1.length_enabled as u16) << 14),
        0x08 => {
          ((self.ch2.duty as u16) << 6)
            | ((self.ch2.envelope_period as u16) << 8)
            | ((self.ch2.envelope_dir as u16) << 11)
            | ((self.ch2.envelope_init as u16) << 12)
        }
        0x0C => self.ch2.frequency | ((self.ch2.length_enabled as u16) << 14),
        0x20 => {
          let mut v = (self.psg_volume_right as u16 & 7) | ((self.psg_volume_left as u16 & 7) << 4);
          for i in 0..4 {
            if self.psg_enable_right[i] {
              v |= 1 << (8 + i);
            }
            if self.psg_enable_left[i] {
              v |= 1 << (12 + i);
            }
          }
          v
        }
        0x22 => {
          let mut v = self.psg_master_volume as u16 & 3;
          if self.fifo_a.volume_full {
            v |= 1 << 2;
          }
          if self.fifo_b.volume_full {
            v |= 1 << 3;
          }
          if self.fifo_a.enable_right {
            v |= 1 << 8;
          }
          if self.fifo_a.enable_left {
            v |= 1 << 9;
          }
          v |= (self.fifo_a.timer_select as u16 & 1) << 10;
          if self.fifo_b.enable_right {
            v |= 1 << 12;
          }
          if self.fifo_b.enable_left {
            v |= 1 << 13;
          }
          v |= (self.fifo_b.timer_select as u16 & 1) << 14;
          v
        }
        0x24 => {
          (self.ch1.enabled as u16)
            | ((self.ch2.enabled as u16) << 1)
            | ((self.ch3.enabled as u16) << 2)
            | ((self.ch4.enabled as u16) << 3)
            | ((self.master_enable as u16) << 7)
        }
        0x28 => self.bias_level,
        _ => 0,
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn test_apu_master_enable() {
      let mut apu = Apu::new();
      apu.write_reg(0x24, 1 << 7);
      assert!(apu.master_enable);
      apu.write_reg(0x24, 0);
      assert!(!apu.master_enable);
    }

    #[test]
    fn test_apu_generates_samples_at_48khz() {
      let mut apu = Apu::new();
      apu.master_enable = true;
      apu.sample_buffer_max = 200_000;
      apu.tick(CPU_CLOCK_HZ);
      let n_samples = apu.sample_buffer.len() >> 1;
      assert!(
        (n_samples as i64 - 48000).abs() <= 1,
        "expected ~48000 samples, got {}",
        n_samples
      );
    }

    #[test]
    fn test_fifo_timer_overflow() {
      let mut apu = Apu::new();
      apu.fifo_a.timer_select = 0;
      apu.fifo_a.volume_full = true;
      apu.fifo_a.enable_left = true;
      apu.fifo_a.enable_right = true;
      apu.fifo_a.write32(0x10203040);
      let (needs_a, _) = apu.on_timer_overflow(0);
      assert_eq!(apu.fifo_a.current_sample, 0x40);
      assert!(needs_a);
    }

    #[test]
    fn test_drain_samples_preserves_order_and_remainder() {
      let mut apu = Apu::new();
      apu.sample_buffer = vec![1, 2, 3, 4, 5, 6];
      let mut out = [0; 4];
      assert_eq!(apu.drain_samples(&mut out), 4);
      assert_eq!(out, [1, 2, 3, 4]);
      assert_eq!(apu.sample_buffer, vec![5, 6]);
      let mut out = [0; 4];
      assert_eq!(apu.drain_samples(&mut out), 2);
      assert_eq!(out, [5, 6, 0, 0]);
      assert!(apu.sample_buffer.is_empty());
    }

    #[test]
    fn test_soundcnt_h_parse() {
      let mut apu = Apu::new();
      let val: u16 = (1 << 2) | (1 << 8) | (1 << 9) | (1 << 12) | (1 << 14);
      apu.write_reg(0x22, val);
      assert!(apu.fifo_a.volume_full);
      assert!(apu.fifo_a.enable_right);
      assert!(apu.fifo_a.enable_left);
      assert_eq!(apu.fifo_a.timer_select, 0);
      assert!(!apu.fifo_b.volume_full);
      assert!(apu.fifo_b.enable_right);
      assert!(!apu.fifo_b.enable_left);
      assert_eq!(apu.fifo_b.timer_select, 1);
    }

    #[test]
    fn current_mix_routes_fifo_channels_without_crossfeed() {
      let mut apu = Apu::new();
      apu.fifo_a.current_sample = 10;
      apu.fifo_a.volume_full = true;
      apu.fifo_a.enable_left = true;
      apu.fifo_b.current_sample = -6;
      apu.fifo_b.volume_full = true;
      apu.fifo_b.enable_right = true;
      assert_eq!(apu.current_mix(), (10, -6));
    }
  }
}

pub mod backup {
  pub mod sram {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Sram {
      pub data: Vec<u8>,
    }

    impl Sram {
      pub fn new() -> Self {
        Sram {
          data: vec![0xFF; 32 * 1024],
        }
      }

      pub fn read(&self, addr: u32) -> u8 {
        let index = (addr & 0x7FFF) as usize;
        if index < self.data.len() {
          self.data[index]
        } else {
          0xFF
        }
      }

      pub fn write(&mut self, addr: u32, val: u8) {
        let index = (addr & 0x7FFF) as usize;
        if index < self.data.len() {
          self.data[index] = val;
        }
      }
    }
  }
  pub mod flash {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    enum FlashState {
      Ready,
      Cmd1,
      Cmd2,
      ChipId,
      PrepareErase,
      Erase1,
      Erase2,
      WriteByte,
      BankSwitch,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Flash {
      pub data: Vec<u8>,
      state: FlashState,
      bank: u8,
      size: usize,
      #[serde(default, skip)]
      busy_cycles: u32,
      #[serde(default)]
      reported_id: (u8, u8),
    }

    impl Flash {
      pub fn new(size: usize) -> Self {
        Self::new_with_rom(size, None)
      }

      pub fn new_with_rom(size: usize, rom: Option<&[u8]>) -> Self {
        let reported_id = pick_chip_id(size, rom);
        Flash {
          data: vec![0xFF; size],
          state: FlashState::Ready,
          bank: 0,
          size,
          busy_cycles: 0,
          reported_id,
        }
      }

      fn is_128k(&self) -> bool {
        self.size > 64 * 1024
      }

      pub fn is_busy(&self) -> bool {
        self.state != FlashState::Ready || self.busy_cycles > 0
      }

      pub fn tick(&mut self, cycles: u32) {
        self.busy_cycles = self.busy_cycles.saturating_sub(cycles);
      }

      fn chip_id(&self) -> (u8, u8) {
        self.reported_id
      }

      fn bank_offset(&self) -> usize {
        if self.is_128k() {
          self.bank as usize * 0x10000
        } else {
          0
        }
      }

      pub fn read(&self, addr: u32) -> u8 {
        let offset = (addr & 0xFFFF) as usize;
        if self.state == FlashState::ChipId {
          let (manufacturer, device) = self.chip_id();
          return match offset {
            0 => manufacturer,
            1 => device,
            _ => 0,
          };
        }
        let index = self.bank_offset() + offset;
        if index < self.data.len() {
          self.data[index]
        } else {
          0xFF
        }
      }

      pub fn write(&mut self, addr: u32, val: u8) {
        let offset = (addr & 0xFFFF) as usize;
        self.busy_cycles = 200_000;
        match self.state {
          FlashState::Ready => {
            if offset == 0x5555 && val == 0xAA {
              self.state = FlashState::Cmd1;
            }
          }
          FlashState::Cmd1 => {
            if offset == 0x2AAA && val == 0x55 {
              self.state = FlashState::Cmd2;
            } else {
              self.state = FlashState::Ready;
            }
          }
          FlashState::Cmd2 => {
            if offset == 0x5555 {
              match val {
                0x90 => self.state = FlashState::ChipId,
                0xF0 => self.state = FlashState::Ready,
                0x80 => self.state = FlashState::PrepareErase,
                0xA0 => self.state = FlashState::WriteByte,
                0xB0 if self.is_128k() => self.state = FlashState::BankSwitch,
                _ => self.state = FlashState::Ready,
              }
            } else {
              self.state = FlashState::Ready;
            }
          }
          FlashState::ChipId => {
            if val == 0xF0 {
              self.state = FlashState::Ready;
            } else if offset == 0x5555 && val == 0xAA {
              self.state = FlashState::Cmd1;
            }
          }
          FlashState::PrepareErase => {
            if offset == 0x5555 && val == 0xAA {
              self.state = FlashState::Erase1;
            } else {
              self.state = FlashState::Ready;
            }
          }
          FlashState::Erase1 => {
            if offset == 0x2AAA && val == 0x55 {
              self.state = FlashState::Erase2;
            } else {
              self.state = FlashState::Ready;
            }
          }
          FlashState::Erase2 => {
            if offset == 0x5555 && val == 0x10 {
              self.data.fill(0xFF);
              self.state = FlashState::Ready;
            } else if val == 0x30 {
              let sector = offset & 0xF000;
              let base = self.bank_offset() + sector;
              let end = (base + 0x1000).min(self.data.len());
              if base < self.data.len() {
                self.data[base..end].fill(0xFF);
              }
              self.state = FlashState::Ready;
            } else {
              self.state = FlashState::Ready;
            }
          }
          FlashState::WriteByte => {
            let index = self.bank_offset() + offset;
            if index < self.data.len() {
              self.data[index] &= val;
            }
            self.state = FlashState::Ready;
          }
          FlashState::BankSwitch => {
            if offset == 0x0000 {
              self.bank = val & 1;
            }
            self.state = FlashState::Ready;
          }
        }
      }
    }

    fn pick_chip_id(size: usize, rom: Option<&[u8]>) -> (u8, u8) {
      let candidates_64k: &[(u8, u8, &str)] = &[
        (0x32, 0x1B, "Panasonic MN63F805MNP"),
        (0xBF, 0xD4, "SST 39VF512"),
        (0xC2, 0x1C, "Macronix MX29L512"),
        (0x62, 0x13, "Sanyo LE26FV10N1TS"),
        (0x1F, 0x3D, "Atmel AT29LV512"),
      ];
      let candidates_128k: &[(u8, u8, &str)] = &[
        (0xC2, 0x09, "Macronix MX29L1100B"),
        (0xC2, 0x1C, "Macronix MX29L010"),
        (0x62, 0x13, "Sanyo LE26FV10N1TS"),
      ];
      let candidates: &[(u8, u8, &str)] = if size > 64 * 1024 {
        candidates_128k
      } else {
        candidates_64k
      };
      if let Some(rom) = rom {
        for &(mfr, dev, _name) in candidates {
          let id_le = (mfr as u16) | ((dev as u16) << 8);
          let lo = id_le as u8;
          let hi = (id_le >> 8) as u8;
          for i in (0..rom.len().saturating_sub(2)).step_by(2) {
            if rom[i] == lo && rom[i + 1] == hi {
              return (mfr, dev);
            }
          }
        }
      }
      let (mfr, dev, _) = candidates[0];
      (mfr, dev)
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      #[test]
      fn test_flash_chip_id_64k() {
        let mut flash = Flash::new(64 * 1024);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0x90);
        assert_eq!(flash.read(0), 0x32);
        assert_eq!(flash.read(1), 0x1B);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xF0);
        assert_eq!(flash.read(0), 0xFF);
      }

      #[test]
      fn test_flash_write_byte() {
        let mut flash = Flash::new(64 * 1024);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xA0);
        flash.write(0x0100, 0x42);
        assert_eq!(flash.read(0x0100), 0x42);
      }

      #[test]
      fn test_flash_sector_erase() {
        let mut flash = Flash::new(64 * 1024);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xA0);
        flash.write(0x0100, 0x42);
        assert_eq!(flash.read(0x0100), 0x42);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0x80);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x0000, 0x30);
        assert_eq!(flash.read(0x0100), 0xFF);
      }

      #[test]
      fn test_flash_128k_bank_switch() {
        let mut flash = Flash::new(128 * 1024);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xA0);
        flash.write(0x0000, 0xAA);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xB0);
        flash.write(0x0000, 0x01);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xA0);
        flash.write(0x0000, 0xBB);
        assert_eq!(flash.read(0x0000), 0xBB);
        flash.write(0x5555, 0xAA);
        flash.write(0x2AAA, 0x55);
        flash.write(0x5555, 0xB0);
        flash.write(0x0000, 0x00);
        assert_eq!(flash.read(0x0000), 0xAA);
      }
    }
  }
  pub mod eeprom {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    enum State {
      Idle,
      CmdType,
      Address,
      Data,
      WriteStop,
      ReadOut,
      WriteDone,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    enum SizeKind {
      Unknown,
      Small,
      Large,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Eeprom {
      pub data: Vec<u8>,
      state: State,
      size: SizeKind,
      cmd: u8,
      bit_buf: u64,
      bits_in: u32,
      address: u16,
      read_buf: u64,
      read_bits_left: u32,
      addr_width: u32,
    }

    impl Eeprom {
      pub fn new() -> Self {
        Eeprom {
          data: vec![0xFF; 8 * 1024],
          state: State::Idle,
          size: SizeKind::Unknown,
          cmd: 0,
          bit_buf: 0,
          bits_in: 0,
          address: 0,
          read_buf: 0,
          read_bits_left: 0,
          addr_width: 0,
        }
      }

      pub fn load_bytes(&mut self, data: &[u8]) {
        let size = if data.len() <= 512 { 512 } else { 8 * 1024 };
        self.data = vec![0xFF; size];
        let len = data.len().min(size);
        self.data[..len].copy_from_slice(&data[..len]);
        self.size = if size == 512 {
          SizeKind::Small
        } else {
          SizeKind::Large
        };
        self.addr_width = if size == 512 { 6 } else { 14 };
        self.state = State::Idle;
        self.bit_buf = 0;
        self.bits_in = 0;
        self.read_buf = 0;
        self.read_bits_left = 0;
      }

      fn effective_addr_width(&self) -> u32 {
        match self.size {
          SizeKind::Small => 6,
          SizeKind::Large => 14,
          SizeKind::Unknown => {
            if self.addr_width > 0 {
              self.addr_width
            } else {
              14
            }
          }
        }
      }

      pub fn hint_transfer_bits(&mut self, bits: u32) {
        match bits {
          9 | 73 => self.set_size(SizeKind::Small),
          17 | 81 => self.set_size(SizeKind::Large),
          _ => {}
        }
      }

      fn set_size(&mut self, size: SizeKind) {
        let (bytes, width) = match size {
          SizeKind::Small => (512, 6),
          SizeKind::Large => (8 * 1024, 14),
          SizeKind::Unknown => return,
        };
        if self.data.len() != bytes {
          self.data.resize(bytes, 0xFF);
        }
        self.size = size;
        self.addr_width = width;
      }

      pub fn peek(&self, _addr: u32) -> u8 {
        match self.state {
          State::ReadOut if self.read_bits_left > 64 => 0,
          State::ReadOut if self.read_bits_left > 0 => ((self.read_buf >> 63) & 1) as u8,
          State::WriteDone => 1,
          _ => 1,
        }
      }

      pub fn read(&mut self, addr: u32) -> u8 {
        let bit = self.peek(addr);
        if self.state == State::ReadOut && self.read_bits_left > 0 {
          self.read_bits_left -= 1;
          if self.read_bits_left < 64 {
            self.read_buf <<= 1;
          }
          if self.read_bits_left == 0 {
            self.state = State::Idle;
          }
        }
        bit
      }

      pub fn write(&mut self, _addr: u32, val: u8) {
        let bit = (val & 1) as u64;
        match self.state {
          State::Idle | State::WriteDone => {
            self.bit_buf = bit;
            self.bits_in = 1;
            self.state = State::CmdType;
          }
          State::CmdType => {
            self.bit_buf = (self.bit_buf << 1) | bit;
            self.bits_in += 1;
            if self.bits_in == 2 {
              self.cmd = self.bit_buf as u8;
              self.bit_buf = 0;
              self.bits_in = 0;
              if self.cmd == 2 || self.cmd == 3 {
                self.state = State::Address;
              } else {
                self.state = State::Idle;
              }
            }
          }
          State::Address => {
            self.bit_buf = (self.bit_buf << 1) | bit;
            self.bits_in += 1;
            let aw = self.effective_addr_width();
            if self.bits_in >= aw {
              if self.size == SizeKind::Unknown {
                if self.bits_in <= 6 {
                  self.size = SizeKind::Small;
                  self.data.resize(512, 0xFF);
                } else {
                  self.size = SizeKind::Large;
                }
                self.addr_width = self.bits_in;
              }
              let mask = (1u64 << aw) - 1;
              self.address = (self.bit_buf & mask) as u16;
              self.bit_buf = 0;
              self.bits_in = 0;
              self.state = State::Data;
            }
          }
          State::Data => {
            if self.cmd == 3 {
              self.begin_read();
              self.state = State::ReadOut;
            } else {
              self.bit_buf = (self.bit_buf << 1) | bit;
              self.bits_in += 1;
              if self.bits_in == 64 {
                self.store_data(self.bit_buf);
                self.state = State::WriteStop;
                self.bit_buf = 0;
                self.bits_in = 0;
              }
            }
          }
          State::WriteStop => {
            self.state = State::WriteDone;
          }
          State::ReadOut => {}
        }
      }

      fn begin_read(&mut self) {
        let base = self.address as usize * 8;
        let mut val: u64 = 0;
        for i in 0..8 {
          let byte = if base + i < self.data.len() {
            self.data[base + i]
          } else {
            0xFF
          };
          val = (val << 8) | byte as u64;
        }
        self.read_buf = val;
        self.read_bits_left = 68;
      }

      fn store_data(&mut self, data: u64) {
        let base = self.address as usize * 8;
        for i in 0..8 {
          let byte = ((data >> (56 - i * 8)) & 0xFF) as u8;
          if base + i < self.data.len() {
            self.data[base + i] = byte;
          }
        }
      }
    }

    #[cfg(test)]
    mod tests {
      use super::*;
      fn send_bits(eeprom: &mut Eeprom, val: u64, count: u32) {
        for i in (0..count).rev() {
          eeprom.write(0, ((val >> i) & 1) as u8);
        }
      }

      fn read_bits(eeprom: &mut Eeprom, count: u32) -> u64 {
        let mut result = 0u64;
        for _ in 0..count {
          let bit = eeprom.read(0) as u64;
          result = (result << 1) | bit;
        }
        result
      }

      #[test]
      fn test_eeprom_write_read_small() {
        let mut eeprom = Eeprom::new();
        eeprom.set_size(SizeKind::Small);
        let data: u64 = 0x0102030405060708;
        send_bits(&mut eeprom, 0b10, 2);
        send_bits(&mut eeprom, 0, 6);
        send_bits(&mut eeprom, data, 64);
        send_bits(&mut eeprom, 0, 1);
        assert_eq!(eeprom.data[0], 0x01);
        assert_eq!(eeprom.data[7], 0x08);
        send_bits(&mut eeprom, 0b11, 2);
        send_bits(&mut eeprom, 0, 6);
        send_bits(&mut eeprom, 0, 1);
        let _dummy = read_bits(&mut eeprom, 4);
        let result = read_bits(&mut eeprom, 64);
        assert_eq!(result, data);
      }

      #[test]
      fn test_eeprom_write_done_ready() {
        let mut eeprom = Eeprom::new();
        eeprom.set_size(SizeKind::Small);
        send_bits(&mut eeprom, 0b10, 2);
        send_bits(&mut eeprom, 0, 6);
        send_bits(&mut eeprom, 0xAAAAAAAAAAAAAAAA, 64);
        send_bits(&mut eeprom, 0, 1);
        assert_eq!(eeprom.read(0), 1);
      }

      #[test]
      fn eeprom_write_preserves_first_data_bit() {
        let mut eeprom = Eeprom::new();
        eeprom.set_size(SizeKind::Small);
        send_bits(&mut eeprom, 0b10, 2);
        send_bits(&mut eeprom, 0, 6);
        send_bits(&mut eeprom, 0xFEDC_BA98_7654_3210, 64);
        send_bits(&mut eeprom, 0, 1);
        assert_eq!(&eeprom.data[..8], &0xFEDC_BA98_7654_3210u64.to_be_bytes());
      }

      #[test]
      fn eeprom_readout_advances_on_reads() {
        let mut eeprom = Eeprom::new();
        eeprom.set_size(SizeKind::Small);
        eeprom.data[..8].copy_from_slice(&0x8000_0000_0000_0001u64.to_be_bytes());
        send_bits(&mut eeprom, 0b11, 2);
        send_bits(&mut eeprom, 0, 6);
        send_bits(&mut eeprom, 0, 1);
        let mut bits = 0u128;
        for _ in 0..68 {
          bits = (bits << 1) | eeprom.read(0) as u128;
        }
        assert_eq!(bits >> 64, 0);
        assert_eq!(bits as u64, 0x8000_0000_0000_0001);
      }

      #[test]
      fn eeprom_dma_transfer_size_hint_selects_capacity() {
        let mut eeprom = Eeprom::new();
        eeprom.hint_transfer_bits(9);
        assert_eq!(eeprom.data.len(), 512);
        assert_eq!(eeprom.effective_addr_width(), 6);
        eeprom.hint_transfer_bits(17);
        assert_eq!(eeprom.data.len(), 8 * 1024);
        assert_eq!(eeprom.effective_addr_width(), 14);
      }
    }
  }
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub enum BackupMedia {
    None,
    Sram(sram::Sram),
    Flash(flash::Flash),
    Eeprom(eeprom::Eeprom),
  }

  impl BackupMedia {
    pub fn peek(&self, addr: u32) -> u8 {
      match self {
        BackupMedia::None => 0xFF,
        BackupMedia::Sram(s) => s.read(addr),
        BackupMedia::Flash(f) => f.read(addr),
        BackupMedia::Eeprom(e) => e.peek(addr),
      }
    }

    pub fn read(&mut self, addr: u32) -> u8 {
      match self {
        BackupMedia::None => 0xFF,
        BackupMedia::Sram(s) => s.read(addr),
        BackupMedia::Flash(f) => f.read(addr),
        BackupMedia::Eeprom(e) => e.read(addr),
      }
    }

    pub fn write(&mut self, addr: u32, val: u8) {
      match self {
        BackupMedia::None => {}
        BackupMedia::Sram(s) => s.write(addr, val),
        BackupMedia::Flash(f) => f.write(addr, val),
        BackupMedia::Eeprom(e) => e.write(addr, val),
      }
    }

    pub fn is_busy(&self) -> bool {
      match self {
        BackupMedia::Flash(f) => f.is_busy(),
        _ => false,
      }
    }

    pub fn tick(&mut self, cycles: u32) {
      if let BackupMedia::Flash(f) = self {
        f.tick(cycles);
      }
    }

    pub fn to_raw(&self) -> Option<Vec<u8>> {
      match self {
        BackupMedia::None => None,
        BackupMedia::Sram(s) => Some(s.data.clone()),
        BackupMedia::Flash(f) => Some(f.data.clone()),
        BackupMedia::Eeprom(e) => Some(e.data.clone()),
      }
    }
  }

  pub fn detect_backup_type(rom: &[u8]) -> BackupMedia {
    let rom_str = String::from_utf8_lossy(rom);
    if rom_str.contains("SRAM_V") || rom_str.contains("SRAM_F_V") {
      BackupMedia::Sram(sram::Sram::new())
    } else if rom_str.contains("FLASH1M_V") {
      BackupMedia::Flash(flash::Flash::new_with_rom(128 * 1024, Some(rom)))
    } else if rom_str.contains("FLASH_V") || rom_str.contains("FLASH512_V") {
      BackupMedia::Flash(flash::Flash::new_with_rom(64 * 1024, Some(rom)))
    } else if rom_str.contains("EEPROM_V") {
      BackupMedia::Eeprom(eeprom::Eeprom::new())
    } else {
      BackupMedia::None
    }
  }
}

pub mod bios {
  use crate::arm7tdmi::Cpu;
  use crate::bus::Bus;
  pub fn handle_swi(cpu: &mut Cpu, bus: &mut Bus, comment: u8) -> bool {
    match comment {
      0x00 => swi_soft_reset(cpu, bus),
      0x01 => swi_register_ram_reset(cpu, bus),
      0x02 => swi_halt(cpu),
      0x03 => swi_stop(cpu),
      0x04 => swi_intr_wait(cpu, bus),
      0x05 => swi_vblank_intr_wait(cpu, bus),
      0x06 => swi_div(cpu),
      0x07 => swi_div_arm(cpu),
      0x08 => swi_sqrt(cpu),
      0x09 => swi_arctan(cpu),
      0x0A => swi_arctan2(cpu),
      0x0B => swi_cpu_set(cpu, bus),
      0x0C => swi_cpu_fast_set(cpu, bus),
      0x0D => swi_get_bios_checksum(cpu),
      0x0E => swi_bg_affine_set(cpu, bus),
      0x0F => swi_obj_affine_set(cpu, bus),
      0x10 => swi_bit_unpack(cpu, bus),
      0x11 => swi_lz77_uncomp_wram(cpu, bus),
      0x12 => swi_lz77_uncomp_vram(cpu, bus),
      0x13 => swi_huffman_uncomp(cpu, bus),
      0x14 => swi_rl_uncomp_wram(cpu, bus),
      0x15 => swi_rl_uncomp_vram(cpu, bus),
      0x1D => swi_sound_driver_vsync(bus),
      _ => {
        eprintln!("Unhandled SWI 0x{:02X}", comment);
        return false;
      }
    }
    bus.bios_latch = 0xE3A0_2004;
    true
  }

  fn swi_sound_driver_vsync(bus: &mut Bus) {
    for ch in [1usize, 2usize] {
      let ctl = bus.dma.channels[ch].control;
      bus.write_dma_control(ch, ctl & !(1 << 15));
      bus.write_dma_control(ch, ctl);
    }
  }

  fn swi_soft_reset(cpu: &mut Cpu, bus: &mut Bus) {
    let flag = bus.read8(0x0300_7FFA);
    for addr in 0x0300_7E00..0x0300_8000u32 {
      bus.write8(addr, 0);
    }
    for i in 0..13 {
      cpu.regs[i] = 0;
    }
    cpu.regs[14] = 0;
    cpu.regs[13] = 0x0300_7F00;
    cpu.banked.sp[crate::arm7tdmi::CpuMode::Irq.bank_index()] = 0x0300_7FA0;
    cpu.banked.sp[crate::arm7tdmi::CpuMode::Supervisor.bank_index()] = 0x0300_7FE0;
    if flag != 0 {
      cpu.regs[15] = 0x0200_0000;
    } else {
      cpu.regs[15] = 0x0800_0000;
    }
    cpu.cpsr = crate::arm7tdmi::Psr::new(crate::arm7tdmi::CpuMode::System);
    cpu.cpsr.bits &= !(1 << 7);
    cpu.pipeline_flushed = true;
  }

  fn swi_register_ram_reset(cpu: &mut Cpu, bus: &mut Bus) {
    let flags = cpu.regs[0];
    if flags & (1 << 0) != 0 {
      for addr in (0x0200_0000..0x0204_0000u32).step_by(4) {
        bus.write32(addr, 0);
      }
    }
    if flags & (1 << 1) != 0 {
      for addr in (0x0300_0000..0x0300_7E00u32).step_by(4) {
        bus.write32(addr, 0);
      }
    }
    if flags & (1 << 2) != 0 {
      for addr in (0x0500_0000..0x0500_0400u32).step_by(4) {
        bus.write32(addr, 0);
      }
    }
    if flags & (1 << 3) != 0 {
      for addr in (0x0600_0000..0x0601_8000u32).step_by(4) {
        bus.write32(addr, 0);
      }
    }
    if flags & (1 << 4) != 0 {
      for addr in (0x0700_0000..0x0700_0400u32).step_by(4) {
        bus.write32(addr, 0);
      }
    }
  }

  fn swi_halt(cpu: &mut Cpu) {
    cpu.halted = true;
  }

  fn swi_stop(cpu: &mut Cpu) {
    cpu.halted = true;
  }

  fn swi_intr_wait(cpu: &mut Cpu, bus: &mut Bus) {
    let discard_old = cpu.regs[0] != 0;
    let irq_flags = cpu.regs[1] as u16;
    if discard_old {
      let current = bus.read16(0x0300_7FF8);
      bus.write16(0x0300_7FF8, current & !irq_flags);
    }
    cpu.intrwait_mask = if irq_flags != 0 { irq_flags } else { 0xFFFF };
    cpu.halted = true;
  }

  fn swi_vblank_intr_wait(cpu: &mut Cpu, bus: &mut Bus) {
    cpu.regs[0] = 1;
    cpu.regs[1] = 1;
    swi_intr_wait(cpu, bus);
  }

  fn swi_div(cpu: &mut Cpu) {
    let numer = cpu.regs[0] as i32;
    let denom = cpu.regs[1] as i32;
    if denom == 0 {
      eprintln!("SWI Div: division by zero");
      return;
    }
    cpu.regs[0] = (numer / denom) as u32;
    cpu.regs[1] = (numer % denom) as u32;
    cpu.regs[3] = (numer / denom).unsigned_abs();
  }

  fn swi_div_arm(cpu: &mut Cpu) {
    cpu.regs.swap(0, 1);
    swi_div(cpu);
  }

  fn swi_sqrt(cpu: &mut Cpu) {
    let val = cpu.regs[0];
    cpu.regs[0] = (val as f64).sqrt() as u32;
  }

  fn swi_arctan(cpu: &mut Cpu) {
    let tan = cpu.regs[0] as i16 as f64 / 16384.0;
    let result = tan.atan() * (16384.0 / std::f64::consts::FRAC_PI_2);
    cpu.regs[0] = result as i16 as u16 as u32;
  }

  fn swi_arctan2(cpu: &mut Cpu) {
    let x = cpu.regs[0] as i16 as f64;
    let y = cpu.regs[1] as i16 as f64;
    let result = y.atan2(x);
    let angle = result * (0x8000 as f64 / std::f64::consts::PI);
    cpu.regs[0] = angle as i16 as u16 as u32;
  }

  fn swi_cpu_set(cpu: &mut Cpu, bus: &mut Bus) {
    let src = cpu.regs[0];
    let dst = cpu.regs[1];
    let ctrl = cpu.regs[2];
    let count = ctrl & 0x1F_FFFF;
    let fixed = ctrl & (1 << 24) != 0;
    let word = ctrl & (1 << 26) != 0;
    if word {
      let fill_val = if fixed { bus.read32(src & !3) } else { 0 };
      for i in 0..count {
        let val = if fixed {
          fill_val
        } else {
          bus.read32(src.wrapping_add(i * 4) & !3)
        };
        bus.write32(dst.wrapping_add(i * 4) & !3, val);
      }
    } else {
      let fill_val = if fixed { bus.read16(src & !1) } else { 0 };
      for i in 0..count {
        let val = if fixed {
          fill_val
        } else {
          bus.read16(src.wrapping_add(i * 2) & !1)
        };
        bus.write16(dst.wrapping_add(i * 2) & !1, val);
      }
    }
  }

  fn swi_cpu_fast_set(cpu: &mut Cpu, bus: &mut Bus) {
    let src = cpu.regs[0];
    let dst = cpu.regs[1];
    let ctrl = cpu.regs[2];
    let count = ctrl & 0x1F_FFFF;
    let fixed = ctrl & (1 << 24) != 0;
    let fill_val = if fixed { bus.read32(src & !3) } else { 0 };
    let count_rounded = (count + 7) & !7;
    for i in 0..count_rounded {
      let val = if fixed {
        fill_val
      } else {
        bus.read32(src.wrapping_add(i * 4) & !3)
      };
      bus.write32(dst.wrapping_add(i * 4) & !3, val);
    }
  }

  fn swi_get_bios_checksum(cpu: &mut Cpu) {
    cpu.regs[0] = 0xBAAE_187F;
  }

  fn swi_bg_affine_set(cpu: &mut Cpu, bus: &mut Bus) {
    let src = cpu.regs[0];
    let dst = cpu.regs[1];
    let count = cpu.regs[2];
    for i in 0..count {
      let src_addr = src.wrapping_add(i * 20);
      let dst_addr = dst.wrapping_add(i * 16);
      let cx = bus.read32(src_addr) as i32;
      let cy = bus.read32(src_addr + 4) as i32;
      let disp_x = bus.read16(src_addr + 8) as i16 as i32;
      let disp_y = bus.read16(src_addr + 10) as i16 as i32;
      let sx = bus.read16(src_addr + 12) as i16 as f64 / 256.0;
      let sy = bus.read16(src_addr + 14) as i16 as f64 / 256.0;
      let angle = bus.read16(src_addr + 16);
      let theta = (angle as f64) * 2.0 * std::f64::consts::PI / 65536.0;
      let cos_a = theta.cos();
      let sin_a = theta.sin();
      let pa = (sx * cos_a * 256.0) as i16;
      let pb = (-sx * sin_a * 256.0) as i16;
      let pc = (sy * sin_a * 256.0) as i16;
      let pd = (sy * cos_a * 256.0) as i16;
      let start_x = cx - (pa as i32 * disp_x + pb as i32 * disp_y);
      let start_y = cy - (pc as i32 * disp_x + pd as i32 * disp_y);
      bus.write16(dst_addr, pa as u16);
      bus.write16(dst_addr + 2, pb as u16);
      bus.write16(dst_addr + 4, pc as u16);
      bus.write16(dst_addr + 6, pd as u16);
      bus.write32(dst_addr + 8, start_x as u32);
      bus.write32(dst_addr + 12, start_y as u32);
    }
  }

  fn swi_obj_affine_set(cpu: &mut Cpu, bus: &mut Bus) {
    let src = cpu.regs[0];
    let dst = cpu.regs[1];
    let count = cpu.regs[2];
    let stride = cpu.regs[3];
    for i in 0..count {
      let src_addr = src.wrapping_add(i * 8);
      let dst_addr = dst.wrapping_add(i * stride * 4);
      let sx = bus.read16(src_addr) as i16 as f64 / 256.0;
      let sy = bus.read16(src_addr + 2) as i16 as f64 / 256.0;
      let angle = bus.read16(src_addr + 4);
      let theta = (angle as f64) * 2.0 * std::f64::consts::PI / 65536.0;
      let cos_a = theta.cos();
      let sin_a = theta.sin();
      let pa = (sx * cos_a * 256.0) as i16;
      let pb = (-sx * sin_a * 256.0) as i16;
      let pc = (sy * sin_a * 256.0) as i16;
      let pd = (sy * cos_a * 256.0) as i16;
      let offset = stride;
      bus.write16(dst_addr, pa as u16);
      bus.write16(dst_addr + offset, pb as u16);
      bus.write16(dst_addr + offset * 2, pc as u16);
      bus.write16(dst_addr + offset * 3, pd as u16);
    }
  }

  fn swi_bit_unpack(cpu: &mut Cpu, bus: &mut Bus) {
    let src = cpu.regs[0];
    let dst = cpu.regs[1];
    let info_ptr = cpu.regs[2];
    let src_len = bus.read16(info_ptr) as u32;
    let src_width = bus.read8(info_ptr + 2);
    let dst_width = bus.read8(info_ptr + 3);
    let data_offset = bus.read32(info_ptr + 4);
    let zero_flag = data_offset & (1 << 31) != 0;
    let data_offset = data_offset & 0x7FFF_FFFF;
    if src_width == 0 || dst_width == 0 {
      return;
    }
    let mut src_pos = 0u32;
    let mut dst_pos = 0u32;
    let mut dst_buffer = 0u32;
    let mut dst_bits = 0u32;
    let src_mask = (1u32 << src_width) - 1;
    while src_pos < src_len {
      let byte = bus.read8(src.wrapping_add(src_pos));
      src_pos += 1;
      let mut bit_offset = 0u8;
      while bit_offset < 8 {
        let val = ((byte >> bit_offset) as u32) & src_mask;
        bit_offset += src_width;
        let out = if val == 0 && !zero_flag {
          0
        } else {
          val + data_offset
        };
        dst_buffer |= out << dst_bits;
        dst_bits += dst_width as u32;
        if dst_bits >= 32 {
          bus.write32(dst.wrapping_add(dst_pos), dst_buffer);
          dst_pos += 4;
          dst_buffer = 0;
          dst_bits = 0;
        }
      }
    }
    if dst_bits > 0 {
      bus.write32(dst.wrapping_add(dst_pos), dst_buffer);
    }
  }

  fn swi_lz77_uncomp_wram(cpu: &mut Cpu, bus: &mut Bus) {
    lz77_decompress(cpu.regs[0], cpu.regs[1], bus, false);
  }

  fn swi_lz77_uncomp_vram(cpu: &mut Cpu, bus: &mut Bus) {
    lz77_decompress(cpu.regs[0], cpu.regs[1], bus, true);
  }

  fn lz77_decompress(src: u32, dst: u32, bus: &mut Bus, vram_mode: bool) {
    let header = bus.read32(src);
    let decompressed_size = header >> 8;
    let mut src_pos = src + 4;
    let mut dst_pos = dst;
    let mut remaining = decompressed_size;
    let mut vram_buffer = 0u16;
    let mut vram_byte_count = 0u32;
    while remaining > 0 {
      let flags = bus.read8(src_pos);
      src_pos += 1;
      for bit in (0..8).rev() {
        if remaining == 0 {
          break;
        }
        if flags & (1 << bit) != 0 {
          let byte1 = bus.read8(src_pos) as u32;
          let byte2 = bus.read8(src_pos + 1) as u32;
          src_pos += 2;
          let length = (byte1 >> 4) + 3;
          let offset = (((byte1 & 0xF) << 8) | byte2) + 1;
          for _ in 0..length {
            if remaining == 0 {
              break;
            }
            let val = bus.read8(dst_pos.wrapping_sub(offset));
            if vram_mode {
              if vram_byte_count & 1 == 0 {
                vram_buffer = val as u16;
              } else {
                vram_buffer |= (val as u16) << 8;
                bus.write16(dst_pos & !1, vram_buffer);
              }
              vram_byte_count += 1;
            } else {
              bus.write8(dst_pos, val);
            }
            dst_pos += 1;
            remaining -= 1;
          }
        } else {
          let val = bus.read8(src_pos);
          src_pos += 1;
          if vram_mode {
            if vram_byte_count & 1 == 0 {
              vram_buffer = val as u16;
            } else {
              vram_buffer |= (val as u16) << 8;
              bus.write16(dst_pos & !1, vram_buffer);
            }
            vram_byte_count += 1;
          } else {
            bus.write8(dst_pos, val);
          }
          dst_pos += 1;
          remaining -= 1;
        }
      }
    }
  }

  fn swi_huffman_uncomp(cpu: &mut Cpu, bus: &mut Bus) {
    let src = cpu.regs[0];
    let dst = cpu.regs[1];
    let header = bus.read32(src);
    let data_size = header >> 8;
    let bit_length = header & 0xF;
    if bit_length == 0 || bit_length > 8 {
      return;
    }
    let tree_size = (bus.read8(src + 4) as u32 + 1) * 2;
    let tree_start = src + 5;
    let data_start = tree_start + tree_size - 1;
    let data_start = (data_start + 3) & !3;
    let mut src_pos = data_start;
    let mut dst_pos = dst;
    let mut remaining = data_size;
    let mut dst_buffer = 0u32;
    let mut dst_bits = 0u32;
    let mut node_addr = tree_start;
    while remaining > 0 {
      let data_word = bus.read32(src_pos);
      src_pos += 4;
      for bit_idx in (0..32).rev() {
        if remaining == 0 {
          break;
        }
        let bit = (data_word >> bit_idx) & 1;
        let node = bus.read8(node_addr);
        let child_addr = (node_addr & !1) + ((node & 0x3F) as u32) * 2 + 2 + bit;
        let is_leaf = if bit == 0 {
          node & 0x80 != 0
        } else {
          node & 0x40 != 0
        };
        if is_leaf {
          let leaf_val = bus.read8(child_addr) as u32;
          dst_buffer |= leaf_val << dst_bits;
          dst_bits += bit_length;
          node_addr = tree_start;
          if dst_bits >= 32 {
            bus.write32(dst_pos, dst_buffer);
            dst_pos += 4;
            remaining = remaining.saturating_sub(4);
            dst_buffer = 0;
            dst_bits = 0;
          }
        } else {
          node_addr = child_addr;
        }
      }
    }
  }

  fn swi_rl_uncomp_wram(cpu: &mut Cpu, bus: &mut Bus) {
    rl_decompress(cpu.regs[0], cpu.regs[1], bus, false);
  }

  fn swi_rl_uncomp_vram(cpu: &mut Cpu, bus: &mut Bus) {
    rl_decompress(cpu.regs[0], cpu.regs[1], bus, true);
  }

  fn rl_decompress(src: u32, dst: u32, bus: &mut Bus, vram_mode: bool) {
    let header = bus.read32(src);
    let decompressed_size = header >> 8;
    let mut src_pos = src + 4;
    let mut dst_pos = dst;
    let mut remaining = decompressed_size;
    let mut vram_buffer = 0u16;
    let mut vram_byte_count = 0u32;
    while remaining > 0 {
      let flag = bus.read8(src_pos);
      src_pos += 1;
      if flag & 0x80 != 0 {
        let length = (flag & 0x7F) as u32 + 3;
        let val = bus.read8(src_pos);
        src_pos += 1;
        for _ in 0..length {
          if remaining == 0 {
            break;
          }
          if vram_mode {
            if vram_byte_count & 1 == 0 {
              vram_buffer = val as u16;
            } else {
              vram_buffer |= (val as u16) << 8;
              bus.write16(dst_pos & !1, vram_buffer);
            }
            vram_byte_count += 1;
          } else {
            bus.write8(dst_pos, val);
          }
          dst_pos += 1;
          remaining -= 1;
        }
      } else {
        let length = (flag & 0x7F) as u32 + 1;
        for _ in 0..length {
          if remaining == 0 {
            break;
          }
          let val = bus.read8(src_pos);
          src_pos += 1;
          if vram_mode {
            if vram_byte_count & 1 == 0 {
              vram_buffer = val as u16;
            } else {
              vram_buffer |= (val as u16) << 8;
              bus.write16(dst_pos & !1, vram_buffer);
            }
            vram_byte_count += 1;
          } else {
            bus.write8(dst_pos, val);
          }
          dst_pos += 1;
          remaining -= 1;
        }
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    fn make_cpu_bus() -> (Cpu, Bus) {
      let cpu = Cpu::new_post_bios();
      let bus = Bus::new(None, vec![0; 256]);
      (cpu, bus)
    }

    #[test]
    fn test_swi_div() {
      let (mut cpu, _bus) = make_cpu_bus();
      cpu.regs[0] = 100;
      cpu.regs[1] = 7;
      swi_div(&mut cpu);
      assert_eq!(cpu.regs[0], 14);
      assert_eq!(cpu.regs[1], 2);
      assert_eq!(cpu.regs[3], 14);
    }

    #[test]
    fn test_swi_div_negative() {
      let (mut cpu, _bus) = make_cpu_bus();
      cpu.regs[0] = (-100i32) as u32;
      cpu.regs[1] = 7;
      swi_div(&mut cpu);
      assert_eq!(cpu.regs[0] as i32, -14);
      assert_eq!(cpu.regs[1] as i32, -2);
      assert_eq!(cpu.regs[3], 14);
    }

    #[test]
    fn test_swi_sqrt() {
      let (mut cpu, _bus) = make_cpu_bus();
      cpu.regs[0] = 144;
      swi_sqrt(&mut cpu);
      assert_eq!(cpu.regs[0], 12);
    }

    #[test]
    fn test_swi_cpu_set_fill() {
      let (mut cpu, mut bus) = make_cpu_bus();
      bus.write32(0x0300_0000, 0xDEAD_BEEF);
      cpu.regs[0] = 0x0300_0000;
      cpu.regs[1] = 0x0200_0000;
      cpu.regs[2] = 4 | (1 << 24) | (1 << 26);
      swi_cpu_set(&mut cpu, &mut bus);
      assert_eq!(bus.read32(0x0200_0000), 0xDEAD_BEEF);
      assert_eq!(bus.read32(0x0200_0004), 0xDEAD_BEEF);
      assert_eq!(bus.read32(0x0200_0008), 0xDEAD_BEEF);
      assert_eq!(bus.read32(0x0200_000C), 0xDEAD_BEEF);
    }

    #[test]
    fn test_swi_lz77_decompress() {
      let (mut cpu, mut bus) = make_cpu_bus();
      let src = 0x0200_0000u32;
      let dst = 0x0200_1000u32;
      bus.write32(src, 0x0000_0810);
      bus.write8(src + 4, 0x00);
      for i in 0..8u32 {
        bus.write8(src + 5 + i, (i + 1) as u8);
      }
      cpu.regs[0] = src;
      cpu.regs[1] = dst;
      swi_lz77_uncomp_wram(&mut cpu, &mut bus);
      for i in 0..8u32 {
        assert_eq!(bus.read8(dst + i), (i + 1) as u8);
      }
    }

    #[test]
    fn huffman_walks_child_nodes_until_leaf() {
      let (mut cpu, mut bus) = make_cpu_bus();
      let src = 0x0200_0000u32;
      let dst = 0x0200_1000u32;
      bus.write32(src, (4 << 8) | (2 << 4) | 8);
      bus.write8(src + 4, 3);
      bus.write8(src + 5, 0x80);
      bus.write8(src + 6, b'f');
      bus.write8(src + 7, 0xC0);
      bus.write8(src + 8, b'H');
      bus.write8(src + 9, b'u');
      bus.write32(src + 12, 0xB000_0000);
      cpu.regs[0] = src;
      cpu.regs[1] = dst;
      swi_huffman_uncomp(&mut cpu, &mut bus);
      assert_eq!(bus.read32(dst), u32::from_le_bytes(*b"Huff"));
    }

    #[test]
    fn huffman_invalid_bit_width_returns() {
      let (mut cpu, mut bus) = make_cpu_bus();
      let src = 0x0200_0000u32;
      let dst = 0x0200_1000u32;
      bus.write32(src, (4 << 8) | (2 << 4));
      bus.write32(dst, 0xA5A5_A5A5);
      cpu.regs[0] = src;
      cpu.regs[1] = dst;
      swi_huffman_uncomp(&mut cpu, &mut bus);
      assert_eq!(bus.read32(dst), 0xA5A5_A5A5);
    }

    #[test]
    fn test_swi_rl_decompress() {
      let (mut cpu, mut bus) = make_cpu_bus();
      let src = 0x0200_0000u32;
      let dst = 0x0200_1000u32;
      bus.write32(src, 0x0000_0A30);
      bus.write8(src + 4, 0x82);
      bus.write8(src + 5, 0xAB);
      bus.write8(src + 6, 0x04);
      for i in 0..5u32 {
        bus.write8(src + 7 + i, (0x10 + i) as u8);
      }
      cpu.regs[0] = src;
      cpu.regs[1] = dst;
      swi_rl_uncomp_wram(&mut cpu, &mut bus);
      for i in 0..5u32 {
        assert_eq!(bus.read8(dst + i), 0xAB);
      }
      for i in 0..5u32 {
        assert_eq!(bus.read8(dst + 5 + i), (0x10 + i) as u8);
      }
    }
  }
}

pub mod dma {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum DmaTiming {
    Immediate = 0,
    VBlank = 1,
    HBlank = 2,
    Special = 3,
  }

  #[derive(Debug, Clone, Copy)]
  pub enum AddrControl {
    Increment = 0,
    Decrement = 1,
    Fixed = 2,
    IncrementReload = 3,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct DmaChannel {
    pub sad: u32,
    pub dad: u32,
    pub count: u16,
    pub control: u16,
    pub internal_sad: u32,
    pub internal_dad: u32,
    pub internal_count: u32,
    pub active: bool,
    #[serde(default)]
    pub last_latch_cycle: u64,
  }

  impl DmaChannel {
    pub fn new() -> Self {
      DmaChannel {
        sad: 0,
        dad: 0,
        count: 0,
        control: 0,
        internal_sad: 0,
        internal_dad: 0,
        internal_count: 0,
        active: false,
        last_latch_cycle: 0,
      }
    }

    pub fn is_enabled(&self) -> bool {
      self.control & (1 << 15) != 0
    }

    pub fn timing_mode(&self) -> DmaTiming {
      match (self.control >> 12) & 3 {
        0 => DmaTiming::Immediate,
        1 => DmaTiming::VBlank,
        2 => DmaTiming::HBlank,
        3 => DmaTiming::Special,
        _ => unreachable!(),
      }
    }

    pub fn is_word_transfer(&self) -> bool {
      self.control & (1 << 10) != 0
    }

    pub fn repeat(&self) -> bool {
      self.control & (1 << 9) != 0
    }

    pub fn irq_enabled(&self) -> bool {
      self.control & (1 << 14) != 0
    }

    pub fn dst_addr_control(&self) -> AddrControl {
      match (self.control >> 5) & 3 {
        0 => AddrControl::Increment,
        1 => AddrControl::Decrement,
        2 => AddrControl::Fixed,
        3 => AddrControl::IncrementReload,
        _ => unreachable!(),
      }
    }

    pub fn src_addr_control(&self) -> AddrControl {
      match (self.control >> 7) & 3 {
        0 => AddrControl::Increment,
        1 => AddrControl::Decrement,
        2 => AddrControl::Fixed,
        3 => AddrControl::Increment,
        _ => unreachable!(),
      }
    }

    fn latch(&mut self, channel_id: usize) {
      self.internal_sad = match channel_id {
        0..=2 => self.sad & 0x07FF_FFFF,
        3 => self.sad & 0x0FFF_FFFF,
        _ => self.sad,
      };
      self.internal_dad = match channel_id {
        0..=2 => self.dad & 0x07FF_FFFF,
        3 => self.dad & 0x0FFF_FFFF,
        _ => self.dad,
      };
      let raw_count = self.count as u32;
      self.internal_count = if raw_count == 0 {
        match channel_id {
          0..=2 => 0x4000,
          3 => 0x10000,
          _ => 0x10000,
        }
      } else {
        raw_count
      };
    }

    pub(crate) fn reload_for_repeat(&mut self, channel_id: usize) {
      if let AddrControl::IncrementReload = self.dst_addr_control() {
        self.internal_dad = match channel_id {
          0..=2 => self.dad & 0x07FF_FFFF,
          3 => self.dad & 0x0FFF_FFFF,
          _ => self.dad,
        };
      }
      let raw_count = self.count as u32;
      self.internal_count = if raw_count == 0 {
        match channel_id {
          0..=2 => 0x4000,
          3 => 0x10000,
          _ => 0x10000,
        }
      } else {
        raw_count
      };
    }
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct DmaController {
    pub channels: [DmaChannel; 4],
  }

  impl DmaController {
    pub fn new() -> Self {
      DmaController {
        channels: [
          DmaChannel::new(),
          DmaChannel::new(),
          DmaChannel::new(),
          DmaChannel::new(),
        ],
      }
    }

    pub fn write_control(&mut self, channel_id: usize, value: u16, now: u64) -> Option<usize> {
      let old_enabled = self.channels[channel_id].is_enabled();
      self.channels[channel_id].control = value;
      let new_enabled = self.channels[channel_id].is_enabled();
      if !old_enabled && new_enabled {
        self.channels[channel_id].latch(channel_id);
        self.channels[channel_id].active = true;
        self.channels[channel_id].last_latch_cycle = now;
        if self.channels[channel_id].timing_mode() == DmaTiming::Immediate {
          return Some(channel_id);
        }
      } else if !new_enabled {
        self.channels[channel_id].active = false;
      }
      None
    }

    pub fn active_channels_for(&self, timing: DmaTiming) -> ([usize; 4], usize) {
      let mut channels = [0usize; 4];
      let mut count = 0usize;
      for i in 0..4 {
        if self.channels[i].is_enabled()
          && self.channels[i].active
          && self.channels[i].timing_mode() == timing
        {
          channels[count] = i;
          count += 1;
        }
      }
      (channels, count)
    }
  }
  pub struct DmaTransferResult {
    pub cycles: u32,
    pub irq: bool,
    pub channel_disabled: bool,
  }

  pub fn execute_dma_transfer(
    channel: &mut DmaChannel,
    channel_id: usize,
    memory: &mut DmaMemory,
  ) -> DmaTransferResult {
    let word_size = if channel.is_word_transfer() {
      4u32
    } else {
      2u32
    };
    let count = channel.internal_count;
    let src_step = match channel.src_addr_control() {
      AddrControl::Increment | AddrControl::IncrementReload => word_size as i32,
      AddrControl::Decrement => -(word_size as i32),
      AddrControl::Fixed => 0,
    };
    let dst_step = match channel.dst_addr_control() {
      AddrControl::Increment | AddrControl::IncrementReload => word_size as i32,
      AddrControl::Decrement => -(word_size as i32),
      AddrControl::Fixed => 0,
    };
    let is_fifo =
      (channel_id == 1 || channel_id == 2) && channel.timing_mode() == DmaTiming::Special;
    if is_fifo {
      for _ in 0..4 {
        let val = memory.read32(channel.internal_sad);
        memory.write32(channel.internal_dad, val);
        channel.internal_sad = channel.internal_sad.wrapping_add(4);
      }
      return DmaTransferResult {
        cycles: 4,
        irq: channel.irq_enabled(),
        channel_disabled: false,
      };
    }
    for _ in 0..count {
      if word_size == 4 {
        let val = memory.read32(channel.internal_sad & !3);
        memory.write32(channel.internal_dad & !3, val);
      } else {
        let val = memory.read16(channel.internal_sad & !1);
        memory.write16(channel.internal_dad & !1, val);
      }
      channel.internal_sad = (channel.internal_sad as i32).wrapping_add(src_step) as u32;
      channel.internal_dad = (channel.internal_dad as i32).wrapping_add(dst_step) as u32;
    }
    let irq = channel.irq_enabled();
    let channel_disabled = if channel.repeat() && channel.timing_mode() != DmaTiming::Immediate {
      channel.reload_for_repeat(channel_id);
      false
    } else {
      channel.control &= !(1 << 15);
      channel.active = false;
      true
    };
    DmaTransferResult {
      cycles: count,
      irq,
      channel_disabled,
    }
  }
  pub struct DmaMemory<'a> {
    pub read16_fn: &'a dyn Fn(u32) -> u16,
    pub read32_fn: &'a dyn Fn(u32) -> u32,
    pub write16_fn: &'a mut dyn FnMut(u32, u16),
    pub write32_fn: &'a mut dyn FnMut(u32, u32),
  }
  impl<'a> DmaMemory<'a> {
    pub fn read16(&self, addr: u32) -> u16 {
      (self.read16_fn)(addr)
    }
    pub fn read32(&self, addr: u32) -> u32 {
      (self.read32_fn)(addr)
    }
    pub fn write16(&mut self, addr: u32, val: u16) {
      (self.write16_fn)(addr, val)
    }
    pub fn write32(&mut self, addr: u32, val: u32) {
      (self.write32_fn)(addr, val)
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn test_dma_channel_latch() {
      let mut dma = DmaController::new();
      dma.channels[3].sad = 0x0800_0000;
      dma.channels[3].dad = 0x0600_0000;
      dma.channels[3].count = 100;
      let result = dma.write_control(3, 1 << 15, 0);
      assert_eq!(result, Some(3));
      assert_eq!(dma.channels[3].internal_sad, 0x0800_0000);
      assert_eq!(dma.channels[3].internal_dad, 0x0600_0000);
      assert_eq!(dma.channels[3].internal_count, 100);
    }

    #[test]
    fn test_dma_count_zero_means_max() {
      let mut dma = DmaController::new();
      dma.channels[0].count = 0;
      let _ = dma.write_control(0, 1 << 15, 0);
      assert_eq!(dma.channels[0].internal_count, 0x4000);
      dma.channels[3].count = 0;
      let _ = dma.write_control(3, 1 << 15, 0);
      assert_eq!(dma.channels[3].internal_count, 0x10000);
    }

    #[test]
    fn test_dma_timing_detection() {
      let mut ch = DmaChannel::new();
      ch.control = (1 << 15) | (1 << 12);
      assert_eq!(ch.timing_mode(), DmaTiming::VBlank);
      ch.control = (1 << 15) | (2 << 12);
      assert_eq!(ch.timing_mode(), DmaTiming::HBlank);
    }

    #[test]
    fn active_channels_for_filters_by_timing() {
      let mut dma = DmaController::new();
      dma.channels[0].sad = 0x0800_0000;
      dma.channels[0].dad = 0x0600_0000;
      dma.channels[0].count = 10;
      dma.write_control(0, (1 << 15) | (1 << 12), 0);
      dma.channels[2].sad = 0x0800_0000;
      dma.channels[2].dad = 0x0600_0000;
      dma.channels[2].count = 10;
      dma.write_control(2, (1 << 15) | (2 << 12), 0);
      assert_eq!(
        dma.active_channels_for(DmaTiming::VBlank),
        ([0, 0, 0, 0], 1)
      );
      assert_eq!(
        dma.active_channels_for(DmaTiming::HBlank),
        ([2, 0, 0, 0], 1)
      );
      assert_eq!(
        dma.active_channels_for(DmaTiming::Immediate),
        ([0, 0, 0, 0], 0)
      );
    }

    #[test]
    fn active_channels_for_returns_ordered_fixed_buffer() {
      let mut dma = DmaController::new();
      for channel in [0usize, 2, 3] {
        dma.channels[channel].sad = 0x0800_0000;
        dma.channels[channel].dad = 0x0600_0000;
        dma.channels[channel].count = 1;
        dma.write_control(channel, (1 << 15) | (2 << 12), 0);
      }
      assert_eq!(
        dma.active_channels_for(DmaTiming::HBlank),
        ([0, 2, 3, 0], 3)
      );
    }
  }
}

pub mod timer {
  use serde::{Deserialize, Serialize};
  const PRESCALER_DIVIDERS: [u32; 4] = [1, 64, 256, 1024];

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Timer {
    pub reload: u16,
    pub counter: u16,
    pub control: u16,
    pub(crate) prescaler_counter: u32,
  }

  impl Timer {
    pub fn new() -> Self {
      Timer {
        reload: 0,
        counter: 0,
        control: 0,
        prescaler_counter: 0,
      }
    }

    pub fn is_enabled(&self) -> bool {
      self.control & (1 << 7) != 0
    }

    pub fn cascade(&self) -> bool {
      self.control & (1 << 2) != 0
    }

    pub fn irq_enabled(&self) -> bool {
      self.control & (1 << 6) != 0
    }

    pub fn prescaler(&self) -> u32 {
      PRESCALER_DIVIDERS[(self.control & 3) as usize]
    }
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Timers {
    pub timers: [Timer; 4],
  }
  pub struct TimerTickResult {
    pub irqs: [bool; 4],
    pub timer0_overflow: bool,
    pub timer1_overflow: bool,
  }

  impl Timers {
    pub fn new() -> Self {
      Timers {
        timers: [Timer::new(), Timer::new(), Timer::new(), Timer::new()],
      }
    }

    pub fn read_counter(&self, id: usize) -> u16 {
      self.timers[id].counter
    }

    pub fn write_reload(&mut self, id: usize, value: u16) {
      self.timers[id].reload = value;
    }

    pub fn write_control(&mut self, id: usize, value: u16) {
      let old_enabled = self.timers[id].is_enabled();
      self.timers[id].control = value;
      let new_enabled = self.timers[id].is_enabled();
      if !old_enabled && new_enabled {
        self.timers[id].counter = self.timers[id].reload;
        self.timers[id].prescaler_counter = 0;
      }
    }

    pub fn tick(&mut self, cycles: u32) -> TimerTickResult {
      let mut result = TimerTickResult {
        irqs: [false; 4],
        timer0_overflow: false,
        timer1_overflow: false,
      };
      let mut prev_overflow = false;
      for i in 0..4 {
        if !self.timers[i].is_enabled() {
          prev_overflow = false;
          continue;
        }
        let overflows = if self.timers[i].cascade() && i > 0 {
          if prev_overflow {
            self.increment_timer(i, 1)
          } else {
            0
          }
        } else {
          let prescaler = self.timers[i].prescaler();
          self.timers[i].prescaler_counter += cycles;
          let ticks = self.timers[i].prescaler_counter / prescaler;
          self.timers[i].prescaler_counter %= prescaler;
          if ticks > 0 {
            self.increment_timer(i, ticks)
          } else {
            0
          }
        };
        prev_overflow = overflows > 0;
        if prev_overflow {
          if self.timers[i].irq_enabled() {
            result.irqs[i] = true;
          }
          if i == 0 {
            result.timer0_overflow = true;
          }
          if i == 1 {
            result.timer1_overflow = true;
          }
        }
      }
      result
    }

    pub fn cycles_to_next_fifo_overflow(&self) -> u32 {
      let mut best = u32::MAX;
      for i in 0..2usize {
        let t = &self.timers[i];
        if !t.is_enabled() || (t.cascade() && i > 0) {
          continue;
        }
        let prescaler = t.prescaler();
        let to_next_tick = prescaler - t.prescaler_counter;
        let ticks_to_overflow = (0x10000u32 - t.counter as u32).saturating_sub(1);
        let cycles = to_next_tick + ticks_to_overflow.saturating_mul(prescaler);
        if cycles < best {
          best = cycles;
        }
      }
      best
    }

    fn increment_timer(&mut self, id: usize, ticks: u32) -> u32 {
      let counter = self.timers[id].counter as u32;
      let reload = self.timers[id].reload as u32;
      let max = 0x10000u32;
      let total = counter + ticks;
      if total >= max {
        let range = max - reload;
        if range == 0 {
          self.timers[id].counter = reload as u16;
          return ticks;
        }
        let remaining = total - max;
        let extra_overflows = remaining / range;
        let final_counter = reload + (remaining % range);
        self.timers[id].counter = final_counter as u16;
        1 + extra_overflows
      } else {
        self.timers[id].counter = total as u16;
        0
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn test_timer_basic_tick() {
      let mut timers = Timers::new();
      timers.write_reload(0, 0xFFF0);
      timers.write_control(0, 1 << 7);
      assert_eq!(timers.timers[0].counter, 0xFFF0);
      let result = timers.tick(10);
      assert_eq!(timers.timers[0].counter, 0xFFFA);
      assert!(!result.irqs[0]);
    }

    #[test]
    fn test_timer_overflow() {
      let mut timers = Timers::new();
      timers.write_reload(0, 0xFFF0);
      timers.write_control(0, (1 << 7) | (1 << 6));
      let result = timers.tick(20);
      assert_eq!(timers.timers[0].counter, 0xFFF4);
      assert!(result.irqs[0]);
      assert!(result.timer0_overflow);
    }

    #[test]
    fn test_timer_prescaler() {
      let mut timers = Timers::new();
      timers.write_reload(0, 0);
      timers.write_control(0, (1 << 7) | 1);
      timers.tick(63);
      assert_eq!(timers.timers[0].counter, 0);
      timers.tick(1);
      assert_eq!(timers.timers[0].counter, 1);
    }

    #[test]
    fn test_timer_cascade() {
      let mut timers = Timers::new();
      timers.write_reload(0, 0xFFFF);
      timers.write_control(0, 1 << 7);
      timers.write_reload(1, 0);
      timers.write_control(1, (1 << 7) | (1 << 2));
      assert_eq!(timers.timers[1].counter, 0);
      let result = timers.tick(1);
      assert!(result.timer0_overflow);
      assert_eq!(timers.timers[1].counter, 1);
    }

    #[test]
    fn test_timer_reload_on_enable() {
      let mut timers = Timers::new();
      timers.write_reload(0, 0x1234);
      assert_eq!(timers.timers[0].counter, 0);
      timers.write_control(0, 1 << 7);
      assert_eq!(timers.timers[0].counter, 0x1234);
    }
  }
}

pub mod interrupt {
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Copy)]
  pub enum Irq {
    VBlank = 0,
    HBlank = 1,
    VCountMatch = 2,
    Timer0 = 3,
    Timer1 = 4,
    Timer2 = 5,
    Timer3 = 6,
    Serial = 7,
    Dma0 = 8,
    Dma1 = 9,
    Dma2 = 10,
    Dma3 = 11,
    Keypad = 12,
    GamePak = 13,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct InterruptController {
    pub ie: u16,
    pub ir: u16,
    pub ime: bool,
  }

  impl InterruptController {
    pub fn new() -> Self {
      InterruptController {
        ie: 0,
        ir: 0,
        ime: false,
      }
    }

    pub fn request_irq(&mut self, irq: Irq) {
      self.ir |= 1 << (irq as u16);
    }

    pub fn acknowledge(&mut self, value: u16) {
      self.ir &= !value;
    }

    pub fn has_pending(&self) -> bool {
      self.ime && (self.ie & self.ir) != 0
    }

    pub fn read_ie(&self) -> u16 {
      self.ie
    }

    pub fn write_ie(&mut self, value: u16) {
      self.ie = value;
    }

    pub fn read_if(&self) -> u16 {
      self.ir
    }

    pub fn write_if(&mut self, value: u16) {
      self.acknowledge(value);
    }

    pub fn read_ime(&self) -> u16 {
      self.ime as u16
    }

    pub fn write_ime(&mut self, value: u16) {
      self.ime = value & 1 != 0;
    }
  }
}

pub mod keypad {
  use serde::{Deserialize, Serialize};
  pub const KEY_A: u16 = 1 << 0;
  pub const KEY_B: u16 = 1 << 1;
  pub const KEY_SELECT: u16 = 1 << 2;
  pub const KEY_START: u16 = 1 << 3;
  pub const KEY_RIGHT: u16 = 1 << 4;
  pub const KEY_LEFT: u16 = 1 << 5;
  pub const KEY_UP: u16 = 1 << 6;
  pub const KEY_DOWN: u16 = 1 << 7;
  pub const KEY_R: u16 = 1 << 8;
  pub const KEY_L: u16 = 1 << 9;

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Keypad {
    keyinput: u16,
    pub keycnt: u16,
  }

  impl Keypad {
    pub fn new() -> Self {
      Keypad {
        keyinput: 0x03FF,
        keycnt: 0,
      }
    }

    pub fn set_keys(&mut self, pressed: u16) {
      self.keyinput = !pressed & 0x03FF;
    }

    pub fn read_keyinput(&self) -> u16 {
      self.keyinput
    }

    pub fn check_irq(&self) -> bool {
      if self.keycnt & (1 << 14) == 0 {
        return false;
      }
      let key_mask = self.keycnt & 0x03FF;
      let pressed = !self.keyinput & 0x03FF;
      if self.keycnt & (1 << 15) != 0 {
        (pressed & key_mask) == key_mask
      } else {
        (pressed & key_mask) != 0
      }
    }
  }
}

pub mod rtc {
  use serde::{Deserialize, Serialize};
  const PIN_SCK: u8 = 1 << 0;
  const PIN_SIO: u8 = 1 << 1;
  const PIN_CS: u8 = 1 << 2;

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  enum TransferState {
    Idle,
    ReceivingCommand,
    ReceivingData,
    SendingData,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Rtc {
    pub enabled: bool,
    data_out: u8,
    direction: u8,
    control: u8,
    state: TransferState,
    last_sck: bool,
    cs: bool,
    in_bits: u8,
    in_bit_count: u8,
    out_bits: u8,
    out_bit_count: u8,
    sio_out: bool,
    command: u8,
    data_buf: [u8; 8],
    data_idx: usize,
    data_len: usize,
    status: u8,
  }

  impl Rtc {
    pub fn new() -> Self {
      Rtc {
        enabled: false,
        data_out: 0,
        direction: 0,
        control: 0,
        state: TransferState::Idle,
        last_sck: false,
        cs: false,
        in_bits: 0,
        in_bit_count: 0,
        out_bits: 0,
        out_bit_count: 0,
        sio_out: false,
        command: 0,
        data_buf: [0; 8],
        data_idx: 0,
        data_len: 0,
        status: 0x40,
      }
    }

    pub fn detect(rom: &[u8]) -> bool {
      let needle = b"SIIRTC_V";
      rom.windows(needle.len()).any(|w| w == needle)
    }

    pub fn read_reg(&self, offset: u32) -> u16 {
      if self.control & 1 == 0 {
        return 0;
      }
      match offset {
        0 => {
          let mut val = 0u8;
          val |= self.data_out & (PIN_SCK | PIN_CS);
          if self.direction & PIN_SIO != 0 {
            val |= self.data_out & PIN_SIO;
          } else if self.sio_out {
            val |= PIN_SIO;
          }
          val as u16
        }
        2 => self.direction as u16,
        4 => self.control as u16,
        _ => 0,
      }
    }

    pub fn write_reg(&mut self, offset: u32, value: u16) {
      match offset {
        0 => {
          self.data_out = (value & 0x0F) as u8;
          self.pin_update();
        }
        2 => {
          self.direction = (value & 0x0F) as u8;
        }
        4 => {
          self.control = (value & 1) as u8;
        }
        _ => {}
      }
    }

    fn pin_update(&mut self) {
      let new_cs = self.data_out & PIN_CS != 0;
      let new_sck = self.data_out & PIN_SCK != 0;
      let sio = self.data_out & PIN_SIO != 0;
      if new_cs && !self.cs {
        self.state = TransferState::ReceivingCommand;
        self.in_bits = 0;
        self.in_bit_count = 0;
        self.out_bit_count = 0;
        self.data_idx = 0;
      } else if !new_cs && self.cs {
        self.state = TransferState::Idle;
      }
      self.cs = new_cs;
      if !self.cs {
        self.last_sck = new_sck;
        return;
      }
      let rising = new_sck && !self.last_sck;
      let falling = !new_sck && self.last_sck;
      self.last_sck = new_sck;
      match self.state {
        TransferState::ReceivingCommand => {
          if rising {
            self.in_bits = (self.in_bits << 1) | (sio as u8);
            self.in_bit_count += 1;
            if self.in_bit_count == 8 {
              self.command = self.in_bits;
              self.execute_command();
              self.in_bit_count = 0;
              self.in_bits = 0;
            }
          }
        }
        TransferState::SendingData => {
          if falling {
            if self.out_bit_count == 0 {
              if self.data_idx < self.data_len {
                self.out_bits = self.data_buf[self.data_idx];
                self.data_idx += 1;
                self.out_bit_count = 8;
              } else {
                self.sio_out = false;
                return;
              }
            }
            self.sio_out = self.out_bits & 1 != 0;
            self.out_bits >>= 1;
            self.out_bit_count -= 1;
            if self.out_bit_count == 0 && self.data_idx >= self.data_len {
              self.state = TransferState::Idle;
            }
          }
        }
        TransferState::ReceivingData => {
          if rising {
            self.in_bits |= (sio as u8) << self.in_bit_count;
            self.in_bit_count += 1;
            if self.in_bit_count == 8 {
              if self.data_idx < self.data_buf.len() {
                self.data_buf[self.data_idx] = self.in_bits;
                self.data_idx += 1;
              }
              self.in_bits = 0;
              self.in_bit_count = 0;
              if self.data_idx >= self.data_len {
                self.finish_write();
                self.state = TransferState::Idle;
              }
            }
          }
        }
        TransferState::Idle => {}
      }
    }

    fn execute_command(&mut self) {
      let is_read = self.command & 0x04 != 0;
      let cmd = self.command & 0xF7;
      let _ = is_read;
      match self.command {
        0x60 => {
          self.status = 0x40;
          self.state = TransferState::Idle;
        }
        0x62 => {
          self.data_len = 1;
          self.data_idx = 0;
          self.in_bits = 0;
          self.in_bit_count = 0;
          self.state = TransferState::ReceivingData;
        }
        0x63 => {
          self.data_buf[0] = self.status;
          self.data_len = 1;
          self.data_idx = 0;
          self.out_bit_count = 0;
          self.state = TransferState::SendingData;
        }
        0x64 => {
          self.data_len = 7;
          self.data_idx = 0;
          self.in_bits = 0;
          self.in_bit_count = 0;
          self.state = TransferState::ReceivingData;
        }
        0x65 => {
          self.fill_datetime(7);
          self.state = TransferState::SendingData;
        }
        0x66 => {
          self.data_len = 3;
          self.data_idx = 0;
          self.in_bits = 0;
          self.in_bit_count = 0;
          self.state = TransferState::ReceivingData;
        }
        0x67 => {
          self.fill_datetime(3);
          self.state = TransferState::SendingData;
        }
        _ => {
          self.state = TransferState::Idle;
        }
      }
      let _ = cmd;
    }

    fn finish_write(&mut self) {
      if self.command == 0x62 {
        self.status = self.data_buf[0];
      }
    }

    fn fill_datetime(&mut self, count: usize) {
      let (year, month, day, dow, hour, minute, second) = current_datetime();
      let bcd = |n: u32| -> u8 { ((n / 10) << 4) as u8 | (n % 10) as u8 };
      if count == 7 {
        self.data_buf[0] = bcd(year % 100);
        self.data_buf[1] = bcd(month);
        self.data_buf[2] = bcd(day);
        self.data_buf[3] = bcd(dow);
        self.data_buf[4] = bcd(hour);
        self.data_buf[5] = bcd(minute);
        self.data_buf[6] = bcd(second);
      } else {
        self.data_buf[0] = bcd(hour);
        self.data_buf[1] = bcd(minute);
        self.data_buf[2] = bcd(second);
      }
      self.data_len = count;
      self.data_idx = 0;
      self.out_bit_count = 0;
    }
  }

  fn current_datetime() -> (u32, u32, u32, u32, u32, u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_secs())
      .unwrap_or(0);
    let sec = (secs % 60) as u32;
    let min = ((secs / 60) % 60) as u32;
    let hour = ((secs / 3600) % 24) as u32;
    let days_since_epoch = (secs / 86400) as u32;
    let dow = (days_since_epoch + 4) % 7;
    let (year, month, day) = days_to_ymd(days_since_epoch as i64);
    (year as u32, month, day, dow, hour, min, sec)
  }

  fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    days += 719468;
    let era = if days >= 0 {
      days / 146097
    } else {
      (days - 146096) / 146097
    };
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + (era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn test_rtc_detect() {
      let rom = b"random data SIIRTC_V001 more data".to_vec();
      assert!(Rtc::detect(&rom));
      let rom2 = b"no signature here".to_vec();
      assert!(!Rtc::detect(&rom2));
    }

    #[test]
    fn test_bcd_conversion() {
      let bcd = |n: u32| -> u8 { ((n / 10) << 4) as u8 | (n % 10) as u8 };
      assert_eq!(bcd(0), 0x00);
      assert_eq!(bcd(9), 0x09);
      assert_eq!(bcd(10), 0x10);
      assert_eq!(bcd(59), 0x59);
      assert_eq!(bcd(99), 0x99);
    }

    #[test]
    fn test_days_to_ymd() {
      let (y, m, d) = days_to_ymd(0);
      assert_eq!((y, m, d), (1970, 1, 1));
      let (y, m, d) = days_to_ymd(10957);
      assert_eq!((y, m, d), (2000, 1, 1));
    }

    #[test]
    fn test_rtc_status_read() {
      let mut rtc = Rtc::new();
      rtc.enabled = true;
      rtc.control = 1;
      assert_eq!(rtc.status, 0x40);
      assert_eq!(rtc.status & 0x80, 0);
    }
  }
}

pub mod scheduler {
  use serde::{Deserialize, Serialize};
  use std::cmp::Ordering;
  use std::collections::BinaryHeap;

  #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
  pub enum EventKind {
    HBlank,
    HBlankEnd,
  }

  #[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
  pub struct Event {
    pub fire_time: u64,
    pub kind: EventKind,
  }

  impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
      other.fire_time.cmp(&self.fire_time)
    }
  }

  impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
    }
  }

  #[derive(Serialize, Deserialize)]
  pub struct Scheduler {
    timestamp: u64,
    events: BinaryHeap<Event>,
  }

  impl Scheduler {
    pub fn new() -> Self {
      Scheduler {
        timestamp: 0,
        events: BinaryHeap::new(),
      }
    }

    pub fn timestamp(&self) -> u64 {
      self.timestamp
    }

    pub fn add_cycles(&mut self, cycles: u64) {
      self.timestamp += cycles;
    }

    pub fn advance_to(&mut self, time: u64) {
      self.timestamp = time;
    }

    pub fn schedule(&mut self, event: Event) {
      self.events.push(event);
    }

    pub fn peek_time(&self) -> Option<u64> {
      self.events.peek().map(|e| e.fire_time)
    }

    pub fn pop_if_ready(&mut self) -> Option<Event> {
      if let Some(event) = self.events.peek()
        && event.fire_time <= self.timestamp
      {
        return self.events.pop();
      }
      None
    }
  }
}
macro_rules! impl_default_via_new {
    ($($ty:path),+ $(,)?) => {
        $(
            impl Default for $ty {
                fn default() -> Self {
                    Self::new()
                }
            }
        )+
    };
}
impl_default_via_new!(
  arm7tdmi::BankedRegisters,
  arm7tdmi::Cpu,
  bus::io_regs::IoRegisters,
  ppu::Ppu,
  apu::psg::Channel1,
  apu::psg::Channel2,
  apu::psg::Channel3,
  apu::psg::Channel4,
  apu::fifo::FifoChannel,
  apu::Apu,
  backup::sram::Sram,
  backup::eeprom::Eeprom,
  dma::DmaChannel,
  dma::DmaController,
  timer::Timer,
  timer::Timers,
  interrupt::InterruptController,
  keypad::Keypad,
  rtc::Rtc,
  scheduler::Scheduler,
);
use arm7tdmi::Cpu;
use bus::Bus;
use dma::DmaTiming;
use scheduler::{Event, EventKind, Scheduler};
use serde::{Deserialize, Serialize};
pub const CPU_CLOCK_HZ: u32 = 16_777_216;
pub const CYCLES_PER_DOT: u32 = 4;
pub const DOTS_PER_LINE: u32 = 308;
pub const CYCLES_PER_LINE: u32 = DOTS_PER_LINE * CYCLES_PER_DOT;
pub const VISIBLE_LINES: u16 = 160;
pub const VBLANK_LINES: u16 = 68;
pub const LINES_PER_FRAME: u16 = VISIBLE_LINES + VBLANK_LINES;
pub const CYCLES_PER_FRAME: u64 = CYCLES_PER_LINE as u64 * LINES_PER_FRAME as u64;
pub const SCREEN_WIDTH: usize = 240;
pub const SCREEN_HEIGHT: usize = 160;
pub const HDRAW_CYCLES: u32 = 240 * CYCLES_PER_DOT;
pub const HBLANK_CYCLES: u32 = 68 * CYCLES_PER_DOT;

#[derive(Serialize, Deserialize)]
pub struct Gba {
  pub cpu: Cpu,
  pub bus: Bus,
  pub scheduler: Scheduler,
  frame_buffer: Vec<u16>,
}
const STATE_MAGIC: [u8; 8] = *b"GMASTATE";
const STATE_VERSION: u16 = 1;

#[derive(Serialize)]
struct SaveStateV1<'a> {
  magic: [u8; 8],
  version: u16,
  gba: &'a Gba,
}

#[derive(Deserialize)]
struct SaveStateV1Owned {
  magic: [u8; 8],
  version: u16,
  gba: Gba,
}

fn state_error(message: &str) -> bincode::Error {
  Box::new(bincode::ErrorKind::Custom(message.to_string()))
}

impl Gba {
  pub fn new(bios: Option<Vec<u8>>, rom: Vec<u8>) -> Self {
    let mut scheduler = Scheduler::new();
    scheduler.schedule(Event {
      fire_time: HDRAW_CYCLES as u64,
      kind: EventKind::HBlank,
    });
    let bus = Bus::new(bios, rom);
    let cpu = Cpu::new();
    Gba {
      cpu,
      bus,
      scheduler,
      frame_buffer: vec![0u16; SCREEN_WIDTH * SCREEN_HEIGHT],
    }
  }

  pub fn step_frame(&mut self) -> &[u16] {
    self.advance_cycles(CYCLES_PER_FRAME);
    &self.frame_buffer
  }

  pub fn advance_cycles(&mut self, cycles: u64) {
    let target_time = self.scheduler.timestamp() + cycles;
    while self.scheduler.timestamp() < target_time {
      let next_event_time = self.scheduler.peek_time().unwrap_or(target_time);
      let step_target = next_event_time.min(target_time);
      while self.scheduler.timestamp() < step_target {
        if self.cpu.halted {
          let mut remaining = (step_target - self.scheduler.timestamp()) as u32;
          while remaining > 0 {
            let to_next = self.bus.timers.cycles_to_next_fifo_overflow();
            let chunk = remaining.min(to_next).clamp(1, 1024);
            self.bus.apu.tick(chunk);
            self.scheduler.add_cycles(chunk as u64);
            self.tick_timers(chunk);
            remaining -= chunk;
          }
          break;
        }
        self.step_cpu_once();
      }
      while let Some(event) = self.scheduler.pop_if_ready() {
        self.handle_event(event);
      }
      if self.cpu.halted {
        let pending = self.bus.interrupt.ie & self.bus.interrupt.ir;
        let mask = self.cpu.intrwait_mask;
        let wake = if mask != 0 {
          pending & mask != 0
        } else {
          pending != 0
        };
        if wake {
          self.cpu.halted = false;
          self.cpu.intrwait_mask = 0;
        }
      }
    }
  }

  fn handle_event(&mut self, event: Event) {
    let current_time = self.scheduler.timestamp();
    match event.kind {
      EventKind::HBlank => {
        let line = self.bus.io.vcount;
        self.bus.io.dispstat |= 0x0002;
        if line < VISIBLE_LINES {
          self.bus.ppu.render_scanline(
            line,
            &self.bus.io,
            &self.bus.palette,
            &self.bus.vram,
            &self.bus.oam,
            &mut self.frame_buffer,
          );
        }
        if self.bus.io.dispstat & 0x0010 != 0 {
          self.bus.interrupt.request_irq(interrupt::Irq::HBlank);
        }
        if line < VISIBLE_LINES {
          self.run_dma_for_timing(dma::DmaTiming::HBlank);
        }
        self.scheduler.schedule(Event {
          fire_time: current_time + HBLANK_CYCLES as u64,
          kind: EventKind::HBlankEnd,
        });
      }
      EventKind::HBlankEnd => {
        self.bus.io.dispstat &= !0x0002;
        self.bus.io.vcount = (self.bus.io.vcount + 1) % LINES_PER_FRAME;
        let line = self.bus.io.vcount;
        let lyc = self.bus.io.dispstat >> 8;
        if line == lyc {
          self.bus.io.dispstat |= 0x0004;
          if self.bus.io.dispstat & 0x0020 != 0 {
            self.bus.interrupt.request_irq(interrupt::Irq::VCountMatch);
          }
        } else {
          self.bus.io.dispstat &= !0x0004;
        }
        if line == VISIBLE_LINES {
          self.bus.io.dispstat |= 0x0001;
          if self.bus.io.dispstat & 0x0008 != 0 {
            self.bus.interrupt.request_irq(interrupt::Irq::VBlank);
          }
          self.run_dma_for_timing(dma::DmaTiming::VBlank);
          const RECENT_LATCH_CYCLES: u64 = 2 * CYCLES_PER_FRAME;
          let now = self.scheduler.timestamp();
          for ch in [1usize, 2] {
            let c = &mut self.bus.dma.channels[ch];
            if !(c.active && matches!(c.timing_mode(), dma::DmaTiming::Special)) {
              continue;
            }
            if now.saturating_sub(c.last_latch_cycle) <= RECENT_LATCH_CYCLES {
              continue;
            }
            c.internal_sad = c.sad & 0x07FF_FFFF;
          }
        } else if line == 0 {
          self.bus.io.dispstat &= !0x0001;
          self.bus.ppu.on_vblank(&self.bus.io);
        }
        self.scheduler.schedule(Event {
          fire_time: self.scheduler.timestamp() + HDRAW_CYCLES as u64,
          kind: EventKind::HBlank,
        });
      }
    }
  }

  fn tick_timers(&mut self, cycles: u32) {
    let result = self.bus.timers.tick(cycles);
    const TIMER_IRQS: [interrupt::Irq; 4] = [
      interrupt::Irq::Timer0,
      interrupt::Irq::Timer1,
      interrupt::Irq::Timer2,
      interrupt::Irq::Timer3,
    ];
    for (&irq, &active) in TIMER_IRQS.iter().zip(result.irqs.iter()) {
      if active {
        self.bus.interrupt.request_irq(irq);
      }
    }
    const FIFO_A_ADDR: u32 = 0x0400_00A0;
    const FIFO_B_ADDR: u32 = 0x0400_00A4;
    if result.timer0_overflow {
      let (fifo_a_refill, fifo_b_refill) = self.bus.apu.on_timer_overflow(0);
      if fifo_a_refill {
        self.run_dma_for_fifo(FIFO_A_ADDR);
      }
      if fifo_b_refill {
        self.run_dma_for_fifo(FIFO_B_ADDR);
      }
    }
    if result.timer1_overflow {
      let (fifo_a_refill, fifo_b_refill) = self.bus.apu.on_timer_overflow(1);
      if fifo_a_refill {
        self.run_dma_for_fifo(FIFO_A_ADDR);
      }
      if fifo_b_refill {
        self.run_dma_for_fifo(FIFO_B_ADDR);
      }
    }
  }

  fn run_dma_for_fifo(&mut self, fifo_addr: u32) {
    for ch_id in 0..4 {
      let c = &self.bus.dma.channels[ch_id];
      if c.is_enabled()
        && c.active
        && c.timing_mode() == DmaTiming::Special
        && (c.dad & 0x07FF_FFFF) == (fifo_addr & 0x07FF_FFFF)
      {
        let (_cycles, irq) = self.bus.run_dma(ch_id);
        if irq {
          let irq_type = match ch_id {
            0 => interrupt::Irq::Dma0,
            1 => interrupt::Irq::Dma1,
            2 => interrupt::Irq::Dma2,
            3 => interrupt::Irq::Dma3,
            _ => continue,
          };
          self.bus.interrupt.request_irq(irq_type);
        }
      }
    }
  }

  fn run_dma_for_timing(&mut self, timing: DmaTiming) {
    let (channels, count) = self.bus.dma.active_channels_for(timing);
    for &ch_id in channels[..count].iter() {
      let (_cycles, irq) = self.bus.run_dma(ch_id);
      if irq {
        let irq_type = match ch_id {
          0 => interrupt::Irq::Dma0,
          1 => interrupt::Irq::Dma1,
          2 => interrupt::Irq::Dma2,
          3 => interrupt::Irq::Dma3,
          _ => continue,
        };
        self.bus.interrupt.request_irq(irq_type);
      }
    }
  }

  fn handle_swi(&mut self, swi_num: u8) {
    if self.bus.has_bios {
      self.cpu.software_interrupt(swi_num as u32);
    } else {
      bios::handle_swi(&mut self.cpu, &mut self.bus, swi_num);
    }
  }

  pub fn set_buttons(&mut self, keys: u16) {
    self.bus.keypad.set_keys(keys);
  }

  pub fn frame_buffer(&self) -> &[u16] {
    &self.frame_buffer
  }

  pub fn drain_audio_samples(&mut self, out: &mut [i16]) -> usize {
    self.bus.apu.drain_samples(out)
  }

  pub fn serialize_state(&self) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&SaveStateV1 {
      magic: STATE_MAGIC,
      version: STATE_VERSION,
      gba: self,
    })
  }

  pub fn deserialize_state(&mut self, data: &[u8]) -> Result<(), bincode::Error> {
    let state = Self::deserialize_state_bytes(data)?;
    *self = state;
    Ok(())
  }

  pub fn deserialize_state_bytes(data: &[u8]) -> Result<Gba, bincode::Error> {
    let state: SaveStateV1Owned = bincode::deserialize(data)?;
    if state.magic != STATE_MAGIC {
      return Err(state_error("invalid save-state magic"));
    }
    if state.version != STATE_VERSION {
      return Err(state_error("unsupported save-state version"));
    }
    Ok(state.gba)
  }

  pub fn save_bytes(&self) -> Option<Vec<u8>> {
    self.bus.backup.to_raw()
  }

  pub fn load_save_bytes(&mut self, data: &[u8]) {
    match &mut self.bus.backup {
      backup::BackupMedia::None => {}
      backup::BackupMedia::Sram(s) => {
        let len = data.len().min(s.data.len());
        s.data[..len].copy_from_slice(&data[..len]);
      }
      backup::BackupMedia::Flash(f) => {
        let len = data.len().min(f.data.len());
        f.data[..len].copy_from_slice(&data[..len]);
      }
      backup::BackupMedia::Eeprom(e) => {
        e.load_bytes(data);
      }
    }
  }

  fn step_cpu_once(&mut self) {
    if self.cpu.halted {
      self.scheduler.add_cycles(1);
      self.tick_timers(1);
      self.bus.apu.tick(1);
      return;
    }
    self.bus.now = self.scheduler.timestamp();
    let cycles = self.cpu.step(&mut self.bus) as u64;
    self.scheduler.add_cycles(cycles);
    self.tick_timers(cycles as u32);
    self.bus.apu.tick(cycles as u32);
    if let Some(swi_num) = self.cpu.pending_swi.take() {
      self.handle_swi(swi_num);
    }
    if self.bus.halt_requested {
      self.bus.halt_requested = false;
      self.cpu.halted = true;
    }
  }

  #[cfg(test)]
  fn step_cpu(&mut self, n: usize) {
    for _ in 0..n {
      self.step_cpu_once();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn make_mode3_test_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x100];
    let instructions: &[u32] = &[
      0xE3A0_0404,
      0xE3A0_1C04,
      0xE281_1003,
      0xE1C0_10B0,
      0xE3A0_0406,
      0xE3A0_101F,
      0xE1C0_10B0,
      0xE3A0_1B1F,
      0xE1C0_10B2,
      0xEAFF_FFFE,
    ];
    for (i, &inst) in instructions.iter().enumerate() {
      let offset = i * 4;
      rom[offset..offset + 4].copy_from_slice(&inst.to_le_bytes());
    }
    rom
  }

  fn make_loop_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x100];
    rom[0..4].copy_from_slice(&0xEAFF_FFFEu32.to_le_bytes());
    rom
  }

  #[test]
  fn serialized_state_uses_versioned_wrapper_and_rejects_bad_header() {
    let mut gba = Gba::new(None, make_loop_rom());
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    let data = gba.serialize_state().expect("serialize versioned state");
    assert!(data.starts_with(b"GMASTATE"));
    let mut bad_magic = data.clone();
    bad_magic[0] = b'X';
    assert!(Gba::deserialize_state_bytes(&bad_magic).is_err());
    let mut bad_version = data;
    bad_version[8] = 0xFF;
    assert!(Gba::deserialize_state_bytes(&bad_version).is_err());
  }

  #[test]
  fn stepping_api_advances_cpu_once_per_step() {
    let mut gba = Gba::new(None, make_mode3_test_rom());
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    gba.step_cpu(3);
    assert_eq!(gba.cpu.regs[0], 0x0400_0000);
  }

  #[test]
  fn save_bytes_round_trip_preserves_backup_data() {
    let mut gba = Gba::new(None, make_loop_rom());
    gba.bus.backup = backup::BackupMedia::Sram(backup::sram::Sram::new());
    gba.load_save_bytes(&[1, 2, 3, 4]);
    assert_eq!(gba.save_bytes().unwrap()[..4], [1, 2, 3, 4]);
  }

  #[test]
  fn eeprom_rom_maps_backup_at_0d_region() {
    let mut rom = make_loop_rom();
    rom.extend_from_slice(b"EEPROM_V122");
    let mut gba = Gba::new(None, rom);
    assert!(matches!(gba.bus.backup, backup::BackupMedia::Eeprom(_)));
    assert_eq!(gba.bus.read16(0x0D00_0000), 1);
  }

  #[test]
  fn eeprom_load_save_bytes_preserves_loaded_size() {
    let mut rom = make_loop_rom();
    rom.extend_from_slice(b"EEPROM_V122");
    let mut gba = Gba::new(None, rom);
    gba.load_save_bytes(&vec![0xAA; 512]);
    let save = gba.save_bytes().unwrap();
    assert_eq!(save.len(), 512);
    assert!(save.iter().all(|&b| b == 0xAA));
  }

  #[test]
  fn eeprom_dma_hint_can_downshift_loaded_large_save() {
    let mut rom = vec![0u8; 8 * 1024 * 1024];
    rom.extend_from_slice(b"EEPROM_V124");
    let mut gba = Gba::new(None, rom);
    gba.load_save_bytes(&vec![0xAA; 8 * 1024]);
    gba.bus.write16(0x0200_0000, 0);
    gba.bus.dma.channels[3].sad = 0x0200_0000;
    gba.bus.dma.channels[3].dad = 0x0D00_0000;
    gba.bus.dma.channels[3].count = 9;
    assert_eq!(gba.bus.write_dma_control(3, 1 << 15), Some(3));
    gba.bus.run_dma(3);
    assert_eq!(gba.save_bytes().unwrap().len(), 512);
  }

  #[test]
  fn sio_no_cable_clears_start_and_reports_status() {
    let mut gba = Gba::new(None, make_loop_rom());
    gba.bus.write16(0x0400_012A, 0xFEFE);
    gba.bus.write16(0x0400_0128, 0x6083);
    assert_eq!(gba.bus.read16(0x0400_012A), 0xFEFE);
    assert_eq!(gba.bus.read16(0x0400_0128), 0x600F);
  }

  #[test]
  fn test_mode3_pixel_write() {
    let rom = make_mode3_test_rom();
    let mut gba = Gba::new(None, rom);
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    gba.step_cpu(20);
    assert_eq!(
      gba.bus.io.dispcnt, 0x0403,
      "DISPCNT should be Mode 3 + BG2 enable"
    );
    let pixel0 = u16::from_le_bytes([gba.bus.vram[0], gba.bus.vram[1]]);
    assert_eq!(pixel0, 0x001F, "Pixel (0,0) should be red (0x001F)");
    let pixel1 = u16::from_le_bytes([gba.bus.vram[2], gba.bus.vram[3]]);
    assert_eq!(pixel1, 0x7C00, "Pixel (1,0) should be blue (0x7C00)");
  }

  #[test]
  fn test_mode3_renders_to_frame_buffer() {
    let rom = make_mode3_test_rom();
    let mut gba = Gba::new(None, rom);
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    let fb = gba.step_frame();
    assert_eq!(fb[0], 0x001F, "Framebuffer pixel (0,0) should be red");
    assert_eq!(fb[1], 0x7C00, "Framebuffer pixel (1,0) should be blue");
  }

  #[test]
  fn test_vblank_interrupt() {
    let rom = vec![0u8; 256];
    let mut gba = Gba::new(None, rom);
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    gba.bus.io.dispstat = 0x0008;
    gba.bus.interrupt.write_ie(0x0001);
    gba.step_frame();
    assert!(
      gba.bus.interrupt.read_if() & 1 != 0,
      "VBlank IRQ should be pending"
    );
  }

  #[test]
  fn test_branch_then_mov_pipeline() {
    let mut rom = vec![0u8; 0x300];
    let b_instr: u32 = 0xEA00_0080;
    rom[0..4].copy_from_slice(&b_instr.to_le_bytes());
    let mov_instr: u32 = 0xE3A0_0012;
    rom[0x208..0x20C].copy_from_slice(&mov_instr.to_le_bytes());
    let loop_instr: u32 = 0xEAFF_FFFE;
    rom[0x20C..0x210].copy_from_slice(&loop_instr.to_le_bytes());
    let mut gba = Gba::new(None, rom);
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    gba.step_cpu(3);
    assert_eq!(
      gba.cpu.regs[0], 0x12,
      "R0 should be 0x12 (MOV R0, #0x12 @ 0x08000208); got 0x{:X}. \
             If this is wrong, pipeline ordering is broken.",
      gba.cpu.regs[0]
    );
    assert!(
      gba.cpu.regs[15] >= 0x0800_0000 && gba.cpu.regs[15] < 0x0900_0000,
      "PC should stay in ROM; got 0x{:08X}",
      gba.cpu.regs[15]
    );
  }

  #[test]
  fn test_msr_not_decoded_as_mrs() {
    let mut rom = vec![0u8; 0x100];
    let mov: u32 = 0xE3A0_0012;
    let msr: u32 = 0xE129_F000;
    let loop_: u32 = 0xEAFF_FFFE;
    rom[0..4].copy_from_slice(&mov.to_le_bytes());
    rom[4..8].copy_from_slice(&msr.to_le_bytes());
    rom[8..12].copy_from_slice(&loop_.to_le_bytes());
    let mut gba = Gba::new(None, rom);
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    gba.step_cpu(2);
    assert_eq!(
      gba.cpu.cpsr.mode() as u32,
      0x12,
      "MSR should have switched to FIQ mode (0x12); got mode=0x{:X}",
      gba.cpu.cpsr.mode() as u32
    );
    assert_eq!(
      gba.cpu.regs[0], 0x12,
      "R0 should still be 0x12; got 0x{:X}",
      gba.cpu.regs[0]
    );
    assert_ne!(
      gba.cpu.regs[15], 0x1F,
      "PC should not equal CPSR value (indicates MSR decoded as MRS with Rd=PC)"
    );
  }

  #[test]
  fn test_arm_pc_read_during_execute() {
    let mut rom = vec![0u8; 0x100];
    let mov_pc: u32 = 0xE1A0_000F;
    rom[0..4].copy_from_slice(&mov_pc.to_le_bytes());
    let loop_instr: u32 = 0xEAFF_FFFE;
    rom[4..8].copy_from_slice(&loop_instr.to_le_bytes());
    let mut gba = Gba::new(None, rom);
    gba.cpu = arm7tdmi::Cpu::new_post_bios();
    gba.step_cpu(1);
    assert_eq!(
      gba.cpu.regs[0], 0x0800_0008,
      "R0 should read PC as 0x08000008 (instruction_addr + 8); got 0x{:08X}",
      gba.cpu.regs[0]
    );
  }
}

mod video {
  use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
  use sdl2::pixels::{Color, PixelFormatEnum};
  use sdl2::render::{Canvas, TextureCreator};
  use sdl2::video::{Window, WindowContext};
  pub struct Display {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    pixel_buffer: Vec<u8>,
  }

  #[inline]
  fn pack_bgr555_to_argb8888(framebuffer: &[u16], out: &mut [u8]) {
    for (px, &color) in out.chunks_exact_mut(4).zip(framebuffer.iter()) {
      let r = ((color & 0x1F) as u8) << 3;
      let g = (((color >> 5) & 0x1F) as u8) << 3;
      let b = (((color >> 10) & 0x1F) as u8) << 3;
      let packed = 0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
      px.copy_from_slice(&packed.to_le_bytes());
    }
  }

  impl Display {
    pub fn new(sdl: &sdl2::Sdl, scale: u32) -> Self {
      let video = sdl.video().expect("Failed to initialize SDL2 video");
      let window = video
        .window(
          "GBA Emulator",
          SCREEN_WIDTH as u32 * scale,
          SCREEN_HEIGHT as u32 * scale,
        )
        .position_centered()
        .build()
        .expect("Failed to create window");
      let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .expect("Failed to create canvas");
      canvas.set_draw_color(Color::RGB(0, 0, 0));
      canvas.clear();
      canvas.present();
      let texture_creator = canvas.texture_creator();
      let info = canvas.info();
      eprintln!("SDL2 renderer: {} (flags: 0x{:X})", info.name, info.flags);
      Display {
        canvas,
        texture_creator,
        pixel_buffer: vec![0u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
      }
    }

    pub fn clear_to_red(&mut self) {
      self.canvas.set_draw_color(Color::RGB(255, 0, 0));
      self.canvas.clear();
      self.canvas.present();
    }

    pub fn render(&mut self, framebuffer: &[u16]) {
      let framebuffer = &framebuffer[..SCREEN_WIDTH * SCREEN_HEIGHT];
      pack_bgr555_to_argb8888(framebuffer, &mut self.pixel_buffer);
      let mut texture = self
        .texture_creator
        .create_texture_streaming(
          PixelFormatEnum::ARGB8888,
          SCREEN_WIDTH as u32,
          SCREEN_HEIGHT as u32,
        )
        .expect("Failed to create texture");
      texture
        .update(None, &self.pixel_buffer, SCREEN_WIDTH * 4)
        .expect("Failed to update texture");
      self.canvas.set_draw_color(Color::RGB(0, 0, 0));
      self.canvas.clear();
      self
        .canvas
        .copy(&texture, None, None)
        .expect("Failed to copy texture");
      self.canvas.present();
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn bgr555_to_argb8888_packs_expected_little_endian_bytes() {
      let mut out = [0u8; 16];
      pack_bgr555_to_argb8888(&[0x001F, 0x03E0, 0x7C00, 0x7FFF], &mut out);
      assert_eq!(
        out,
        [
          0x00, 0x00, 0xF8, 0xFF, 0x00, 0xF8, 0x00, 0xFF, 0xF8, 0x00, 0x00, 0xFF, 0xF8, 0xF8, 0xF8,
          0xFF,
        ]
      );
    }
  }
}

mod audio {
  use std::collections::VecDeque;
  use std::sync::{Arc, Mutex};
  pub struct AudioBuffer {
    buffer: Arc<Mutex<VecDeque<i16>>>,
  }
  const BUFFER_CAP: usize = 8192;
  impl AudioBuffer {
    pub fn new() -> Self {
      let mut initial = VecDeque::with_capacity(BUFFER_CAP);
      initial.resize(BUFFER_TARGET, 0);
      AudioBuffer {
        buffer: Arc::new(Mutex::new(initial)),
      }
    }

    pub fn enqueue_samples(&self, samples: &[i16]) {
      if let Ok(mut buf) = self.buffer.lock() {
        let samples = &samples[samples.len().saturating_sub(BUFFER_CAP)..];
        let excess = buf
          .len()
          .saturating_add(samples.len())
          .saturating_sub(BUFFER_CAP);
        if excess > 0 {
          let drain_len = excess.min(buf.len());
          buf.drain(..drain_len);
        }
        buf.extend(samples.iter().copied());
      }
    }

    pub fn shared_queue(&self) -> Arc<Mutex<VecDeque<i16>>> {
      self.buffer.clone()
    }

    pub fn queued_samples(&self) -> usize {
      self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }
  }
  pub const BUFFER_HIGH: usize = 6144;
  pub const BUFFER_TARGET: usize = 3072;
  pub fn init_audio(
    sdl: &sdl2::Sdl,
  ) -> Option<(AudioBuffer, sdl2::audio::AudioDevice<AudioCallback>)> {
    let audio_subsystem = sdl.audio().ok()?;
    let desired_spec = sdl2::audio::AudioSpecDesired {
      freq: Some(48_000),
      channels: Some(2),
      samples: Some(1024),
    };
    let buffer = AudioBuffer::new();
    let shared = buffer.shared_queue();
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        eprintln!(
            "SDL2 audio: requested freq=48000 samples=1024, obtained freq={} Hz, channels={}, samples={}, format={:?}",
            spec.freq, spec.channels, spec.samples, spec.format
        );
        AudioCallback { buffer: shared }
    }).ok()?;
    device.resume();
    Some((buffer, device))
  }
  pub struct AudioCallback {
    buffer: Arc<Mutex<VecDeque<i16>>>,
  }

  impl sdl2::audio::AudioCallback for AudioCallback {
    type Channel = i16;
    fn callback(&mut self, out: &mut [i16]) {
      out.fill(0);
      if let Ok(mut buf) = self.buffer.lock() {
        let available = buf.len().min(out.len());
        if available > 0 {
          for (dst, sample) in out[..available].iter_mut().zip(buf.drain(..available)) {
            *dst = sample;
          }
        }
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn callback_drains_enqueued_samples_in_fifo_order() {
      let audio = AudioBuffer::new();
      audio.buffer.lock().unwrap().clear();
      audio.enqueue_samples(&[1, 2, 3, 4]);
      let mut callback = AudioCallback {
        buffer: audio.shared_queue(),
      };
      let mut out = [0; 6];
      <AudioCallback as sdl2::audio::AudioCallback>::callback(&mut callback, &mut out);
      assert_eq!(out, [1, 2, 3, 4, 0, 0]);
      assert_eq!(audio.queued_samples(), 0);
    }

    #[test]
    fn enqueue_samples_drops_oldest_when_over_capacity() {
      let audio = AudioBuffer::new();
      audio.buffer.lock().unwrap().clear();
      let samples: Vec<i16> = (0..(BUFFER_CAP + 4)).map(|n| n as i16).collect();
      audio.enqueue_samples(&samples);
      let mut callback = AudioCallback {
        buffer: audio.shared_queue(),
      };
      let mut out = [0; 4];
      <AudioCallback as sdl2::audio::AudioCallback>::callback(&mut callback, &mut out);
      assert_eq!(out, [4, 5, 6, 7]);
      assert_eq!(audio.queued_samples(), BUFFER_CAP - 4);
    }
  }
}

mod input {
  use crate::keypad::*;
  use sdl2::keyboard::{KeyboardState, Scancode};

  fn pressed_any(mut down: impl FnMut(Scancode) -> bool, keys: &[Scancode]) -> bool {
    keys.iter().any(|&key| down(key))
  }

  pub fn map_keyboard(mut down: impl FnMut(Scancode) -> bool) -> u16 {
    let mut keys = 0u16;
    if pressed_any(&mut down, &[Scancode::Z, Scancode::J]) {
      keys |= KEY_A;
    }
    if pressed_any(&mut down, &[Scancode::X, Scancode::K]) {
      keys |= KEY_B;
    }
    if pressed_any(
      &mut down,
      &[Scancode::Return, Scancode::KpEnter, Scancode::Space],
    ) {
      keys |= KEY_START;
    }
    if pressed_any(
      &mut down,
      &[
        Scancode::Backspace,
        Scancode::Tab,
        Scancode::LShift,
        Scancode::RShift,
      ],
    ) {
      keys |= KEY_SELECT;
    }
    if pressed_any(&mut down, &[Scancode::Right, Scancode::D]) {
      keys |= KEY_RIGHT;
    }
    if pressed_any(&mut down, &[Scancode::Left, Scancode::A]) {
      keys |= KEY_LEFT;
    }
    if pressed_any(&mut down, &[Scancode::Up, Scancode::W]) {
      keys |= KEY_UP;
    }
    if pressed_any(&mut down, &[Scancode::Down, Scancode::S]) {
      keys |= KEY_DOWN;
    }
    if pressed_any(&mut down, &[Scancode::Q]) {
      keys |= KEY_L;
    }
    if pressed_any(&mut down, &[Scancode::E]) {
      keys |= KEY_R;
    }
    keys
  }

  pub fn read_keyboard(keyboard: &KeyboardState) -> u16 {
    map_keyboard(|key| keyboard.is_scancode_pressed(key))
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    fn keys(active: &[Scancode]) -> u16 {
      map_keyboard(|key| active.contains(&key))
    }

    #[test]
    fn arrows_and_wasd_map_to_dpad() {
      assert_eq!(keys(&[Scancode::Left, Scancode::W]), KEY_LEFT | KEY_UP);
      assert_eq!(keys(&[Scancode::A, Scancode::S]), KEY_LEFT | KEY_DOWN);
    }

    #[test]
    fn face_start_select_and_shoulders_have_alternates() {
      assert_eq!(
        keys(&[
          Scancode::J,
          Scancode::K,
          Scancode::Space,
          Scancode::LShift,
          Scancode::Q,
          Scancode::E,
        ]),
        KEY_A | KEY_B | KEY_START | KEY_SELECT | KEY_L | KEY_R
      );
    }
  }
}

mod harness {
  use crate::{CYCLES_PER_FRAME, Gba};
  use serde_json::{Value, json};
  use std::fs;
  use std::io::{self, Read, Write};
  const GBA_BUTTON_MASK: u16 = 0x03FF;
  pub fn run(bios: Option<Vec<u8>>, skip_bios: bool) {
    let mut h = Harness::new(bios, skip_bios);
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    loop {
      let (req, blob) = match read_frame(&mut reader) {
        Ok(frame) => frame,
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
        Err(e) => {
          eprintln!("[harness] read error: {e}");
          break;
        }
      };
      let cmd = req.get("cmd").and_then(Value::as_str).unwrap_or("");
      let (resp, out_blob, quit) = h.handle(cmd, &req, blob);
      if let Err(e) = write_frame(&mut writer, &resp, &out_blob) {
        eprintln!("[harness] write error: {e}");
        break;
      }
      if quit {
        break;
      }
    }
  }
  struct Harness {
    bios: Option<Vec<u8>>,
    skip_bios: bool,
    rom: Option<Vec<u8>>,
    gba: Option<Gba>,
    frame_index: u64,
    buttons: u16,
    audio_accum: Vec<i16>,
    audio_tmp: Vec<i16>,
    save: Option<Vec<u8>>,
  }

  impl Harness {
    fn new(bios: Option<Vec<u8>>, skip_bios: bool) -> Self {
      Harness {
        bios,
        skip_bios,
        rom: None,
        gba: None,
        frame_index: 0,
        buttons: 0,
        audio_accum: Vec::with_capacity(96_000),
        audio_tmp: vec![0i16; 8192],
        save: None,
      }
    }

    fn build(&mut self) -> Result<(), String> {
      let rom = self.rom.clone().ok_or("no ROM loaded")?;
      let mut gba = Gba::new(self.bios.clone(), rom);
      if self.skip_bios {
        gba.cpu = crate::arm7tdmi::Cpu::new_post_bios();
      }
      if let Some(save) = &self.save {
        gba.load_save_bytes(save);
      }
      self.gba = Some(gba);
      self.frame_index = 0;
      self.buttons = 0;
      self.audio_accum.clear();
      Ok(())
    }

    fn gba_mut(&mut self) -> Result<&mut Gba, String> {
      self.gba.as_mut().ok_or_else(|| "no ROM loaded".to_string())
    }

    fn handle(&mut self, cmd: &str, req: &Value, blob: Vec<u8>) -> (Value, Vec<u8>, bool) {
      match cmd {
        "hello" => (self.hello(), vec![], false),
        "bye" => (json!({"ok": true}), vec![], true),
        "load_rom" => wrap(self.load_rom(req)),
        "load_bios" => wrap(self.load_bios(req)),
        "load_save" => wrap(self.load_save(req)),
        "reset" => wrap(self.build()),
        "set_input" => wrap(self.set_input(req)),
        "step" => self.step(req),
        "get_video" => self.get_video(),
        "get_audio" => self.get_audio(),
        "get_save" => self.get_save(),
        "save_state" => self.serialize_state(),
        "load_state" => wrap(self.deserialize_state(blob)),
        "peek" => self.peek(req),
        other => (
          json!({"ok": false, "error": format!("unknown cmd {other:?}")}),
          vec![],
          false,
        ),
      }
    }

    fn hello(&self) -> Value {
      json!({
          "ok": true,
          "engine": "gba",
          "version": env!("CARGO_PKG_VERSION"),
          "screens": [{"w": 240, "h": 160, "fmt": "BGR555"}],
          "audio": {"rate": crate::apu::OUTPUT_SAMPLE_RATE, "channels": 2, "fmt": "s16le"},
          "buttons": ["A","B","Select","Start","Right","Left","Up","Down","R","L"],
          "has_touch": false,
          "has_extkeys": false,
          "peek": true,
      })
    }

    fn load_rom(&mut self, req: &Value) -> Result<(), String> {
      let path = req
        .get("path")
        .and_then(Value::as_str)
        .ok_or("missing path")?;
      let data = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
      self.rom = Some(data);
      self.build()
    }

    fn load_bios(&mut self, req: &Value) -> Result<(), String> {
      let path = req
        .get("path")
        .and_then(Value::as_str)
        .ok_or("missing path")?;
      let data = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
      self.bios = Some(data);
      if self.rom.is_some() {
        self.build()?;
      }
      Ok(())
    }

    fn load_save(&mut self, req: &Value) -> Result<(), String> {
      let path = req
        .get("path")
        .and_then(Value::as_str)
        .ok_or("missing path")?;
      let data = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
      self.gba_mut()?.load_save_bytes(&data);
      self.save = Some(data);
      Ok(())
    }

    fn set_input(&mut self, req: &Value) -> Result<(), String> {
      let buttons = req.get("buttons").and_then(Value::as_u64).unwrap_or(0) as u16;
      self.buttons = buttons & GBA_BUTTON_MASK;
      Ok(())
    }

    fn step(&mut self, req: &Value) -> (Value, Vec<u8>, bool) {
      let frames = req
        .get("frames")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .max(1);
      let buttons = self.buttons;
      let gba = match self.gba.as_mut() {
        Some(g) => g,
        None => {
          return (
            json!({"ok": false, "error": "no ROM loaded"}),
            vec![],
            false,
          );
        }
      };
      for _ in 0..frames {
        gba.set_buttons(buttons);
        gba.advance_cycles(CYCLES_PER_FRAME);
        loop {
          let n = gba.drain_audio_samples(&mut self.audio_tmp);
          self.audio_accum.extend_from_slice(&self.audio_tmp[..n]);
          if n < self.audio_tmp.len() {
            break;
          }
        }
      }
      self.frame_index += frames;
      (
        json!({"ok": true, "frame_index": self.frame_index}),
        vec![],
        false,
      )
    }

    fn get_video(&mut self) -> (Value, Vec<u8>, bool) {
      let gba = match self.gba.as_ref() {
        Some(g) => g,
        None => {
          return (
            json!({"ok": false, "error": "no ROM loaded"}),
            vec![],
            false,
          );
        }
      };
      let fb = gba.frame_buffer();
      let mut blob = Vec::with_capacity(fb.len() * 2);
      for &px in fb {
        blob.extend_from_slice(&px.to_le_bytes());
      }
      let hdr = json!({
          "ok": true,
          "screens": [{
              "index": 0, "w": 240, "h": 160, "fmt": "BGR555",
              "offset": 0, "len": blob.len(),
          }],
      });
      (hdr, blob, false)
    }

    fn get_audio(&mut self) -> (Value, Vec<u8>, bool) {
      let nsamples = self.audio_accum.len() >> 1;
      let mut blob = Vec::with_capacity(self.audio_accum.len() * 2);
      for &s in &self.audio_accum {
        blob.extend_from_slice(&s.to_le_bytes());
      }
      self.audio_accum.clear();
      let hdr = json!({
          "ok": true,
          "rate": crate::apu::OUTPUT_SAMPLE_RATE,
          "channels": 2,
          "fmt": "s16le",
          "nsamples": nsamples,
      });
      (hdr, blob, false)
    }

    fn get_save(&self) -> (Value, Vec<u8>, bool) {
      match self.gba.as_ref().and_then(Gba::save_bytes) {
        Some(data) => (json!({"ok": true, "len": data.len()}), data, false),
        None => (json!({"ok": true, "len": 0}), vec![], false),
      }
    }

    fn serialize_state(&mut self) -> (Value, Vec<u8>, bool) {
      match self.gba.as_ref().map(|g| g.serialize_state()) {
        Some(Ok(data)) => (json!({"ok": true}), data, false),
        Some(Err(e)) => (
          json!({"ok": false, "error": format!("save_state: {e}")}),
          vec![],
          false,
        ),
        None => (
          json!({"ok": false, "error": "no ROM loaded"}),
          vec![],
          false,
        ),
      }
    }

    fn deserialize_state(&mut self, blob: Vec<u8>) -> Result<(), String> {
      self
        .gba_mut()?
        .deserialize_state(&blob)
        .map_err(|e| format!("load_state: {e}"))
    }

    fn peek(&mut self, req: &Value) -> (Value, Vec<u8>, bool) {
      let addr = req.get("addr").and_then(Value::as_u64).unwrap_or(0) as u32;
      let len = req.get("len").and_then(Value::as_u64).unwrap_or(0) as usize;
      let gba = match self.gba.as_ref() {
        Some(g) => g,
        None => {
          return (
            json!({"ok": false, "error": "no ROM loaded"}),
            vec![],
            false,
          );
        }
      };
      let mut blob = Vec::with_capacity(len);
      for i in 0..len as u32 {
        blob.push(gba.bus.peek8(addr.wrapping_add(i)));
      }
      (json!({"ok": true}), blob, false)
    }
  }

  fn wrap(r: Result<(), String>) -> (Value, Vec<u8>, bool) {
    match r {
      Ok(()) => (json!({"ok": true}), vec![], false),
      Err(e) => (json!({"ok": false, "error": e}), vec![], false),
    }
  }

  fn read_frame(r: &mut impl Read) -> io::Result<(Value, Vec<u8>)> {
    let mut hdr = [0u8; 8];
    r.read_exact(&mut hdr)?;
    let total_len = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as usize;
    let json_len = u32::from_le_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]) as usize;
    if total_len < 4 + json_len {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "bad frame lengths",
      ));
    }
    let mut json_bytes = vec![0u8; json_len];
    r.read_exact(&mut json_bytes)?;
    let mut blob = vec![0u8; total_len - 4 - json_len];
    r.read_exact(&mut blob)?;
    let v: Value = serde_json::from_slice(&json_bytes)
      .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok((v, blob))
  }

  fn write_frame(w: &mut impl Write, header: &Value, blob: &[u8]) -> io::Result<()> {
    let json_bytes = serde_json::to_vec(header)?;
    let total_len = (4 + json_bytes.len() + blob.len()) as u32;
    w.write_all(&total_len.to_le_bytes())?;
    w.write_all(&(json_bytes.len() as u32).to_le_bytes())?;
    w.write_all(&json_bytes)?;
    w.write_all(blob)?;
    w.flush()
  }
}
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Default)]
struct Args {
  rom: Option<String>,
  bios: Option<String>,
  skip_bios: bool,
  scale: u32,
  no_audio: bool,
  harness: bool,
  test_pattern: bool,
  test_solid_red: bool,
}

impl Args {
  fn parse() -> Self {
    let mut args = Args {
      skip_bios: true,
      scale: 3,
      ..Args::default()
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
      match arg.as_str() {
        "-h" | "--help" => {
          print_help();
          std::process::exit(0);
        }
        "-b" | "--bios" => args.bios = Some(next_arg(&mut it, &arg)),
        "-s" | "--scale" => {
          let raw = next_arg(&mut it, &arg);
          args.scale = raw.parse::<u32>().unwrap_or_else(|_| {
            eprintln!("error: --scale expects a positive integer");
            std::process::exit(2);
          });
        }
        "--skip-bios" => args.skip_bios = true,
        "--no-audio" => args.no_audio = true,
        "--harness" => args.harness = true,
        "--test-pattern" => args.test_pattern = true,
        "--test-solid-red" => args.test_solid_red = true,
        _ if arg.starts_with('-') => {
          eprintln!("error: unknown option '{arg}'");
          std::process::exit(2);
        }
        _ if args.rom.is_none() => args.rom = Some(arg),
        _ => {
          eprintln!("error: multiple ROM paths supplied");
          std::process::exit(2);
        }
      }
    }
    args
  }
}

fn next_arg(it: &mut impl Iterator<Item = String>, flag: &str) -> String {
  it.next().unwrap_or_else(|| {
    eprintln!("error: {flag} requires a value");
    std::process::exit(2);
  })
}

fn print_help() {
  println!(
    "Usage: gba-rs [OPTIONS] [ROM]\n\nOptions:\n  -b, --bios <PATH>\n      --skip-bios\n  -s, --scale <N>\n      --no-audio\n      --harness\n      --test-pattern\n      --test-solid-red\n  -h, --help"
  );
}

fn load_bios_arg(args: &Args) -> Option<Vec<u8>> {
  if args.skip_bios {
    return None;
  }
  args.bios.as_ref().map(|path| {
    let data = fs::read(path).unwrap_or_else(|e| {
      eprintln!("Failed to read BIOS '{}': {}", path, e);
      std::process::exit(1);
    });
    if data.len() != 0x4000 {
      eprintln!(
        "Invalid BIOS '{}': expected 16384 bytes, got {}. Extract gba_bios.bin from the zip first.",
        path,
        data.len()
      );
      std::process::exit(2);
    }
    data
  })
}

fn sav_path(rom_path: &str) -> PathBuf {
  let p = Path::new(rom_path);
  p.with_extension("sav")
}

fn state_path(rom_path: &str) -> PathBuf {
  let p = Path::new(rom_path);
  p.with_extension("state")
}

fn load_sav(gba: &mut Gba, path: &Path) {
  if path.exists() {
    match fs::read(path) {
      Ok(data) => {
        gba.load_save_bytes(&data);
        eprintln!("Loaded save from {}", path.display());
      }
      Err(e) => eprintln!("Failed to load save: {}", e),
    }
  }
}

fn backup_path(path: &Path, n: u32) -> PathBuf {
  let mut s = path.as_os_str().to_os_string();
  s.push(format!(".bak-{}", n));
  PathBuf::from(s)
}

fn rotate_sav_backups(path: &Path, slots: u32) {
  if slots == 0 {
    return;
  }
  let _ = fs::remove_file(backup_path(path, slots));
  for n in (1..slots).rev() {
    let src = backup_path(path, n);
    let dst = backup_path(path, n + 1);
    let _ = fs::rename(&src, &dst);
  }
  if path.exists() {
    let bak1 = backup_path(path, 1);
    let _ = fs::rename(path, &bak1);
  }
}

fn save_sav(gba: &Gba, path: &Path) {
  const BACKUP_SLOTS: u32 = 5;
  if let Some(data) = gba.save_bytes() {
    if let Ok(existing) = fs::read(path)
      && existing == data
    {
      return;
    }
    rotate_sav_backups(path, BACKUP_SLOTS);
    match fs::write(path, &data) {
      Ok(()) => eprintln!(
        "Saved to {} (rotated 1 backup, kept up to {})",
        path.display(),
        BACKUP_SLOTS
      ),
      Err(e) => eprintln!("Failed to write save: {}", e),
    }
  }
}

fn save_state(gba: &Gba, path: &Path) {
  match gba.serialize_state() {
    Ok(data) => match zstd::encode_all(data.as_slice(), 3) {
      Ok(compressed) => match fs::write(path, &compressed) {
        Ok(()) => eprintln!("Save state written to {}", path.display()),
        Err(e) => eprintln!("Failed to write save state: {}", e),
      },
      Err(e) => eprintln!("Failed to compress save state: {}", e),
    },
    Err(e) => eprintln!("Failed to serialize state: {}", e),
  }
}

fn load_state(gba: &mut Gba, path: &Path) {
  if !path.exists() {
    eprintln!("No save state found at {}", path.display());
    return;
  }
  match fs::read(path) {
    Ok(compressed) => match zstd::decode_all(compressed.as_slice()) {
      Ok(data) => match gba.deserialize_state(&data) {
        Ok(()) => eprintln!("Loaded save state from {}", path.display()),
        Err(e) => eprintln!("Failed to deserialize state: {}", e),
      },
      Err(e) => eprintln!("Failed to decompress save state: {}", e),
    },
    Err(e) => eprintln!("Failed to read save state: {}", e),
  }
}

fn main() {
  let args = Args::parse();
  let bios = load_bios_arg(&args);
  if args.harness {
    harness::run(bios, args.skip_bios);
    return;
  }
  let rom_path = args.rom.clone().unwrap_or_else(|| {
    eprintln!("error: a ROM path is required (unless --harness)");
    std::process::exit(1);
  });
  let rom = fs::read(&rom_path).unwrap_or_else(|e| {
    eprintln!("Failed to read ROM '{}': {}", rom_path, e);
    std::process::exit(1);
  });
  let mut gba = Gba::new(bios, rom);
  if args.skip_bios {
    gba.cpu = crate::arm7tdmi::Cpu::new_post_bios();
  }
  let sav = sav_path(&rom_path);
  load_sav(&mut gba, &sav);
  let state = state_path(&rom_path);
  let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
  let mut display = video::Display::new(&sdl_context, args.scale);
  let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");
  let audio_state = if !args.no_audio {
    audio::init_audio(&sdl_context)
  } else {
    None
  };
  let frame_duration = Duration::from_nanos(16_742_706);
  let mut audio_tmp = vec![0i16; 4096];
  let mut test_buf = vec![0u16; 240 * 160];
  'running: loop {
    let frame_start = Instant::now();
    for event in event_pump.poll_iter() {
      match event {
        sdl2::event::Event::Quit { .. } => break 'running,
        sdl2::event::Event::KeyDown {
          keycode: Some(key), ..
        } => match key {
          sdl2::keyboard::Keycode::Escape => break 'running,
          sdl2::keyboard::Keycode::RightBracket => save_state(&gba, &state),
          sdl2::keyboard::Keycode::LeftBracket => load_state(&mut gba, &state),
          _ => {}
        },
        _ => {}
      }
    }
    let keyboard = event_pump.keyboard_state();
    let keys = input::read_keyboard(&keyboard);
    gba.set_buttons(keys);
    if args.test_solid_red {
      display.clear_to_red();
      std::thread::sleep(frame_duration);
      continue;
    }
    const CHUNKS_PER_FRAME: u64 = 4;
    const CHUNK_CYCLES: u64 = crate::CYCLES_PER_FRAME / CHUNKS_PER_FRAME;
    if args.test_pattern {
      for y in 0..160 {
        for x in 0..240 {
          let color = match x / 60 {
            0 => 0x001F,
            1 => 0x03E0,
            2 => 0x7C00,
            _ => 0x7FFF,
          };
          test_buf[y * 240 + x] = color;
        }
      }
      display.render(&test_buf);
    } else {
      for _ in 0..CHUNKS_PER_FRAME {
        gba.advance_cycles(CHUNK_CYCLES);
        if let Some((ref audio_buf, ref _device)) = audio_state {
          let n = gba.drain_audio_samples(&mut audio_tmp);
          if n > 0 {
            audio_buf.enqueue_samples(&audio_tmp[..n]);
          }
        }
      }
      display.render(gba.frame_buffer());
    }
    if let Some((ref audio_buf, ref _device)) = audio_state {
      while audio_buf.queued_samples() > audio::BUFFER_HIGH {
        std::thread::sleep(Duration::from_millis(1));
        if audio_buf.queued_samples() <= audio::BUFFER_TARGET {
          break;
        }
      }
    } else {
      let elapsed = frame_start.elapsed();
      if elapsed < frame_duration {
        std::thread::sleep(frame_duration - elapsed);
      }
    }
  }
  save_sav(&gba, &sav);
}
