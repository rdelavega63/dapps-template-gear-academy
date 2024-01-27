#![no_std]

#[allow(unused_imports)]
use gstd::prelude::*;
use gstd::{exec, msg, debug};

#[derive(Encode, Decode, Clone)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
    pub last_fed_block: u64,
    pub last_entertained_block: u64,
    pub last_slept_block: u64,
    pub fed_level: u64,
    pub entertained_level: u64,
    pub slept_level: u64,
}

#[derive(Encode, Decode)]
pub enum TmgAction {
    Name,
    Age,
    Feed,
    Entertain,
    Sleep,
}

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
        name: name.clone(),
        date_of_birth,
        last_fed_block: 0,
        last_entertained_block: 0,
        last_slept_block: 0,
        fed_level: 1000,
        entertained_level: 1000,
        slept_level: 1000,
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
            tamagotchi.last_fed_block = current_block_height;
            tamagotchi.fed_level = tamagotchi.fed_level.saturating_add(FILL_PER_FEED);
        },
        TmgAction::Entertain => {
            tamagotchi.last_entertained_block = current_block_height;
            tamagotchi.entertained_level = tamagotchi.entertained_level.saturating_add(FILL_PER_ENTERTAINMENT);
        },
        TmgAction::Sleep => {
            tamagotchi.last_slept_block = current_block_height;
            tamagotchi.slept_level = tamagotchi.slept_level.saturating_add(FILL_PER_SLEEP);
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
    let blocks_since_last_fed = current_block_height - tamagotchi.last_fed_block;
    let blocks_since_last_entertained = current_block_height - tamagotchi.last_entertained_block;
    let blocks_since_last_slept = current_block_height - tamagotchi.last_slept_block;

    let hunger = blocks_since_last_fed * HUNGER_PER_BLOCK;
    let boredom = blocks_since_last_entertained * BOREDOM_PER_BLOCK;
    let energy = blocks_since_last_slept * ENERGY_PER_BLOCK;

    tamagotchi.fed_level = tamagotchi.fed_level.saturating_sub(hunger);
    tamagotchi.entertained_level = tamagotchi.entertained_level.saturating_sub(boredom);
    tamagotchi.slept_level = tamagotchi.slept_level.saturating_sub(energy);

    tamagotchi.last_fed_block = current_block_height;
    tamagotchi.last_entertained_block = current_block_height;
    tamagotchi.last_slept_block = current_block_height;

    if tamagotchi.fed_level == 0 {
        msg::reply("Your tamagotchi is hungry!", 0).expect("Failed to send reply");
    }

    if tamagotchi.entertained_level == 0 {
        msg::reply("Your tamagotchi is bored!", 0).expect("Failed to send reply");
    }

    if tamagotchi.slept_level == 0 {
        msg::reply("Your tamagotchi is tired!", 0).expect("Failed to send reply");
    }

    if tamagotchi.fed_level == 0 || tamagotchi.entertained_level == 0 || tamagotchi.slept_level == 0 {
        msg::reply("Your tamagotchi is dead!", 0).expect("Failed to send reply");
    }
}

#[no_mangle]
extern fn state() {
    let tamagotchi = unsafe {
        TAMAGOTCHI_STATE
            .as_ref()
            .expect("The contract is not initialized")
    };

    msg::reply(tamagotchi, 0).expect("Failed to reply with state");
}
