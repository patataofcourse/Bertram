/*

   def luma_ver(maj, min, mic):
       return (maj << 16) + (min << 8) + mic

   def luma_dump(f):
       if unpack_from("<2I", f) != (0xdeadc0de, 0xdeadcafe):
           raise Err("Not a Luma3DS crash dump!")

       self.version, processor, self.exc_type = unpack_from("<3I", f, 8)
       self.num_regs, code_size, stack_size, extra_size = unpack_from("<4I", f, 24)
       self.num_regs //= 4
       self.processor, self.core = processor & 0xffff, processor >> 16

       if self.version < luma_ver(1,0,2):
           raise Err(f"Unsupported crash dump (version {print_ver(version)}, minimum supported 1.0.2)")

       self.r = list(unpack_from("<{0}I".format(self.num_regs), f, 40)) # registers
       self.r.extend([None] * max(0, 23 - len(self.r)))
       self.sp, self.lr, self.pc, self.cpsr = self.r[13:17]
       self.dfsr, self.ifsr, self.far = self.r[17:20]
       self.fpexc, self.fpinst, self.fpinst2 = self.r[20:23]

       code_pos = 40 + 4 * self.num_regs
       self.code = f[code_pos : code_pos + code_size]
       stack_pos = code_pos + code_size
       self.stack = f[stack_pos : stack_pos + stack_size]
       extra_pos = stack_pos + stack_size
       self.extra = f[extra_pos : extra_pos + extra_size]
*/

use crate::crash::ExcType;

pub struct LumaVersion {
    pub major: u16,
    pub minor: u8,
    pub micro: u8,
}

impl LumaVersion {
    pub const fn from(major: u16, minor: u8, micro: u8) -> Self {
        Self {
            major,
            minor,
            micro,
        }
    }

    pub const MINIMUM_VERSION: Self = Self::from(1, 0, 2);
}

#[repr(u16)]
pub enum LumaProcessor {
    Arm9 = 9,
    Arm11(u16) = 11,
}

pub struct CrashLuma {
    pub version: LumaVersion,
    pub processor: LumaProcessor,
    pub exception_type: ExcType,
    pub registers: Vec<u32>,
    pub code: Vec<u8>,
    pub stack: Vec<u8>,
    pub extra: Vec<u8>,
}
