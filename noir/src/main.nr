use math::square;

mod math;
mod logic;

fn main(x: Field) {
    assert(square(x) != logic::LOGIC);
}
