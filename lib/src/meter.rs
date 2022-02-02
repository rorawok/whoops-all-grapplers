use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Inspectable, Component, Clone, Copy)]
pub struct Meter {
    // See Health comment
    ratio: f32,
    value: i32,
    max: i32,
    combo_meter: i32,
}

impl Default for Meter {
    fn default() -> Self {
        Self {
            ratio: 1.0,
            value: 100,
            max: 100,
            combo_meter: 0,
        }
    }
}
impl Meter {
    pub fn get_ratio(&self) -> f32 {
        self.ratio
    }
    pub fn reset(&mut self) {
        self.value = self.max;
        self.ratio = 1.0;
    }
    pub fn can_afford(&self, amount: i32) -> bool {
        self.value >= amount
    }
    pub fn pay(&mut self, amount: i32) {
        assert!(self.value >= amount, "Meter overdraft");
        self.gain(-amount);
    }
    pub fn gain(&mut self, amount: i32) {
        self.value = (self.value + amount).min(self.max);
        self.ratio = self.value as f32 / self.max as f32;
    }
    pub fn add_combo_meter(&mut self, damage: i32) {
        const METER_GAINED_PER_DAMAGE: f32 = 0.5;
        self.combo_meter += (damage as f32 * METER_GAINED_PER_DAMAGE) as i32;
    }
    pub fn flush_combo(&mut self) {
        self.gain(self.combo_meter);
        self.combo_meter = 0;
    }
}
