#![no_std]

#[allow(unused_imports)]
use gstd::prelude::*;
use gstd::{exec, msg, debug};

#[derive(Encode, Decode)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
}

#[derive(Encode, Decode)]
pub enum TmgAction {
    Name,
    Age,
}

static mut TAMAGOTCHI_STATE: Option<Tamagotchi> = None;

#[no_mangle]
extern fn init() {
    let name_bytes = msg::load_bytes().expect("Invalid initial name");
    let name = String::from_utf8(name_bytes).expect("Invalid UTF-8");

    let date_of_birth = exec::block_timestamp();

    let tamagotchi = Tamagotchi {
        name: name.clone(),
        date_of_birth,
    };

    save_tamagotchi_state(tamagotchi);

    debug!("Tamagotchi initialized with name: {:?}, date of birth: {:?}", name, date_of_birth);
}

fn save_tamagotchi_state(tamagotchi: Tamagotchi) {
    unsafe {
        TAMAGOTCHI_STATE = Some(tamagotchi);
    }
}

#[no_mangle]
extern fn handle() {
    let action: TmgAction = msg::load().expect("Failed to decode TmgAction");

    let tamagotchi = unsafe {
        TAMAGOTCHI_STATE
            .as_ref()
            .expect("The contract is not initialized")
    };

    match action {
        TmgAction::Name => {
            msg::reply(&tamagotchi.name, 0).expect("Failed to send reply");
        },
        TmgAction::Age => {
            let current_timestamp = exec::block_timestamp();
            let age_in_milliseconds = current_timestamp - tamagotchi.date_of_birth;
            msg::reply(&age_in_milliseconds, 0).expect("Failed to send reply");
        },
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
