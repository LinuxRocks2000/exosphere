// global structures about fabbers

#[derive(PartialEq)]
pub struct FabLevels {
    pub missiles : u8,
    pub ships : u8,
    pub econ : u8,
    pub defense : u8,
    pub buildings : u8
}


impl FabLevels {
    pub fn default() -> Self {
        Self {
            missiles : 0,
            ships : 0,
            econ : 0,
            defense : 0,
            buildings : 0
        }
    }

    pub fn with_missiles(mut self, lev : u8) -> Self {
        self.missiles = lev;
        self
    }

    pub fn with_ships(mut self, lev : u8) -> Self {
        self.ships = lev;
        self
    }

    pub fn with_econ(mut self, lev : u8) -> Self {
        self.econ = lev;
        self
    }

    pub fn with_defense(mut self, lev : u8) -> Self {
        self.defense = lev;
        self
    }

    pub fn with_buildings(mut self, lev : u8) -> Self {
        self.buildings = lev;
        self
    }

    pub fn missiles(lev : u8) -> Self {
        Self::default().with_missiles(lev)
    }

    pub fn ships(lev : u8) -> Self {
        Self::default().with_ships(lev)
    }

    pub fn econ(lev : u8) -> Self {
        Self::default().with_econ(lev)
    }

    pub fn defense(lev : u8) -> Self {
        Self::default().with_defense(lev)
    }

    pub fn buildings(lev : u8) -> Self {
        Self::default().with_buildings(lev)
    }
}


impl std::cmp::PartialOrd for FabLevels {
    fn partial_cmp(&self, other : &FabLevels) -> Option<std::cmp::Ordering> {
        // a FabLevels is greater than another FabLevels if every level is greater, and equal if they're all the same; otherwise, it is less.
        if *self == *other {
            return Some(std::cmp::Ordering::Equal);
        }
        if self.missiles < other.missiles || self.ships < other.ships || self.econ < other.econ || self.defense < other.defense || self.buildings < other.buildings {
            return Some(std::cmp::Ordering::Less);
        }
        Some(std::cmp::Ordering::Greater)
    }
}