pub struct Sphere {
    val: usize
}

pub trait Shape {}

impl Sphere {
    pub fn new() -> Sphere {
        static mut COUNT: usize = 0;
        let out;
        unsafe {
            COUNT += 1;
            out = COUNT;
        };
        Sphere {
            val: out
        }
    }
}
