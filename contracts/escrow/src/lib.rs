#![no_std]

#[allow(unused_imports)]
use escrow_io::{EscrowAction, Escrow, EscrowState};
use gstd::{msg, prelude::*};

static mut ESCROW: Option<Escrow> = None;

// Define un trait con los métodos que quieres implementar
trait EscrowContract {
    fn deposit(&mut self);
    fn confirm_delivery(&mut self);
}

// Implementa el trait para `Escrow`
impl EscrowContract for Escrow {
    fn deposit(&mut self) {
        self.state = EscrowState::AwaitingDelivery;
    }
    fn confirm_delivery(&mut self) {
        self.state = EscrowState::Closed;
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: EscrowAction = msg::load()
        .expect("Unable to decode `EscrowAction`");
    let escrow: &mut Escrow = unsafe {
        ESCROW
            .as_mut()
            .expect("The contract is not initialized")
    };
    match action {
        EscrowAction::Deposit => escrow.deposit(),
        EscrowAction::ConfirmDelivery => escrow.confirm_delivery(),
    }
}

#[no_mangle]
extern "C" fn state() {
    let escrow = unsafe {
        ESCROW.get_or_insert(Default::default())
    };
    msg::reply(escrow, 0).expect("Failed to share state");
}