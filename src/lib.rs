pub enum Signal<T1, T2> {
    Value(T1),
    Ignore(T2),
}

impl<T1, T2> Signal<T1, T2> {
    #[allow(dead_code)]
    fn unpack(self) -> Option<T1>{
        match self {
            Self::Value(a) => Some(a),
            Self::Ignore(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Bit(pub bool);
impl Bit{
    pub fn new(input:bool) -> Self{
        Self(input)
    }
}
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Byte(pub u8);
impl Byte{
    pub fn new(input: u8) -> Self{
        Self(input)
    }
}
pub trait Module {
    type Input;
    type Output;
    fn update(&mut self, input: Self::Input) -> Self::Output;
}