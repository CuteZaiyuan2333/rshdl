pub trait Module {
    type Input;
    type Output;
    fn update(&mut self, input: Self::Input) -> Self::Output;
}

enum Signal<T1, T2> {
    Result(T1),
    Ignore(T2),
}
enum Op {
    Read(u8),
    Write(u8, u8),
}
struct MyModule {
    regs: [u8; 32],
}
impl Module for MyModule {
    type Input = Op;
    type Output = Signal<u8, ()>;
    fn update(&mut self, input: Self::Input) -> Self::Output {
        match input {
            Op::Read(addr) => {
                return Signal::Result(self.regs[addr as usize]);
            }
            Op::Write(addr, content) => {
                self.regs[addr as usize] = content;
                return Signal::Ignore(());
            }
        }
    }
}
