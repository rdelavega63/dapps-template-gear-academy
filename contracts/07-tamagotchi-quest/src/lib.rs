#![no_std]

#[allow(unused_imports)]
use gstd::{exec, future, msg, prelude::*, ActorId, ReservationId};
use sharded_fungible_token_io::{FTokenAction, FTokenEvent, LogicAction};
use store_io::{AttributeId, StoreAction, StoreEvent};
use tamagotchi_quest_io::{Tamagotchi, TmgAction, TmgEvent, Weapon, Direction};

static mut TAMAGOTCHI_STATE: Option<Tamagotchi> = None;

const HUNGER_PER_BLOCK: u64 = 1;
const BOREDOM_PER_BLOCK: u64 = 2;
const ENERGY_PER_BLOCK: u64 = 2;
const FILL_PER_FEED: u64 = 1000;
const FILL_PER_ENTERTAINMENT: u64 = 1000;
const FILL_PER_SLEEP: u64 = 1000;

async fn approve_tokens(account: &ActorId, amount: u128) -> Result<(), ()> {
    let tamagotchi = unsafe {
        TAMAGOTCHI_STATE
            .as_mut()
            .expect("The contract is not initialized")
    };

    let response = msg::send_for_reply_as::<_, FTokenEvent>(
        tamagotchi.ft_contract_id,
        FTokenAction::Message {
            transaction_id: tamagotchi.transaction_id,
            payload: LogicAction::Approve {
                approved_account: *account,
                amount,
            },
        },
        0,
        0,
    )
        .expect("Error in sending a message `FTokenAction::Message`")
        .await;

    match response {
        Ok(FTokenEvent::Ok) => Ok(()),
        _ => Err(()),
    }
}

async fn buy_attribute(store_id: ActorId, attribute_id: AttributeId) -> Result<(), String> {
    let future = msg::send_for_reply_as::<_, StoreEvent>(
        store_id,
        StoreAction::BuyAttribute { attribute_id },
        0,
        0,
    )
        .map_err(|_| "Failed to send message to store contract")?;

    let response = future
        .await
        .map_err(|_| "Failed to receive response from store contract")?;

    match response {
        StoreEvent::AttributeSold { success } if success => Ok(()),
        _ => Err("Attribute purchase failed".to_string()),
    }
}

async fn reserve_gas(tamagotchi: &mut Tamagotchi, amount: u64, duration: u32) {
    match ReservationId::reserve(amount, duration) {
        Ok(reservation_id) => {
            tamagotchi.reservations.push(reservation_id);
            msg::reply(&TmgEvent::GasReserved, 0).expect("Failed to send reply");
        },
        Err(_) => {
            msg::reply(&TmgEvent::MakeReservation, 0).expect("Failed to send reply");
        }
    }
}

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
        ft_contract_id: Default::default(),
        transaction_id: Default::default(),
        approve_transaction: None,
        reservations: vec![],
        agility: 0,
        intelligence: 0,
        luck: 0,
        weapon: Weapon::None,
        x: 0,
        y: 0,
    };

    save_tamagotchi_state(tamagotchi);
}

