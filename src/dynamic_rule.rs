use crate::rule::{Neighbors, Rule, State};
use rand::Rng;
use rhai::{Dynamic, Engine, Scope, AST};
use rhai::packages::Package;
use rhai_rand::RandomPackage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct DynamicState {
    value: rhai::INT, // currently just int, but later we use Dynamic
}

impl Default for DynamicState {
    fn default() -> Self {
        Self { value: 0 }
    }
}

impl State for DynamicState {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.value, rhai::INT::MIN..=rhai::INT::MAX).text("value"));
    }
}

pub struct DynamicRule {
    engine: Engine,

    update_fn_str: String,
    update_fn: AST,

    clear_fn_str: String,
    clear_fn: AST,

    randomize_fn_str: String,
    randomize_fn: AST,

    next_fn_str: String,
    next_fn: AST,

    color_fn_str: String,
    color_fn: AST,

    background: egui::Color32,
}

impl Default for DynamicRule {
    fn default() -> Self {
        let mut engine = Engine::new();

        // we need a random number generator
        let random = RandomPackage::new();
        engine.register_global_module(random.as_shared_module());

        let randomize_fn_str = "fn randomize() {\
                return if rand_float() < 0.3 { 1 } else { 0 };\
            }"
        .to_string();
        let randomize_fn = engine
            .compile(&randomize_fn_str)
            .expect("default randomize script should compile successfully");

        // rand module becomes unstable when optimization level == full
        engine.set_optimization_level(rhai::OptimizationLevel::Full);

        let update_fn_str = "fn update(self, neighbors) {\
                let alive = neighbors.reduce(|sum, v|\
                    if v == 1 {sum + v} else {sum}, 0);\
                if self == 0 {\
                    return if alive == 3 { 1 } else { 0 };\
                } else {\
                    return if alive == 2 || alive == 3 { 1 } else { 0 };\
                }\
            }"
        .to_string();
        let update_fn = engine
            .compile(&update_fn_str)
            .expect("default update script should compile successfully");

        let clear_fn_str = "fn clear() {\
                return 0;\
            }"
        .to_string();
        let clear_fn = engine
            .compile(&clear_fn_str)
            .expect("default clear script should compile successfully");


        let next_fn_str = "fn next(self) {\
                return if self == 0 { 1 } else { 0 };\
            }"
        .to_string();
        let next_fn = engine
            .compile(&next_fn_str)
            .expect("default next script should compile successfully");

        let color_fn_str = "fn color(self) {\
                return if self == 0 {\
                    [0.0, 0.0, 0.0] // R, G, B in [0, 1]
                } else {
                    [0.0, 1.0, 0.0]
                };
            }"
        .to_string();
        let color_fn = engine
            .compile(&color_fn_str)
            .expect("default color script should compile successfully");

        Self {
            engine,
            update_fn_str,
            update_fn,
            clear_fn_str,
            clear_fn,
            randomize_fn_str,
            randomize_fn,
            next_fn_str,
            next_fn,
            color_fn_str,
            color_fn,
            background: egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl<const N: usize, Neighborhood: Neighbors<N>> Rule<N, Neighborhood> for DynamicRule {
    type CellState = DynamicState;

    fn background(&self) -> egui::Color32 {
        self.background
    }

    fn color(&self, st: &Self::CellState) -> anyhow::Result<egui::Color32> {
        let mut scope = Scope::new();
        let result = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.color_fn,
                false, // eval AST?
                false, // rollback scope?
                "color",
                None,
                [Dynamic::from_int(st.value)],
            )
            .expect("It should not fail");
        let rgb = result.into_array().expect("color should return [f32, f32, f32]");
        let r = (rgb[0].as_float().unwrap() * 256.0).clamp(0.0, 255.0) as u8;
        let g = (rgb[1].as_float().unwrap() * 256.0).clamp(0.0, 255.0) as u8;
        let b = (rgb[2].as_float().unwrap() * 256.0).clamp(0.0, 255.0) as u8;
        Ok(egui::Color32::from_rgb(r, g, b))
    }

    fn default_state(&self) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let result = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.clear_fn,
                false, // eval AST?
                false, // rollback scope?
                "clear",
                None,
                [],
            )
            .expect("It should not fail");
        Ok(Self::CellState { value: result.cast::<rhai::INT>() })
    }

    fn randomize<R: Rng>(&self, _rng: &mut R) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let result = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.randomize_fn,
                false, // eval AST?
                false, // rollback scope?
                "randomize",
                None,
                [],
            )
            .expect("It should not fail");
        Ok(Self::CellState { value: result.cast::<rhai::INT>() })
    }

    fn next(&self, st: Self::CellState) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let result = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.next_fn,
                false, // eval AST?
                false, // rollback scope?
                "next",
                None,
                [Dynamic::from_int(st.value)],
            )
            .expect("It should not fail");
        Ok(Self::CellState { value: result.cast::<rhai::INT>() })
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let result = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.update_fn,
                false, // eval AST?
                false, // rollback scope?
                "update",
                None,
                [
                    Dynamic::from_int(center.value),
                    Dynamic::from_array(neighbor.map(|x| Dynamic::from_int(x.value)).collect())
                ],
            )
            .expect("It should not fail");
        Ok(Self::CellState { value: result.cast::<rhai::INT>() })
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Background Color");
        egui::widgets::color_picker::color_edit_button_srgba(
            ui,
            &mut self.background,
            egui::widgets::color_picker::Alpha::Opaque,
        );
    }
}
