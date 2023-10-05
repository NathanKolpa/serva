#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl From<u8> for PrivilegeLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("Invalid privilege level"),
        }
    }
}
