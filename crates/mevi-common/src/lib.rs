use std::{fmt, ops::Range, sync::mpsc};

use rangemap::RangeMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MemState {
    Resident,
    NotResident,
    Unmapped,
    Untracked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TraceeId(pub u64);

impl fmt::Display for TraceeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

#[cfg(feature = "nix")]
impl From<nix::unistd::Pid> for TraceeId {
    fn from(pid: nix::unistd::Pid) -> Self {
        Self(pid.as_raw() as _)
    }
}

#[cfg(feature = "nix")]
impl From<TraceeId> for nix::unistd::Pid {
    fn from(id: TraceeId) -> Self {
        Self::from_raw(id.0 as _)
    }
}

pub type MemMap = RangeMap<u64, MemState>;

#[derive(Debug, Serialize, Deserialize)]
pub enum MeviEvent {
    Snapshot(Vec<TraceeSnapshot>),
    TraceeEvent(TraceeId, TraceePayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceeSnapshot {
    pub tid: TraceeId,
    pub cmdline: Vec<String>,
    pub map: MemMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceePayload {
    Map {
        range: Range<u64>,
        state: MemState,
        _guard: MapGuard,
    },
    Connected {
        source: ConnectSource,
        uffd: u64,
    },
    Execve,
    PageIn {
        range: Range<u64>,
    },
    PageOut {
        range: Range<u64>,
    },
    Unmap {
        range: Range<u64>,
    },
    Remap {
        old_range: Range<u64>,
        new_range: Range<u64>,
        _guard: MapGuard,
    },
    Batch {
        batch: MemMap,
    },
    Start {
        cmdline: Vec<String>,
    },
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectSource {
    Uds,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapGuard {
    #[serde(skip)]
    pub _inner: Option<mpsc::Sender<()>>,
}

impl Clone for MapGuard {
    fn clone(&self) -> Self {
        Self { _inner: None }
    }
}