
fn main() {
    let fizz = (1..=3).map(|x| if x % 3 == 0 { Some("fizz") } else { None }).cycle();
    let buzz = (1..=5).map(|x| if x % 5 == 0 { Some("buzz") } else { None }).cycle();
    let iter = fizz.zip(buzz).cycle();

    for pair in iter
        .take(100)
        .enumerate()
        .filter(|(_,(f,b))| f.is_some() && b.is_some() ) {
        println!("{:?}", pair);
    }

}