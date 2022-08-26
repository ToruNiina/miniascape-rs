use crate::rule::{Neighbors, Rule, State};
use rand::Rng;
use rhai::packages::Package;
use rhai::{Dynamic, Engine, Scope, AST};
use rhai_rand::RandomPackage;
use serde::{Deserialize, Serialize};

use anyhow::Context as _;
use thiserror::Error;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct DynamicState {
    value: rhai::Dynamic,
}

impl State for DynamicState {
    fn inspect(&mut self, ui: &mut egui::Ui, buf: &mut String) {
        ui.label(format!("Dynamic value: {:?}", self.value));

        let buf_is_multiline = buf.chars().filter(|x| *x == '\n').count() != 0;

        ui.horizontal_wrapped(|ui| {
            ui.label("expr = ");
            if buf_is_multiline {
                ui.text_edit_multiline(buf);
            } else {
                ui.text_edit_singleline(buf);
            }
        });
        if ui.button("eval").clicked() {
            let mut engine = Engine::new();
            let random = RandomPackage::new();
            engine.register_global_module(random.as_shared_module());

            let result = engine.eval::<rhai::Dynamic>(buf);
            if let Ok(val) = result {
                self.value = val;
            } else {
                *buf = format!("//error: {:?}\n{}", result.unwrap_err(), buf)
            }
        }
    }
}

pub struct DynamicRule {
    engine: Engine,

    dropped_files: Vec<egui::DroppedFile>,

    update_fn_str: String,
    update_fn: AST,
    open_update_fn: bool,
    open_update_fn_compilation_result: Option<anyhow::Error>,

    clear_fn_str: String,
    clear_fn: AST,
    open_clear_fn: bool,
    open_clear_fn_compilation_result: Option<anyhow::Error>,

    randomize_fn_str: String,
    randomize_fn: AST,
    open_randomize_fn: bool,
    open_randomize_fn_compilation_result: Option<anyhow::Error>,

    next_fn_str: String,
    next_fn: AST,
    open_next_fn: bool,
    open_next_fn_compilation_result: Option<anyhow::Error>,

    color_fn_str: String,
    color_fn: AST,
    open_color_fn: bool,
    open_color_fn_compilation_result: Option<anyhow::Error>,

    background: egui::Color32,
}

impl Default for DynamicRule {
    fn default() -> Self {
        let mut engine = Engine::new();

        // we need a random number generator
        let random = RandomPackage::new();
        engine.register_global_module(random.as_shared_module());

        let randomize_fn_str = r#"
fn randomize() {
    return if rand_float() < 0.3 { true } else { false };
}
"#
        .to_string();
        let randomize_fn = engine
            .compile(&randomize_fn_str)
            .expect("default randomize script should compile successfully");

        // rand module becomes unstable when optimization level == full
        engine.set_optimization_level(rhai::OptimizationLevel::Full);

        let update_fn_str = r#"
fn update(self, neighbors) {
    let alive = neighbors
        .reduce(|sum, v| {
            if v {sum + 1} else {sum}
        }, 0);

    if !self {
        return if alive == 3 {
            true
        } else {
            false
        };
    } else {
        return if alive == 2 ||
                  alive == 3 {
            true
        } else {
            false
        };
    }
}"#
        .to_string();
        let update_fn = engine
            .compile(&update_fn_str)
            .expect("default update script should compile successfully");

        let clear_fn_str = r#"
fn clear() {
    return false;
}"#
        .to_string();
        let clear_fn = engine
            .compile(&clear_fn_str)
            .expect("default clear script should compile successfully");

        let next_fn_str = r#"
fn next(self) {
    return !self;
}"#
        .to_string();
        let next_fn = engine
            .compile(&next_fn_str)
            .expect("default next script should compile successfully");

        let color_fn_str = r#"
fn color(self) {
    return if self {
        [0.1, 1.0, 0.1]
    } else {
        [0.1, 0.1, 0.1]
    };
}"#
        .to_string();
        let color_fn = engine
            .compile(&color_fn_str)
            .expect("default color script should compile successfully");

        Self {
            engine,
            dropped_files: Default::default(),

            update_fn_str,
            update_fn,
            open_update_fn: true,
            open_update_fn_compilation_result: None,

            clear_fn_str,
            clear_fn,
            open_clear_fn: true,
            open_clear_fn_compilation_result: None,

            randomize_fn_str,
            randomize_fn,
            open_randomize_fn: true,
            open_randomize_fn_compilation_result: None,

            next_fn_str,
            next_fn,
            open_next_fn: true,
            open_next_fn_compilation_result: None,

            color_fn_str,
            color_fn,
            open_color_fn: true,
            open_color_fn_compilation_result: None,

            background: egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

#[derive(Error, Debug)]
pub enum DynamicRuleError {
    #[error("EvalAltResult \"{0}\"\ncaused by the following code: \n{1}")]
    EvalError(String, String),

    #[error(
        "Dynamic::into_{1}() failed, actual type is \"{0}\"\ncaused by the following code: \n{2}"
    )]
    CastFail(String, String, String),

    #[error("Dropped File Error: {0} about file \"{1}\"")]
    FileError(String, String),
}

// Box<rhai::EvalAltResult> does not satisfy trait bound of anyhow context
fn eval_error(item: Box<rhai::EvalAltResult>, code: String) -> DynamicRuleError {
    DynamicRuleError::EvalError(format!("{}", item), code)
}
fn cast_error(item: &str, typename: String, code: String) -> DynamicRuleError {
    DynamicRuleError::CastFail(item.to_string(), typename, code)
}

impl<N: Neighbors> Rule<N> for DynamicRule {
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
                [st.value.clone()],
            )
            .map_err(|x| eval_error(x, self.color_fn_str.clone()))
            .context("Failed to evaluate color")?;

