#[derive(Clone)]
pub struct LLWReg<Value, Time: Ord> {
    pub time: Time,
    pub value: Value,
}

impl <Value, Time: Ord> LLWReg<Value, Time> {
    pub fn set(&mut self, value: Value, time: Time) {
        if time > self.time {
            self.value = value
        }
    }
}

