use gtest::{Program, System};
use gstd::ActorId;
use tamagotchi_nft_io::{Tamagotchi, TmgAction};

const FILL_PER_FEED: u64 = 1000;
const FILL_PER_ENTERTAINMENT: u64 = 1000;
const FILL_PER_SLEEP: u64 = 1000;

const NEW_OWNER: ActorId = ActorId::new([0x01; 32]);
const APPROVED_ACCOUNT: ActorId = ActorId::new([0x02; 32]);

#[test]
fn owning_test() {
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

    // Feed Test
    let feed_res = program.send(2, TmgAction::Feed);
    assert!(!feed_res.main_failed(), "Failed to feed the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.fed_block, initial_block_height);
    assert_eq!(tamagotchi_state.fed, 1000 + FILL_PER_FEED); // Assuming the initial value is 1000

    // Entertain Test
    let entertain_res = program.send(2, TmgAction::Entertain);
    assert!(!entertain_res.main_failed(), "Failed to entertain the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.entertained_block, initial_block_height);
    assert_eq!(tamagotchi_state.entertained, 1000 + FILL_PER_ENTERTAINMENT); // Assuming the initial value is 1000

    // Sleep Test
    let sleep_res = program.send(2, TmgAction::Sleep);
    assert!(!sleep_res.main_failed(), "Failed to put the tamagotchi to sleep");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.slept_block, initial_block_height);
    assert_eq!(tamagotchi_state.slept, 1000 + FILL_PER_SLEEP); // Assuming the initial value is 1000

    // Transfer Test
    let transfer_res = program.send(2, TmgAction::Transfer(NEW_OWNER));
    assert!(!transfer_res.main_failed(), "Failed to transfer the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    println!("Tamagotchi state owner: {:?}", tamagotchi_state.owner);
    assert_eq!(tamagotchi_state.owner, NEW_OWNER);

    // Approve Test
    let approve_res = program.send(2, TmgAction::Approve(APPROVED_ACCOUNT));
    assert!(!approve_res.main_failed(), "Failed to approve account for the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.approved_account, Some(APPROVED_ACCOUNT));

    // RevokeApproval Test
    let revoke_approval_res = program.send(2, TmgAction::RevokeApproval);
    assert!(!revoke_approval_res.main_failed(), "Failed to revoke approval for the tamagotchi");
    let tamagotchi_state: Tamagotchi = program.read_state(()).unwrap();
    assert_eq!(tamagotchi_state.approved_account, None);
}
