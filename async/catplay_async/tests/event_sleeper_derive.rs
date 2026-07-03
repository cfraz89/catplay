use std::time::{Duration, Instant};

use catplay_async::{EventSleeper, EventToken};

struct ManualSleeper;

impl EventSleeper for ManualSleeper {
    async fn sleep(&mut self) -> Option<EventToken> {
        Some(EventToken(99))
    }
}

#[derive(EventSleeper)]
#[event_sleeper(crate = "catplay_async")]
#[sleep(catplay_async::deadline(self.deadline))]
#[sleep(catplay_async::deadline_after(self.delay))]
#[sleep_fut(async {})]
struct DerivedSleeper {
    #[sleep]
    child: Option<ManualSleeper>,

    #[slot(async { Some(10) })]
    optional_slot: Option<u32>,

    #[slot_value(async { 20 })]
    value_slot: Option<u32>,

    #[slot_map(async { Err::<(), _>(30) }, Result::err)]
    mapped_slot: Option<u32>,

    deadline: Instant,
    delay: Duration,
}

#[tokio::test]
async fn derived_event_sleeper_compiles_and_wakes() {
    // Revisit this if branch randomization gets added by default
    let mut sleeper = DerivedSleeper {
        child: None,
        optional_slot: None,
        value_slot: None,
        mapped_slot: None,
        deadline: Instant::now(),
        delay: Duration::from_millis(1),
    };

    assert_eq!(sleeper.sleep().await, Some(EventToken(1)));
    assert_eq!(sleeper.optional_slot, Some(10));

    assert_eq!(sleeper.sleep().await, Some(EventToken(1)));
    assert_eq!(sleeper.value_slot, Some(20));

    assert_eq!(sleeper.sleep().await, Some(EventToken(1)));
    assert_eq!(sleeper.mapped_slot, Some(30));

    assert_eq!(sleeper.sleep().await, Some(EventToken(1)));
}
