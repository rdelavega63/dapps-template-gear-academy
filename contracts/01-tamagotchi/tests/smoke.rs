use gtest::{System, Program};
use tamagotchi_io::{Tamagotchi};

#[test]
fn smoke_test() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);

    let name = "Tammy";
    let init_res = program.send(0, name.to_string());
    assert!(!init_res.main_failed(), "Failed to initialize the contract");

    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    println!("Tamagotchi state name: {:?}", tamagotchi_state.name);

    let cleaned_name = tamagotchi_state.name.trim_matches(char::from(0x14));
    assert_eq!(cleaned_name, name);

    let current_timestamp = system.block_timestamp();
    assert!(tamagotchi_state.date_of_birth <= current_timestamp);
    assert!(tamagotchi_state.date_of_birth > current_timestamp - 1000);

}
