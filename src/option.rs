#![allow(dead_code)]
pub fn both<T1, T2>(opt1: Option<T1>, opt2: Option<T2>) -> Option<(T1, T2)> {
    opt1.and_then(|val1| opt2.map(|val2| (val1, val2)))
}
