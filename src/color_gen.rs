use rand::Rng;

pub fn get_color() -> (u8, u8, u8) {
    let mut rng = rand::thread_rng();
    let r = rng.gen();
    let g = rng.gen();
    let b = rng.gen();
    (r, g, b)
}
