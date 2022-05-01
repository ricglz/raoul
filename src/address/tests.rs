use super::*;

#[test]
fn valid_get_address() {
    let mut address_manager = AddressManager::new(0);
    let address = address_manager.get_address(&Types::INT);
    assert_eq!(address, Some(0));
}

#[test]
fn invalid_get_address() {
    let mut address_manager = AddressManager::new(0);
    for i in 0..250 {
        let address = address_manager.get_address(&Types::INT);
        assert_eq!(address, Some(i));
    }
    let address = address_manager.get_address(&Types::INT);
    assert_eq!(address, None);
}
