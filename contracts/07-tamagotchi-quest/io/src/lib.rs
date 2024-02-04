#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{prelude::*, ActorId, ReservationId};
use scale_info::TypeInfo;
use store_io::{AttributeId, TransactionId};

#[derive(Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Weapon {
    None,
    Sword,
    Shield,
    Bow,
    Staff,
    Dagger,
    Axe,
    Hammer,
    Spear,
    Mace,
    Crossbow,
}

impl Default for Weapon {
    fn default() -> Self {
        Weapon::None
    }
}

#[derive(Default, Encode, Decode, TypeInfo, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Tamagotchi {
    pub name: String,
    pub date_of_birth: u64,
    pub owner: ActorId,
    pub fed: u64,
    pub fed_block: u64,
    pub entertained: u64,
    pub entertained_block: u64,
    pub slept: u64,
    pub slept_block: u64,
    pub approved_account: Option<ActorId>,
    pub ft_contract_id: ActorId,
    pub transaction_id: u64,
    pub approve_transaction: Option<(TransactionId, ActorId, u128)>,
    pub reservations: Vec<ReservationId>,
    pub agility: u64,
    pub intelligence: u64,
    pub luck: u64,
    pub weapon: Weapon,
    pub x: u64,
    pub y: u64,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum TmgAction {
    Name,
    Age,
    Feed,
    Entertain,
    Sleep,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    SetFTokenContract(ActorId),
    ApproveTokens {
        account: ActorId,
        amount: u128,
    },
    BuyAttribute {
        store_id: ActorId,
        attribute_id: AttributeId,
    },
    ChooseWeapon(AttributeId),
    UpgradeAttribute { attribute_id: u64, upgrade_level: u32 },
    Move(Direction),
    CheckState,
    ReserveGas {
        reservation_amount: u64,
        duration: u32,
    },
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum TmgEvent {
    Name(String),
    Age(u64),
    Fed,
    Entertained,
    Slept,
    Transferred(ActorId),
    Approved(ActorId),
    ApprovalRevoked,
    FTokenContractSet,
    TokensApproved { account: ActorId, amount: u128 },
    ApprovalError,
    AttributeBought(AttributeId),
    CompletePrevPurchase,
    ErrorDuringPurchase,
    FeedMe,
    PlayWithMe,
    WantToSleep,
    MakeReservation,
    GasReserved,
    WeaponChosen,
    AttributeUpgraded(AttributeId),
    Moved(Direction),
    ErrorDuringMove,
    CheckState(Tamagotchi),
    StateChecked(Tamagotchi),
    GasReservationError,
    GasReservationComplete,
    GasReservationCancelled,
    GasReservationExpired,
    GasReservationErrorDuringCancellation,
    GasReservationErrorDuringExpiration,
}

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = In<String>;
    type Handle = InOut<TmgAction, TmgEvent>;
    type State = Out<Tamagotchi>;
    type Reply = ();
    type Others = ();
    type Signal = ();
}
