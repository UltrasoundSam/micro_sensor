use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use microbit::{
    hal::gpiote::Gpiote,
    board::Buttons,
    pac::{self, interrupt}
};

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static MEASURINGSTATE: Mutex<RefCell<Measuring>> = Mutex::new(RefCell::new(Measuring {active: false}));

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

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    })
}

pub fn get_state() -> bool {
    free(|cs| {
        let state = MEASURINGSTATE.borrow(cs).borrow().active;
        state
    })
}
#[pac::interrupt]
fn GPIOTE() {
    free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            // Check to see if button has been pressed
            let a_pressed = gpiote.channel0().is_event_triggered();

            if a_pressed {
                let current_state = get_state();
                MEASURINGSTATE.borrow(cs).borrow_mut().active = !current_state;
            }
            // Reset events
            gpiote.channel0().reset_events();
        }
    })

}

/// Struct to hold whether to report data to console or not
pub struct Measuring{
    pub active: bool,
}
