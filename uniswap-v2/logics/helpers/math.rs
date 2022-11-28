use primitive_types::U256;

pub fn casted_mul(a: u128, b: u128) -> U256 {
    U256::from(a) * U256::from(b)
}
