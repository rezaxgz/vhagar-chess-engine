#[derive(Clone)]
pub struct UciSpinOption{
    name: String,
    value: usize,
    default: usize,
    min: usize,
    max: usize
}
impl UciSpinOption{
    pub fn thread_option() -> UciSpinOption{
        UciSpinOption { name: String::from("Threads") ,value: 4, default: 4, min: 1, max: 128 }
    }
}
pub struct UciOptions{
    spin_options: Vec<UciSpinOption>
}
impl UciOptions {
    pub fn new() -> UciOptions{
        UciOptions { spin_options: vec![UciSpinOption::thread_option()] }
    }
    pub fn print(&self){
        for i in 0..self.spin_options.len(){
            let spin = self.spin_options[i].clone();
            println!("option name {} type spin default {} min {} max {}", spin.name, spin.default, spin.min, spin.max);
        }
    }
    pub fn set(&mut self, name: String, value: String){
        for i in 0..self.spin_options.len(){
            if self.spin_options[i].name == name{
                let v: usize = value.parse().unwrap_or(self.spin_options[i].default);
                self.spin_options[i].value = v;
                return;
            }
        }
    }
}

impl UciOptions{
    pub fn thread_cout(&self) -> usize{
        return self.spin_options.iter().find(|a| a.name == "Threads").unwrap().value
    }
}