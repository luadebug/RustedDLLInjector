#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessSelectionMethod {
    ByProcessName,
    ByPID,
    ByPIDInput,
}
