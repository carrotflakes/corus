use nih_plug_egui::egui;
use wavetables::tree::{Tree, Value};

use super::wavetable::wavetable_view;

pub struct WavetableLab {
    pallet: Vec<Tree>,
    selected_pallet: usize,
    param: f64,
    suggest: Vec<Tree>,
}

impl WavetableLab {
    pub fn new() -> Self {
        let mut this = Self {
            pallet: vec![
                Tree::Sin,
                Tree::Triangle,
                Tree::ShiftedTriangle,
                Tree::Saw,
                Tree::ShiftedSaw,
                Tree::Square,
                // Tree::Pulse(Value::Variable(0)),
                Tree::Steps(3.0),
                Tree::Quadratic,
            ],
            selected_pallet: 0,
            param: 0.5,
            suggest: vec![],
        };
        this.compute_sugget();
        this
    }

    pub fn compute_sugget(&mut self) {
        let current = Box::new(self.pallet[self.selected_pallet].clone());
        let mut suggest = vec![
            Tree::Negative(current.clone()),
            Tree::Reversed(current.clone()),
            Tree::Shift(Value::Variable(0), current.clone()),
            Tree::Scale(Value::Variable(0), current.clone()),
            Tree::Mirror(current.clone()),
        ];

        for wt in &self.pallet {
            suggest.push(Tree::Join(current.clone(), Box::new(wt.clone())));
            suggest.push(Tree::Join(Box::new(wt.clone()), current.clone()));
            suggest.push(Tree::Blend(
                Value::Variable(0),
                current.clone(),
                Box::new(wt.clone()),
            ));
            suggest.push(Tree::Product(current.clone(), Box::new(wt.clone())));
            suggest.push(Tree::Mul(current.clone(), Box::new(wt.clone())));
            // dynamic blend
        }

        self.suggest.clear();
        'outer: for wt in suggest {
            let wt_built = wt.build_parameterized();
            for other in self.pallet.iter().chain(self.suggest.iter()) {
                let other = other.build_parameterized();
                if (0..16 * 16)
                    .map(|i| {
                        let x = ((i % 16) as f64 + 0.5) / 16.0;
                        let y = (i / 16) as f64 / 16.0;
                        (wt_built(&[y], x) - other(&[y], x)).abs()
                    })
                    .sum::<f64>()
                    < 0.001
                {
                    continue 'outer;
                }
            }
            self.suggest.push(wt);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, on_select: Option<impl Fn(Tree)>) {
        let mut update = false;
        let mut remove_index = None;

        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for (i, wt) in self.pallet.iter().enumerate() {
                    let res = wavetable_view(
                        ui,
                        egui::vec2(32.0, 32.0),
                        wt.build(),
                        i == self.selected_pallet,
                    )
                    .context_menu(|ui| {
                        if 8 <= i && ui.button("remove").clicked() {
                            remove_index = Some(i);
                            ui.close_menu();
                        }
                        if let Some(on_select) = &on_select {
                            if ui.button("select").clicked() {
                                on_select(wt.clone());
                                ui.close_menu();
                            }
                        }
                    });
                    if res.clicked() {
                        self.selected_pallet = i;
                        update = true;
                    }
                    if res.double_clicked() {
                        if let Some(on_select) = &on_select {
                            on_select(wt.clone());
                        }
                    }
                }
            });
        });

        if let Some(index) = remove_index {
            self.pallet.remove(index);
            if self.selected_pallet >= self.pallet.len() {
                self.selected_pallet = self.pallet.len() - 1;
            }
            update = true;
        }

        if update {
            self.compute_sugget();
        }

        update = false;

        ui.add(egui::Slider::new(&mut self.param, 0.0..=1.0).text("param"));

        egui::ScrollArea::horizontal()
            .id_source("suggest")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for wt in &self.suggest {
                        if wavetable_view(
                            ui,
                            egui::vec2(32.0, 32.0),
                            wt.instant_params(&[self.param]).build(),
                            false,
                        )
                        .double_clicked()
                        {
                            self.pallet.push(wt.instant_params(&[self.param]).clone());
                            self.selected_pallet = self.pallet.len() - 1;
                            update = true;
                        }
                    }
                });
            });

        if update {
            self.compute_sugget();
        }
    }
}
