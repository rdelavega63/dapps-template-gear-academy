#![no_std]

#[allow(unused_imports)]
use gstd::{exec, msg, prelude::*};
use tamagotchi_nft_io::{Tamagotchi, TmgAction, TmgEvent};

static mut TAMAGOTCHI_STATE: Option<Tamagotchi> = None;

const HUNGER_PER_BLOCK: u64 = 1;
const BOREDOM_PER_BLOCK: u64 = 2;
const ENERGY_PER_BLOCK: u64 = 2;
const FILL_PER_FEED: u64 = 1000;
const FILL_PER_ENTERTAINMENT: u64 = 1000;
const FILL_PER_SLEEP: u64 = 1000;

#[no_mangle]
extern fn init() {
    let name: String = msg::load().expect("Invalid initial name");

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
        approved_account: None,
    };

    save_tamagotchi_state(tamagotchi);
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

    if msg::source() != tamagotchi.owner && Some(msg::source()) != tamagotchi.approved_account {
        panic!("Unauthorized");
    }

    update_levels(tamagotchi, current_block_height);

    if tamagotchi.fed == 0 && tamagotchi.entertained == 0 && tamagotchi.slept == 0 {
        panic!("I'm afraid your tamagotchi has died");
    }

    match action {
        TmgAction::Name => {
            msg::reply(&tamagotchi.name, 0).expect("Failed to send reply");
        }
        TmgAction::Age => {
            let current_timestamp = exec::block_timestamp();
            let age_in_milliseconds = current_timestamp - tamagotchi.date_of_birth;
            msg::reply(&age_in_milliseconds, 0).expect("Failed to send reply");
        }
        TmgAction::Feed => {
            tamagotchi.fed_block = current_block_height;
            tamagotchi.fed = tamagotchi.fed.saturating_add(FILL_PER_FEED);
            msg::reply(&TmgEvent::Fed, 0).expect("Failed to send reply");
        }
        TmgAction::Entertain => {
            tamagotchi.entertained_block = current_block_height;
            tamagotchi.entertained = tamagotchi
                .entertained
                .saturating_add(FILL_PER_ENTERTAINMENT);
            msg::reply(&TmgEvent::Entertained, 0).expect("Failed to send reply");
        }
        TmgAction::Sleep => {
            tamagotchi.slept_block = current_block_height;
            tamagotchi.slept = tamagotchi.slept.saturating_add(FILL_PER_SLEEP);
            msg::reply(&TmgEvent::Slept, 0).expect("Failed to send reply");
        }
        TmgAction::Transfer(new_owner) => {
            tamagotchi.owner = new_owner;
            msg::reply(&TmgEvent::Transferred(new_owner), 0).expect("Failed to send reply");
        }
        TmgAction::Approve(account) => {
            tamagotchi.approved_account = Some(account);
            msg::reply(&TmgEvent::Approved(account), 0).expect("Failed to send reply");
        }
        TmgAction::RevokeApproval => {
            tamagotchi.approved_account = None;
            msg::reply(&TmgEvent::ApprovalRevoked, 0).expect("Failed to send reply");
        }
    }

    save_tamagotchi_state(tamagotchi.clone());
}

fn save_tamagotchi_state(tamagotchi: Tamagotchi) {
    unsafe {
        TAMAGOTCHI_STATE = Some(tamagotchi);
    }
}

fn update_levels(tamagotchi: &mut Tamagotchi, current_block_height: u64) {
    let blocks_since_last_fed = current_block_height - tamagotchi.fed_block;
    let blocks_since_last_entertained = current_block_height - tamagotchi.entertained_block;
    let blocks_since_last_slept = current_block_height - tamagotchi.slept_block;

    let hunger = blocks_since_last_fed * HUNGER_PER_BLOCK;
    let boredom = blocks_since_last_entertained * BOREDOM_PER_BLOCK;
    let tiredness = blocks_since_last_slept * ENERGY_PER_BLOCK;

    tamagotchi.fed = tamagotchi.fed.saturating_sub(hunger);
    tamagotchi.entertained = tamagotchi.entertained.saturating_sub(boredom);
    tamagotchi.slept = tamagotchi.slept.saturating_sub(tiredness);
}

#[no_mangle]
extern fn state() {
    let tamagotchi_state = unsafe {
        TAMAGOTCHI_STATE
            .as_mut()
            .expect("The contract is not initialized")
    };

    msg::reply(tamagotchi_state, 0).expect("Failed to reply with state");
}
