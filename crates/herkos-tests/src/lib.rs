// Include generated modules from build.rs (in OUT_DIR)
include!(concat!(env!("OUT_DIR"), "/mod.rs"));

pub fn fibo_orig(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        let mut a: i32 = 0;
        let mut b: i32 = 1;
        for _ in 2..=n {
            let tmp = a.wrapping_add(b);
            a = b;
            b = tmp;
        }
        b
    }
}