        let rgb = result
            .into_array()
            .map_err(|x| cast_error(x, "array".to_string(), self.color_fn_str.clone()))
            .context("Failed to convert `fn color` result into an array")?;

        let r = (rgb[0]
            .as_float()
            .map_err(|x| cast_error(x, "float".to_string(), self.color_fn_str.clone()))
            .context("Failed to convert `fn color` result element")?
            * 256.0)
            .clamp(0.0, 255.0) as u8;
        let g = (rgb[1]
            .as_float()
            .map_err(|x| cast_error(x, "float".to_string(), self.color_fn_str.clone()))
            .context("Failed to convert `fn color` result element")?
            * 256.0)
            .clamp(0.0, 255.0) as u8;
        let b = (rgb[2]
            .as_float()
            .map_err(|x| cast_error(x, "float".to_string(), self.color_fn_str.clone()))
            .context("Failed to convert `fn color` result element")?
            * 256.0)
            .clamp(0.0, 255.0) as u8;
        Ok(egui::Color32::from_rgb(r, g, b))
    }

    fn default_state(&self) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let value = self
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
            .map_err(|x| eval_error(x, self.clear_fn_str.clone()))
            .context("Failed to evaluate clear")?;

        Ok(Self::CellState { value })
    }

    fn randomize<R: Rng>(&self, _rng: &mut R) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let value = self
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
            .map_err(|x| eval_error(x, self.randomize_fn_str.clone()))
            .context("Failed to evaluate randomize")?;

        Ok(Self::CellState { value })
    }

    fn next(&self, st: Self::CellState) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let value = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.next_fn,
                false, // eval AST?
                false, // rollback scope?
                "next",
                None,
                [st.value],
            )
            .map_err(|x| eval_error(x, self.next_fn_str.clone()))
            .context("Failed to evaluate next")?;

        Ok(Self::CellState { value })
    }

    fn update(
        &self,
        center: Self::CellState,
        neighbor: impl Iterator<Item = Self::CellState>,
    ) -> anyhow::Result<Self::CellState> {
        let mut scope = Scope::new();
        let value = self
            .engine
            .call_fn_raw(
                &mut scope,
                &self.update_fn,
                false, // eval AST?
                false, // rollback scope?
                "update",
                None,
                [center.value, Dynamic::from_array(neighbor.map(|x| x.value).collect())],
            )
            .map_err(|x| eval_error(x, self.update_fn_str.clone()))
            .context("Failed to evaluate update")?;

        Ok(Self::CellState { value })
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> anyhow::Result<()> {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("Background Color");
            egui::widgets::color_picker::color_edit_button_srgba(
                ui,
                &mut self.background,
                egui::widgets::color_picker::Alpha::Opaque,
            );

            ui.separator();

            ui.horizontal_wrapped(|ui| {
                ui.label("It uses");
                ui.hyperlink_to("rhai", "https://rhai.rs/");
                ui.label("as a scripting language.");
            });

            ui.separator();

            Self::ui_code_editor(
                "toggle cell update rule",
                "cell update rule takes the central cell and its neighbors and \
                returns the next state of the central cell.",
                ui,
                ctx,
                &mut self.update_fn_str,
                &mut self.update_fn,
                &mut self.open_update_fn,
                &mut self.open_update_fn_compilation_result,
                |fn_str| {
                    self.engine.set_optimization_level(rhai::OptimizationLevel::Full);
                    self.engine.compile(fn_str).context("failed to compile `fn update()`")
                },
            );
            ui.separator();

            Self::ui_code_editor(
                "toggle clear rule",
                "clear rule returns the default state to clear the board.",
                ui,
                ctx,
                &mut self.clear_fn_str,
                &mut self.clear_fn,
                &mut self.open_clear_fn,
                &mut self.open_clear_fn_compilation_result,
                |fn_str| {
                    self.engine.set_optimization_level(rhai::OptimizationLevel::Full);
                    self.engine.compile(fn_str).context("failed to compile `fn clear()`")
                },
            );
            ui.separator();

            Self::ui_code_editor(
                "toggle randomize rule",
                "randomize rule randomizes the cell using rhai-rand module.",
                ui,
                ctx,
                &mut self.randomize_fn_str,
                &mut self.randomize_fn,
                &mut self.open_randomize_fn,
                &mut self.open_randomize_fn_compilation_result,
                |fn_str| {
                    // rand module becomes unstable when optimization level == full
                    self.engine.set_optimization_level(rhai::OptimizationLevel::Simple);
                    self.engine.compile(fn_str).context("failed to compile `fn randomize()`")
                },
            );
            ui.separator();

            Self::ui_code_editor(
                "toggle next rule",
                "next rule is to change the cell state when clicked.",
                ui,
                ctx,
                &mut self.next_fn_str,
                &mut self.next_fn,
                &mut self.open_next_fn,
                &mut self.open_next_fn_compilation_result,
                |fn_str| {
                    self.engine.set_optimization_level(rhai::OptimizationLevel::Full);
                    self.engine.compile(fn_str).context("failed to compile `fn next()`")
                },
            );
            ui.separator();

            Self::ui_code_editor(
                "toggle color rule",
                "color rule defines the color depending on the cell state.\
                the resulting value is an array of f32 in [0,1] range, in the order of [r, g, b].",
                ui,
                ctx,
                &mut self.color_fn_str,
                &mut self.color_fn,
                &mut self.open_color_fn,
                &mut self.open_color_fn_compilation_result,
                |fn_str| {
                    self.engine.set_optimization_level(rhai::OptimizationLevel::Full);
                    self.engine.compile(fn_str).context("failed to compile `fn color()`")
                },
            );
        });

        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }

        // load file content and compile the code
        if !self.dropped_files.is_empty() {
            let file = self.dropped_files[0].clone();
            self.dropped_files.clear();
            if let Some(bytes) = &file.bytes {
                let content = std::str::from_utf8(bytes)
                    .context(format!("Couldn't read file content as utf8 -> {}", file.name))?
                    .to_owned();

                self.update_fn_str = content.clone();
                self.clear_fn_str = content.clone();
                self.randomize_fn_str = content.clone();
                self.next_fn_str = content.clone();
                self.color_fn_str = content.clone();

                self.engine.set_optimization_level(rhai::OptimizationLevel::Simple);
                self.randomize_fn = self
                    .engine
                    .compile(&content)
                    .context(format!("failed to compile file content -> {}", file.name))?;

                self.engine.set_optimization_level(rhai::OptimizationLevel::Full);
                let ast = self
                    .engine
                    .compile(content)
                    .context(format!("failed to compile file content -> {}", file.name))?;

                self.update_fn = ast.clone();
                self.clear_fn = ast.clone();
                self.next_fn = ast.clone();
                self.color_fn = ast;
            } else {
                return Err(DynamicRuleError::FileError(
                    "couldn't read file content".to_string(),
                    file.name.clone(),
                )
                .into());
            }
        }
        Ok(())
    }
}

impl DynamicRule {
    #[allow(clippy::too_many_arguments)]
    fn ui_code_editor(
        button_name: &str,
        description: &str,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        fn_str: &mut String,
        fn_ast: &mut AST,
        open_fn: &mut bool,
        result: &mut Option<anyhow::Error>,
        compile: impl FnOnce(&String) -> anyhow::Result<AST>,
    ) {
        ui.label(description);
        if ui.button(button_name).clicked() {
            *open_fn = !*open_fn;
        }

        if *open_fn {
            if ui.button("compile").clicked() {
                let ast = compile(fn_str);
                if let Ok(compiled) = ast {
                    *fn_ast = compiled;
                    *result = None;
                } else {
                    *result = ast.err();
                }
            }
            if let Some(err) = result {
                ui.label(format!("{:?}", err));
            }
            ui.add(egui::TextEdit::multiline(fn_str).code_editor().desired_width(f32::INFINITY));
        }
    }
}
