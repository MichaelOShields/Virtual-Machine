
// use crate::bus::Bus;
// use std::collections::HashMap;

// #[derive(Debug, PartialEq)]
// pub enum JumpCondition {
//     Zero,
//     NotZero,
//     Carry,
//     Sign,
//     Overflow,
// }

// #[derive(Debug, PartialEq)]
// pub enum Operand {
//     Register(u8),
//     Imm8(u8),
//     ImmMem(u16),
// }

// #[derive(Debug, PartialEq)]
// pub enum SingleMode {
//     R,
//     M,
//     I,
// }

// #[derive(Debug, PartialEq)]
// pub enum DoubleMode {
//     Ri,
//     Mr,
//     Rm,
//     Rr,
// }



// #[derive(Debug, PartialEq)]
// pub enum Instruction {
//     Nop,

//     Mov  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },

//     Add  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Sub  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Mul  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Div  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Mod  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },

//     And  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Or   {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Xor  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },

//     Not  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },

//     Cmp  {
//         mode: DoubleMode,
//         lhs: Operand,
//         rhs: Operand,
//     },

//     Jmp  {
//         mode: SingleMode,
//         target: Operand,
//     },
//     Jcc  {
//         mode: SingleMode,
//         cond: JumpCondition,
//         target: Operand,
//     },

//     Push {
//         mode: SingleMode,
//         src: Operand,
//     },
//     Pop  {
//         mode: SingleMode,
//         dst: Operand,
//     },

//     Call {
//         mode: SingleMode,
//         target: Operand,
//     },
//     Ret,

//     Shl  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Shr  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },
//     Sar  {
//         mode: DoubleMode,
//         dst: Operand,
//         src: Operand,
//     },

//     SetSp {
//         mode: SingleMode,
//         src: Operand,
//     },

//     Skip {
//         mode: SingleMode,
//         amount: Operand,
//     },

//     Halt,
// }

// pub struct InstructionHandler {
//     single_modes: HashMap<u16, SingleMode>,
//     double_modes: HashMap<u16, DoubleMode>,
    
// }

// impl InstructionHandler {
//     pub fn new() -> Self {
//         let mut single_modes: HashMap<u16, SingleMode> = HashMap::new();
//         single_modes.insert(0b0000, SingleMode::R);
//         single_modes.insert(0b0001, SingleMode::M);
//         single_modes.insert(0b0010, SingleMode::I);

//         let mut double_modes: HashMap<u16, DoubleMode> = HashMap::new();
//         double_modes.insert(0b0, DoubleMode::Rr);
//         double_modes.insert(0b0001, DoubleMode::Rm);
//         double_modes.insert(0b0010, DoubleMode::Mr);
//         double_modes.insert(0b0011, DoubleMode::Ri);
        
//         Self {
//             single_modes,
//             double_modes,
//         }
//     }
// }

// #[derive(PartialEq, Debug)]
// pub enum Fault {
//     IllegalInstruction,
//     IllegalMemAccess,
//     UnknownAction,
// }

// #[derive(PartialEq, Debug)]
// pub enum CPUExit {
//     InstructionCounterReset,
//     Halt,
// }