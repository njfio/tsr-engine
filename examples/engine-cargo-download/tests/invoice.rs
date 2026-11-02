#[test]
fn subtotal() { assert_eq!(invoice::pricing::subtotal(100, 3), 300); }
#[test]
fn discount() { assert_eq!(invoice::discount::total(300, 30), 270); }
#[test]
fn floor() { assert_eq!(invoice::discount::total(30, 300), 0); }
