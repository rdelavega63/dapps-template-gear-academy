#![no_std]

#[allow(unused_imports)]
use gstd::prelude::*;
use gstd::{exec, msg, debug};
use tamagotchi_io::{Tamagotchi, TmgAction};

static mut TAMAGOTCHI_STATE: Option<Tamagotchi> = None;

#[no_mangle]
extern fn init() {
    let name: String = msg::load().expect("Invalid initial name");

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
