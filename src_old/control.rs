use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use microbit::{
    hal::gpiote::Gpiote,
    board::Buttons,
    pac::{self, interrupt}
};

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static MEASURINGSTATE: Mutex<RefCell<Measuring>> = Mutex::new(RefCell::new(Measuring {active: true, num_aves: 8}));

/// Initialise buttons and enable interrupts
/// Taken from Discovery book from rust-embedded
pub(crate) fn init_buttons(board_gpiote: pac::GPIOTE, board_buttons: Buttons) {
    let gpiote = Gpiote::new(board_gpiote);

    // Linking button a to channel0. Trigger interrupt off a falling edge .
    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&board_buttons.button_a.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel0.reset_events();

    // Linking button b to channel1. Trigger interrupt off a falling edge .
    let channel1 = gpiote.channel1();
    channel1
        .input_pin(&board_buttons.button_b.degrade())
        .hi_to_lo()
        .enable_interrupt();
    channel1.reset_events();

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    })
}

pub fn get_meas_state() -> bool {
    free(|cs| {
        let state = MEASURINGSTATE.borrow(cs).borrow();
        state.active
    })
}

pub fn get_num_aves() -> u8 {
    free(|cs| {
        let num_aves = MEASURINGSTATE.borrow(cs).borrow().num_aves;
        num_aves
    })
}

#[pac::interrupt]
fn GPIOTE() {
    free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            // Check to see if button has been pressed
            let a_pressed = gpiote.channel0().is_event_triggered();
            let b_pressed = gpiote.channel1().is_event_triggered();

            if a_pressed {
                let current_state = get_meas_state();
                MEASURINGSTATE.borrow(cs).borrow_mut().active = !current_state;
            }

            if b_pressed {
                let num_aves = get_num_aves() + 1;
                MEASURINGSTATE.borrow(cs).borrow_mut().num_aves = num_aves;
            }
            // Reset events
            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();
        }
    })

}


pub struct Measuring {
    pub active: bool,
    pub num_aves: u8
}
