#![no_std]

#[allow(unused_imports)]
use gstd::prelude::*;
use gstd::{exec, msg, debug};
use tamagotchi_interaction_io::{Tamagotchi, TmgAction};

static mut TAMAGOTCHI_STATE: Option<Tamagotchi> = None;

const HUNGER_PER_BLOCK: u64 = 1;
const BOREDOM_PER_BLOCK: u64 = 2;
const ENERGY_PER_BLOCK: u64 = 2;
const FILL_PER_FEED: u64 = 1000;
const FILL_PER_ENTERTAINMENT: u64 = 1000;
const FILL_PER_SLEEP: u64 = 1000;

#[no_mangle]
extern fn init() {
    let name_bytes = msg::load_bytes().expect("Invalid initial name");
    let name = String::from_utf8(name_bytes).expect("Invalid UTF-8");

    let date_of_birth = exec::block_timestamp();

    let tamagotchi = Tamagotchi {
        name,
        date_of_birth,
        owner: msg::source(),
        fed: 1000,
        fed_block: 0,
        entertained: 1000,
        entertained_block: 0,
        slept: 1000,
        slept_block: 0,
    };

    save_tamagotchi_state(tamagotchi);

    debug!("Tamagotchi initialized with name: {:?}, date of birth: {:?}", name, date_of_birth);
}

#[no_mangle]
extern fn handle() {
    let action: TmgAction = msg::load().expect("Failed to decode TmgAction");

    let current_block_height = exec::block_height() as u64;
    let tamagotchi = unsafe {
        TAMAGOTCHI_STATE
            .as_mut()
            .expect("The contract is not initialized")
    };

    update_levels(tamagotchi, current_block_height);

    match action {
        TmgAction::Name => {
            msg::reply(&tamagotchi.name, 0).expect("Failed to send reply");
        },
        TmgAction::Age => {
            let current_timestamp = exec::block_timestamp();
            let age_in_milliseconds = current_timestamp - tamagotchi.date_of_birth;
            msg::reply(&age_in_milliseconds, 0).expect("Failed to send reply");
        },
        TmgAction::Feed => {
            tamagotchi.fed_block = current_block_height;
            tamagotchi.fed = tamagotchi.fed.saturating_add(FILL_PER_FEED);
        },
        TmgAction::Entertain => {
            tamagotchi.entertained_block = current_block_height;
            tamagotchi.entertained = tamagotchi.entertained.saturating_add(FILL_PER_ENTERTAINMENT);
        },
        TmgAction::Sleep => {
            tamagotchi.slept_block = current_block_height;
            tamagotchi.slept = tamagotchi.slept.saturating_add(FILL_PER_SLEEP);
        },
    }

    save_tamagotchi_state(tamagotchi.clone());
}

fn save_tamagotchi_state(tamagotchi: Tamagotchi) {
    unsafe {
        TAMAGOTCHI_STATE = Some(tamagotchi);
    }
}

fn update_levels(tamagotchi: &mut Tamagotchi, current_block_height: u64) {
    // Calculate the number of blocks since the last time the Tamagotchi was fed, entertained, and slept
    let blocks_since_last_fed = current_block_height - tamagotchi.fed_block;
    let blocks_since_last_entertained = current_block_height - tamagotchi.entertained_block;
    let blocks_since_last_slept = current_block_height - tamagotchi.slept_block;

    // Calculate how much hunger, boredom, and tiredness the Tamagotchi has accumulated since the last time it was fed,
    // entertained, and slept
    let hunger = blocks_since_last_fed * HUNGER_PER_BLOCK;
    let boredom = blocks_since_last_entertained * BOREDOM_PER_BLOCK;
    let tiredness = blocks_since_last_slept * ENERGY_PER_BLOCK;

    // Subtract the accumulated hunger, boredom, and tiredness from the Tamagotchi's current fed, entertained,
    // and slept levels
    tamagotchi.fed = tamagotchi.fed.saturating_sub(hunger);
    tamagotchi.entertained = tamagotchi.entertained.saturating_sub(boredom);
    tamagotchi.slept = tamagotchi.slept.saturating_sub(tiredness);

    // Check if the Tamagotchi is hungry, bored, or tired and send a message to the Tamagotchi's owner in each case
    if tamagotchi.fed == 0 {
        msg::reply("Your tamagotchi is hungry!", 0).expect("Failed to send reply");
    }

    if tamagotchi.entertained == 0 {
        msg::reply("Your tamagotchi is bored!", 0).expect("Failed to send reply");
    }

    if tamagotchi.slept == 0 {
        msg::reply("Your tamagotchi is tired!", 0).expect("Failed to send reply");
    }

    // Check if the Tamagotchi is in a critical state (i.e., it is hungry, bored, and tired at the same time)
    // and send a message to the Tamagotchi's owner
    if tamagotchi.fed == 0 || tamagotchi.entertained == 0 || tamagotchi.slept == 0 {
        msg::reply("Your tamagotchi is in a critical state!", 0).expect("Failed to send reply");
    }

    // Check if the Tamagotchi is dead (i.e., it is hungry, bored, and tired at the same time) and send a message
    // to the Tamagotchi's owner
    if tamagotchi.fed == 0 && tamagotchi.entertained == 0 && tamagotchi.slept == 0 {
        msg::reply("Your tamagotchi is dead!", 0).expect("Failed to send reply");
    }
}

#[no_mangle]
extern fn state() {
    let tamagotchi = unsafe {
        TAMAGOTCHI_STATE
            .take()
            .expect("The contract is not initialized")
    };

    msg::reply(tamagotchi, 0).expect("Failed to reply with state");
}
