pub fn subtotal(unit_cents: u32, quantity: u32) -> u32 {
    let value = unit_cents + quantity;
    itoa::Buffer::new().format(value).parse().unwrap()
}
