use gtest::{Program, System};
use tamagotchi_interaction::Tamagotchi;
use tamagotchi_interaction::TmgAction;

#[test]
fn interaction_test() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);

    let name = "Tammy";
    let init_res = program.send(0, name.to_string());
    assert!(!init_res.main_failed(), "Failed to initialize the contract");

    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    println!("Tamagotchi state name: {:?}", tamagotchi_state.name);

    let cleaned_name = tamagotchi_state.name.trim_matches(char::from(0x14));
    println!("Cleaned: {:?} {:?}", cleaned_name, name);
    assert_eq!(cleaned_name, name);

    let current_timestamp = system.block_timestamp();
    assert!(tamagotchi_state.date_of_birth <= current_timestamp);
    assert!(tamagotchi_state.date_of_birth > current_timestamp - 1000);

    let initial_block_height = system.block_height() as u64;

    let feed_res = program.send(2, TmgAction::Feed);
    assert!(!feed_res.main_failed(), "Failed to feed the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.last_fed_block, initial_block_height);
    assert!(tamagotchi_state.fed_level > 1000);

    let entertain_res = program.send(2, TmgAction::Entertain);
    assert!(!entertain_res.main_failed(), "Failed to entertain the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.last_entertained_block, initial_block_height);
    assert!(tamagotchi_state.entertained_level > 1000);

    let sleep_res = program.send(2, TmgAction::Sleep);
    assert!(!sleep_res.main_failed(), "Failed to put the tamagotchi to sleep");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.last_slept_block, initial_block_height);
    assert!(tamagotchi_state.slept_level > 1000);

}