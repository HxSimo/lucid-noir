use math::square;

mod math;
mod logic;

fn main(x: Field) {
    assert(square(x) != logic::LOGIC);
    assert(unsquare(x) != logic::LOGIC);
}

fn unsquare(x: Field) -> Field {
    x / x 
}