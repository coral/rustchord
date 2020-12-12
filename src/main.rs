mod bindings;
#[link(name = "colorchord")] extern {} 
fn main() {
    println!("Hello, world!");
    unsafe {
        let c = bindings::notefinder::CreateNoteFinder(44100);
    }
}
