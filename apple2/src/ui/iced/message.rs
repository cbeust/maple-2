use iced::window;
use crate::config_file::ConfigFile;
use crate::disk::disk_info::DiskInfo;
use crate::ui::iced::memory_view::MemoryType;
use crate::ui::iced::ui_iced::TabId;

#[derive(Clone, Debug)]
pub enum SpecialKeyMsg {
    AltLeft,
    AltRight,
}

/// Messages received from the CPU are of type ToUi, but we need to have our own
/// UI messages for iced. The structure below duplicates ToUi and adds other enum variants
/// to manage the UI.
#[derive(Clone, Debug)]
pub enum InternalUiMessage {
    Tick,
    Load,
    Reboot,
    Swap,
    OpenDebugger,
    // bool: true if is_hard_drive
    DiskInserted(bool, usize, Option<DiskInfo>),
    MainWindowOpened(window::Id),
    DebuggerWindowOpened(window::Id),
    TabSelected(TabId),
    TabClosed(TabId),
    /// Whenever the user picks a different directory for the Apple disks
    NewDirectorySelected(Option<String>),
    DisksDirectorySelected,
    /// Load drive (0 or 1) with the disk found at the path
    LoadDrive(usize, String),
    /// Load hard drive (0, 1) with the hard drive found at the path
    LoadHardDrive(usize, String),
    /// New filter typed on the Disks tab
    FilterUpdated(String),
    Init(ConfigFile),
    /// Clear the Filter text input
    ClearFilter,
    /// When the user selects a phase in the NibblesTab. Phase is 0..159
    PhaseSelected(u8),

    /// When the user starts the debugger
    StartDebugger,
    BreakpointWasHit(u16),
    /// Messages sent when the user presses a control button
    DebuggerPlay,
    DebuggerPause,
    DebuggerStep,
    EditBreakPoint(String),
    /// The address of the breakpoint to delete
    DebuggerDeleteBreakpoint(u16),
    DebuggerBreakpointValue(String),
    DebuggerAddBreakpoint(String),

    /// Registers from the DebuggerTab
    RegisterA(String),
    /// A key has been pressed that the Apple ][ can consume
    Key(u8),
    /// Special key interpreted by the emulator (e.g. Alt for joystick button)
    /// bool: true if pressed, false if released
    SpecialKey(SpecialKeyMsg, bool),

    /// Select the memory type (main/aux)
    DebuggerMemoryTypeSelected(MemoryType),
    /// New location to display in the memory view
    DebuggerMemoryLocationChanged(String),
    DebuggerMemoryLocationSubmitted,

    EmulatorSpeed(f32),
    WindowClosed(window::Id),
    DriveSelected(usize),

    // Selection in the Drives window
    ShowDrives,
    ShowHardDrives,
    // bool: is_hard_drive, usize: drive_number
    Eject(bool, usize),
    Exit,
}
