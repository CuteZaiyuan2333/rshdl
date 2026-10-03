pub enum Signal<T1, T2> {
    Value(T1),
    Ignore(T2),
}

impl<T1, T2> Signal<T1, T2> {
    fn unpack(self) -> Option<T1>{
        match self {
            Self::Value(a) => Some(a),
            Self::Ignore(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bit(bool);
impl Bit {}
#[derive(Clone, Copy)]
pub struct Byte(u8);
impl Byte {}
pub trait Module {
    type Input;
    type Output;
    fn update(&mut self, input: Self::Input) -> Self::Output;
}

//----------------example----------------
//defines


enum Caltulations{
    Add,
    Sub,
}

enum Compare{
    Bigger,
    Equal,
    Smaller,
}

enum Output{
    Out(Byte),
    Error(Byte),
    PlaceHolder(Byte),
}

//regset
enum RegCommand {
    Read(Byte),
    Write(Byte, Byte),
}
struct Regs([Byte; 16]);
impl Module for Regs {
    type Input = Signal<RegCommand, ()>;
    type Output = Signal<Byte, ()>;
    fn update(&mut self, input: Self::Input) -> Self::Output {
        match input {
            Signal::Value(command) => match command {
                RegCommand::Read(addr) => {
                    return Signal::Value(self.0[addr.0 as usize]);
                }
                RegCommand::Write(addr, content) => {
                    self.0[addr.0 as usize] = content;
                    return Signal::Ignore(());
                }
            },
            Signal::Ignore(_) => {
                return Signal::Ignore(());
            }
        }
    }
}

//alu
enum AluCommand {
    Add(Byte, Byte),
    Sub(Byte, Byte),
}

struct ALU;
impl Module for ALU {
    type Input = Signal<AluCommand, ()>;
    type Output = Signal<Byte, ()>;
    fn update(&mut self, input: Self::Input) -> Self::Output {
        match input {
            Signal::Value(command) => match command {
                AluCommand::Add(a, b) => {
                    return Signal::Value(Byte(a.0 + b.0));
                }
                AluCommand::Sub(a, b) => {
                    return Signal::Value(Byte(a.0 - b.0));
                }
            },
            Signal::Ignore(_) => {
                return Signal::Ignore(());
            }
        }
    }
}

//cond

enum CondCommand {
    Equal(Byte, Byte),
    UnsignedBiger(Byte, Byte),
    UnsignedSmaller(Byte, Byte),
}

struct Cond;
impl Module for Cond {
    type Input = Signal<CondCommand, ()>;
    type Output = Signal<Bit, ()>;
    fn update(&mut self, input: Self::Input) -> Self::Output {
        match input {
            Signal::Value(command) => {
                return Signal::Value(Bit(match command {
                    CondCommand::Equal(a, b) => a.0 == b.0,
                    CondCommand::UnsignedBiger(a, b) => a.0 > b.0,
                    CondCommand::UnsignedSmaller(a, b) => a.0 < b.0,
                }));
            }
            Signal::Ignore(_) => {
                return Signal::Ignore(());
            }
        }
    }
}

//PC

enum PCCommand {
    Set(Byte),
    None,
}
struct PC(Byte);
impl Module for PC {
    type Input = Signal<PCCommand, ()>;
    type Output = Signal<Byte, ()>;
    fn update(&mut self, input: Self::Input) -> Self::Output {
        match input {
            Signal::Value(command) => match command {
                PCCommand::Set(addr) => {
                    self.0 = addr;
                }
                PCCommand::None => {
                    self.0.0 += 1;
                }
            },
            Signal::Ignore(_) => {
                self.0.0 += 1;
            }
        }
        return Signal::Value(self.0);
    }
}

//top

enum ISA {
    Imd(Byte, Byte),
    Move(Byte, Byte),
    Calculate(Caltulations, Byte, Byte, Byte),
    Jump(Compare, Byte, Byte, Byte),
    Output(Byte),
}


struct Top {
    pub pc: PC,
    regs: Regs,
    alu: ALU,
    cond: Cond,
}
impl Module for Top{
    type Input = Signal<ISA, ()>;
    type Output = Signal<Output, ()>;
    fn update(&mut self, input: Self::Input) -> Self::Output{
        match input{
            Signal::Value(command) =>{
                match command{
                    ISA::Imd(target_addr, number) =>{
                        let signal = Signal::Value(RegCommand::Write(target_addr, number));
                        self.regs.update(signal);
                        return Signal::Ignore(())
                    }
                    ISA::Move(target,addr) =>{
                        let temp = self.regs.update(Signal::Value(RegCommand::Read(target))).unpack().unwrap();
                        self.regs.update(Signal::Value(RegCommand::Write(addr, temp)));
                        return Signal::Ignore(())
                    }
                    ISA::Calculate(op, addr1, addr2, target_addr) => {
                        let a = self.regs.update(Signal::Value(RegCommand::Read(addr1))).unpack().unwrap();
                        let b = self.regs.update(Signal::Value(RegCommand::Read(addr2))).unpack().unwrap();
                        let temp = match op {
                            Caltulations::Add =>{
                                Signal::Value(AluCommand::Add(a, b))
                            }
                            Caltulations::Sub =>{
                                Signal::Value(AluCommand::Sub(a, b))
                            }
                        };
                        let wire = self.alu.update(temp).unpack().unwrap();
                        self.regs.update(Signal::Value(RegCommand::Write(target_addr, wire)));
                        return Signal::Ignore(())
                    }
                    ISA::Jump(op, addr1, addr2, pc_addr) => {
                        let a = self.regs.update(Signal::Value(RegCommand::Read(addr1))).unpack().unwrap();
                        let b = self.regs.update(Signal::Value(RegCommand::Read(addr2))).unpack().unwrap();
                        let collect = match op {
                            Compare::Bigger => {
                                Signal::Value(CondCommand::UnsignedBiger(a, b))
                            }
                            Compare::Equal => {
                                Signal::Value(CondCommand::Equal(a, b))
                            }
                            Compare::Smaller => {
                                Signal::Value(CondCommand::UnsignedSmaller(a, b))
                            }
                        };
                        let result = self.cond.update(collect).unpack().unwrap();
                        let pc_signal = match result.0 {
                            true => {
                                Signal::Value(PCCommand::Set(pc_addr))
                            }
                            false => {
                                Signal::Ignore(())
                            }
                        };
                        self.pc.update(pc_signal);
                        return Signal::Ignore(())
                    }
                    ISA::Output(addr) =>{
                        let wire = self.regs.update(Signal::Value(RegCommand::Read(addr))).unpack().unwrap();
                        return Signal::Value(Output::Out(wire));
                    }
                }
                
            }
            Signal::Ignore(_) => { return Signal::Ignore(())}
        }
    }
}