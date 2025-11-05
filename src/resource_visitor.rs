use crate::types::{ResourceKind, Resources, WarehouseStore};

/// Trait для элементов, которые могут принимать посетителя ресурсов
/// Это позволяет унифицировать операции над ресурсами
pub trait ResourceVisitable {
    /// Принять посетителя и выполнить операцию над ресурсом указанного типа
    fn accept<T>(&self, visitor: &mut T, resource: ResourceKind) -> T::Output
    where
        T: ResourceVisitor;
    
    /// Принять посетителя для мутабельной операции
    fn accept_mut<T>(&mut self, visitor: &mut T, resource: ResourceKind) -> T::Output
    where
        T: ResourceVisitorMut;
}

/// Trait для посетителя ресурсов (immutable операции)
pub trait ResourceVisitor {
    type Output;
    
    fn visit_wood(&mut self, amount: i32) -> Self::Output;
    fn visit_gold(&mut self, amount: i32) -> Self::Output;
    fn visit_stone(&mut self, amount: i32) -> Self::Output;
    fn visit_clay(&mut self, amount: i32) -> Self::Output;
    fn visit_bricks(&mut self, amount: i32) -> Self::Output;
    fn visit_wheat(&mut self, amount: i32) -> Self::Output;
    fn visit_flour(&mut self, amount: i32) -> Self::Output;
    fn visit_bread(&mut self, amount: i32) -> Self::Output;
    fn visit_fish(&mut self, amount: i32) -> Self::Output;
    fn visit_iron_ore(&mut self, amount: i32) -> Self::Output;
    fn visit_iron_ingot(&mut self, amount: i32) -> Self::Output;
}

/// Trait для посетителя ресурсов (mutable операции)
pub trait ResourceVisitorMut {
    type Output;
    
    fn visit_wood_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_gold_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_stone_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_clay_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_bricks_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_wheat_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_flour_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_bread_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_fish_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_iron_ore_mut(&mut self, amount: &mut i32) -> Self::Output;
    fn visit_iron_ingot_mut(&mut self, amount: &mut i32) -> Self::Output;
}

// Реализация для Resources
impl ResourceVisitable for Resources {
    fn accept<T>(&self, visitor: &mut T, resource: ResourceKind) -> T::Output
    where
        T: ResourceVisitor,
    {
        match resource {
            ResourceKind::Wood => visitor.visit_wood(self.wood),
            ResourceKind::Gold => visitor.visit_gold(self.gold),
            ResourceKind::Stone => visitor.visit_stone(self.stone),
            ResourceKind::Clay => visitor.visit_clay(self.clay),
            ResourceKind::Bricks => visitor.visit_bricks(self.bricks),
            ResourceKind::Wheat => visitor.visit_wheat(self.wheat),
            ResourceKind::Flour => visitor.visit_flour(self.flour),
            ResourceKind::Bread => visitor.visit_bread(self.bread),
            ResourceKind::Fish => visitor.visit_fish(self.fish),
            ResourceKind::IronOre => visitor.visit_iron_ore(self.iron_ore),
            ResourceKind::IronIngot => visitor.visit_iron_ingot(self.iron_ingots),
        }
    }
    
    fn accept_mut<T>(&mut self, visitor: &mut T, resource: ResourceKind) -> T::Output
    where
        T: ResourceVisitorMut,
    {
        match resource {
            ResourceKind::Wood => visitor.visit_wood_mut(&mut self.wood),
            ResourceKind::Gold => visitor.visit_gold_mut(&mut self.gold),
            ResourceKind::Stone => visitor.visit_stone_mut(&mut self.stone),
            ResourceKind::Clay => visitor.visit_clay_mut(&mut self.clay),
            ResourceKind::Bricks => visitor.visit_bricks_mut(&mut self.bricks),
            ResourceKind::Wheat => visitor.visit_wheat_mut(&mut self.wheat),
            ResourceKind::Flour => visitor.visit_flour_mut(&mut self.flour),
            ResourceKind::Bread => visitor.visit_bread_mut(&mut self.bread),
            ResourceKind::Fish => visitor.visit_fish_mut(&mut self.fish),
            ResourceKind::IronOre => visitor.visit_iron_ore_mut(&mut self.iron_ore),
            ResourceKind::IronIngot => visitor.visit_iron_ingot_mut(&mut self.iron_ingots),
        }
    }
}

// Реализация для WarehouseStore
impl ResourceVisitable for WarehouseStore {
    fn accept<T>(&self, visitor: &mut T, resource: ResourceKind) -> T::Output
    where
        T: ResourceVisitor,
    {
        match resource {
            ResourceKind::Wood => visitor.visit_wood(self.wood),
            ResourceKind::Gold => visitor.visit_gold(self.gold),
            ResourceKind::Stone => visitor.visit_stone(self.stone),
            ResourceKind::Clay => visitor.visit_clay(self.clay),
            ResourceKind::Bricks => visitor.visit_bricks(self.bricks),
            ResourceKind::Wheat => visitor.visit_wheat(self.wheat),
            ResourceKind::Flour => visitor.visit_flour(self.flour),
            ResourceKind::Bread => visitor.visit_bread(self.bread),
            ResourceKind::Fish => visitor.visit_fish(self.fish),
            ResourceKind::IronOre => visitor.visit_iron_ore(self.iron_ore),
            ResourceKind::IronIngot => visitor.visit_iron_ingot(self.iron_ingots),
        }
    }
    
