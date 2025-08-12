use crate::types::{BuildingKind, ResourceKind};

// Единая цветовая палитра

pub fn building_color(kind: BuildingKind) -> [u8; 4] {
    use BuildingKind::*;
    match kind {
        Lumberjack => [140, 90, 40, 255],
        House => [180, 180, 180, 255],
        Warehouse => [150, 120, 80, 255],
        Forester => [90, 140, 90, 255],
        StoneQuarry => [120, 120, 120, 255],
        ClayPit => [150, 90, 70, 255],
        Kiln => [160, 60, 40, 255],
        WheatField => [200, 180, 80, 255],
        Mill => [210, 210, 180, 255],
        Bakery => [200, 160, 120, 255],
        Fishery => [100, 140, 200, 255],
        IronMine => [90, 90, 110, 255],
        Smelter => [190, 190, 210, 255],
    }
}

pub fn resource_color(kind: ResourceKind) -> [u8; 4] {
    use ResourceKind::*;
    match kind {
        Wood => [110, 70, 30, 255],
        Stone => [120, 120, 120, 255],
        Clay => [150, 90, 70, 255],
        Bricks => [180, 120, 90, 255],
        Wheat => [200, 180, 80, 255],
        Flour => [210, 210, 180, 255],
        Bread => [200, 160, 120, 255],
        Fish => [100, 140, 200, 255],
        Gold => [220, 180, 60, 255],
        IronOre => [90, 90, 110, 255],
        IronIngot => [190, 190, 210, 255],
    }
}


