pub mod mapping;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleOverlay,
    TogglePttMode,
    PttHold,
    PttRelease,
    NavUp,
    NavDown,
    VolUp,
    VolDown,
    Mute,
    JumpTop,
    JumpBottom,
    AccordionOpen,
    AccordionClose,
}