#[gstd::async_main]
async fn main() {
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
        TmgAction::SetFTokenContract(ft_contract_id) => {
            tamagotchi.ft_contract_id = ft_contract_id;
            msg::reply(&TmgEvent::FTokenContractSet, 0).expect("Failed to send reply");
        }
        TmgAction::ApproveTokens { account, amount } => {
            match approve_tokens(&account, amount).await {
                Ok(()) => {
                    msg::reply(&TmgEvent::TokensApproved { account, amount }, 0)
                        .expect("Failed to send reply");
                }
                Err(()) => {
                    msg::reply(&TmgEvent::ApprovalError, 0).expect("Failed to send reply");
                }
            }
        }
        TmgAction::BuyAttribute {
            store_id,
            attribute_id,
        } => match buy_attribute(store_id, attribute_id).await {
            Ok(()) => {
                msg::reply(&TmgEvent::AttributeBought(attribute_id), 0)
                    .expect("Failed to send reply");
            }
            Err(_error) => {
                msg::reply(&TmgEvent::ErrorDuringPurchase, 0).expect("Failed to send reply");
            }
        },
        TmgAction::UpgradeAttribute { attribute_id, upgrade_level } => {
            match upgrade_attribute(tamagotchi, attribute_id as u64, u64::from(upgrade_level)).await {
                Ok(()) => {
                    msg::reply(&TmgEvent::CompletePrevPurchase, 0)
                        .expect("Failed to send reply");
                }
                Err(_error) => {
                    msg::reply(&TmgEvent::ErrorDuringPurchase, 0).expect("Failed to send reply");
                }
            }
        },
        TmgAction::Move(direction) => {
            move_tamagotchi(tamagotchi, &direction);
            msg::reply(&TmgEvent::Moved(direction), 0).expect("Failed to send reply");
        },
        TmgAction::ChooseWeapon(attribute_id) => {
            match choose_weapon(tamagotchi, attribute_id) {
                Ok(()) => {
                    msg::reply(&TmgEvent::WeaponChosen, 0).expect("Failed to send reply");
                }
                Err(_error) => {
                    msg::reply(&TmgEvent::ErrorDuringPurchase, 0).expect("Failed to send reply");
                }
            }
        },
        TmgAction::CheckState => {
            check_state(tamagotchi).await;
        },
        TmgAction::ReserveGas {
            reservation_amount,
            duration,
        } => {
            reserve_gas(tamagotchi, reservation_amount, duration).await;
        },
    }

    save_tamagotchi_state(tamagotchi.clone());
}

async fn upgrade_attribute(tamagotchi: &mut Tamagotchi, attribute_id: u64, upgrade_level: u64) -> Result<(), &'static str> {
    let upgrade_cost = calculate_upgrade_cost(upgrade_level);

    if can_afford_upgrade(tamagotchi, upgrade_cost) {
        let _attribute = match attribute_id {
            0 => tamagotchi.agility += upgrade_level as u64,
            1 => tamagotchi.intelligence += upgrade_level as u64,
            2 => tamagotchi.luck += upgrade_level as u64,
            _ => return Err("Invalid attribute ID"),
        };
        deduct_upgrade_cost(tamagotchi, upgrade_cost);
        Ok(())
    } else {
        Err("Insufficient resources for upgrade")
    }
}

fn calculate_upgrade_cost(upgrade_level: u64) -> u64 {
    100 * upgrade_level as u64
}

fn can_afford_upgrade(tamagotchi: &Tamagotchi, cost: u64) -> bool {
    tamagotchi.fed >= cost
}

fn deduct_upgrade_cost(tamagotchi: &mut Tamagotchi, cost: u64) {
    tamagotchi.fed = tamagotchi.fed.saturating_sub(cost);
}

fn choose_weapon(tamagotchi: &mut Tamagotchi, attribute_id: u32) -> Result<(), &'static str> {

    let weapon = match attribute_id {
        0 => Weapon::None,
        1 => Weapon::Sword,
        2 => Weapon::Shield,
        3 => Weapon::Bow,
        4 => Weapon::Axe,
        5 => Weapon::Staff,
        6 => Weapon::Dagger,
        7 => Weapon::Spear,
        8 => Weapon::Mace,
        9 => Weapon::Crossbow,
        _ => return Err("Invalid weapon choice"),
    };

    tamagotchi.weapon = weapon;
    Ok(())
}

fn save_tamagotchi_state(tamagotchi: Tamagotchi) {
    unsafe {
        TAMAGOTCHI_STATE = Some(tamagotchi);
    }
}

fn move_tamagotchi(tamagotchi: &mut Tamagotchi, direction: &Direction) {
    match direction {
        Direction::Up => tamagotchi.y += 1,
        Direction::Down => tamagotchi.y -= 1,
        Direction::Left => tamagotchi.x -= 1,
        Direction::Right => tamagotchi.x += 1,
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

async fn check_state(tamagotchi: &mut Tamagotchi) {

    let food_threshold: u64 = 100;
    let sleep_threshold: u64 = 100;
    let entertainment_threshold: u64 = 100;

    if tamagotchi.fed < food_threshold {
        msg::reply(&TmgEvent::FeedMe, 0).expect("Failed to send reply");
    }

    if tamagotchi.slept < sleep_threshold {
        msg::reply(&TmgEvent::WantToSleep, 0).expect("Failed to send reply");
    }

    if tamagotchi.entertained < entertainment_threshold {
        msg::reply(&TmgEvent::PlayWithMe, 0).expect("Failed to send reply");
    }
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