    fn accept_mut<T>(&mut self, visitor: &mut T, resource: ResourceKind) -> T::Output
    where
        T: ResourceVisitorMut,
    {
        match resource {
            ResourceKind::Wood => visitor.visit_wood_mut(&mut self.wood),
            ResourceKind::Gold => visitor.visit_gold_mut(&mut self.gold),
            ResourceKind::Stone => visitor.visit_stone_mut(&mut self.stone),
            ResourceKind::Clay => visitor.visit_clay_mut(&mut self.clay),
            ResourceKind::Bricks => visitor.visit_bricks_mut(&mut self.bricks),
            ResourceKind::Wheat => visitor.visit_wheat_mut(&mut self.wheat),
            ResourceKind::Flour => visitor.visit_flour_mut(&mut self.flour),
            ResourceKind::Bread => visitor.visit_bread_mut(&mut self.bread),
            ResourceKind::Fish => visitor.visit_fish_mut(&mut self.fish),
            ResourceKind::IronOre => visitor.visit_iron_ore_mut(&mut self.iron_ore),
            ResourceKind::IronIngot => visitor.visit_iron_ingot_mut(&mut self.iron_ingots),
        }
    }
}

/// Посетитель для суммирования ресурсов
pub struct SumVisitor {
    pub total: i32,
}

impl SumVisitor {
    pub fn new() -> Self {
        Self { total: 0 }
    }
}

impl ResourceVisitor for SumVisitor {
    type Output = ();
    
    fn visit_wood(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_gold(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_stone(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_clay(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_bricks(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_wheat(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_flour(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_bread(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_fish(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_iron_ore(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
    
    fn visit_iron_ingot(&mut self, amount: i32) -> Self::Output {
        self.total += amount;
    }
}

/// Посетитель для проверки достаточности ресурса
pub struct CheckEnoughVisitor {
    pub required: i32,
    pub result: bool,
}

impl CheckEnoughVisitor {
    pub fn new(required: i32) -> Self {
        Self {
            required,
            result: false,
        }
    }
}

impl ResourceVisitor for CheckEnoughVisitor {
    type Output = ();
    
    fn visit_wood(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_gold(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_stone(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_clay(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_bricks(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_wheat(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_flour(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_bread(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_fish(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_iron_ore(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
    
    fn visit_iron_ingot(&mut self, amount: i32) -> Self::Output {
        self.result = amount >= self.required;
    }
}

/// Посетитель для списания ресурсов
pub struct SpendVisitor {
    pub amount: i32,
    pub spent: i32,
}

impl SpendVisitor {
    pub fn new(amount: i32) -> Self {
        Self {
            amount,
            spent: 0,
        }
    }
}

impl ResourceVisitorMut for SpendVisitor {
    type Output = ();
    
    fn visit_wood_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_gold_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_stone_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_clay_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_bricks_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_wheat_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_flour_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_bread_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_fish_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_iron_ore_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
    
    fn visit_iron_ingot_mut(&mut self, amount: &mut i32) -> Self::Output {
        let take = self.amount.min(*amount);
        *amount -= take;
        self.spent += take;
        self.amount -= take;
    }
}

/// Посетитель для получения значения ресурса
pub struct GetValueVisitor {
    pub value: Option<i32>,
}

impl GetValueVisitor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl ResourceVisitor for GetValueVisitor {
    type Output = ();
    
    fn visit_wood(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_gold(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_stone(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_clay(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_bricks(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_wheat(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_flour(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_bread(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_fish(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_iron_ore(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
    
    fn visit_iron_ingot(&mut self, amount: i32) -> Self::Output {
        self.value = Some(amount);
    }
}

/// Вспомогательная функция для получения значения ресурса из Resources
#[allow(dead_code)] // Может быть полезно для будущего использования
pub fn get_resource_value(resources: &Resources, resource: ResourceKind) -> i32 {
    let mut visitor = GetValueVisitor::new();
    resources.accept(&mut visitor, resource);
    visitor.value.unwrap_or(0)
}

/// Вспомогательная функция для получения значения ресурса из WarehouseStore
#[allow(dead_code)] // Может быть полезно для будущего использования
pub fn get_warehouse_resource_value(warehouse: &WarehouseStore, resource: ResourceKind) -> i32 {
    let mut visitor = GetValueVisitor::new();
    warehouse.accept(&mut visitor, resource);
    visitor.value.unwrap_or(0)
}

/// Вспомогательная функция для суммирования ресурса из всех складов
pub fn sum_warehouses_resource(warehouses: &[WarehouseStore], resource: ResourceKind) -> i32 {
    let mut visitor = SumVisitor::new();
    for warehouse in warehouses {
        warehouse.accept(&mut visitor, resource);
    }
    visitor.total
}

