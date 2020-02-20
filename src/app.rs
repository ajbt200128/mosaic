use eframe::{
    egui::{
        self,
        plot::{Plot, PlotImage, Value},
    },
    epi,
};
use egui_extras::RetainedImage;

pub struct MosaicApp {
    image_a: RetainedImage,
    image_b: RetainedImage,
    points_a: Vec<Value>,
    points_b: Vec<Value>,
}

impl Default for MosaicApp {
    fn default() -> Self {
        Self {
            image_a: RetainedImage::from_image_bytes("a.jpeg", include_bytes!("../imgs/a.jpg"))
                .unwrap(),
            image_b: RetainedImage::from_image_bytes("b.jpeg", include_bytes!("../imgs/b.jpg"))
                .unwrap(),
            points_a: vec![],
            points_b: vec![],
        }
    }
}

impl epi::App for MosaicApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //self.image_a.show(ui);
            let plot_image_a = PlotImage::new(
                self.image_a.texture_id(ctx),
                egui::plot::Value {
                    x: (self.image_a.size_vec2().x / 2.0) as f64,
                    y: (self.image_a.size_vec2().y / 2.0) as f64,
                },
                self.image_a.size_vec2(),
            );

            let plot_image_b = PlotImage::new(
                self.image_b.texture_id(ctx),
                egui::plot::Value {
                    x: (self.image_b.size_vec2().x / 2.0) as f64,
                    y: (self.image_b.size_vec2().y / 2.0) as f64,
                },
                self.image_b.size_vec2(),
            );
            let plot_a = Plot::new("image_a_plot");

            let plot_b = Plot::new("image_b_plot");
            //let img_plot = PlotImage::new(texture_id, center_position, size)

            ui.horizontal(|ui| {
                plot_a
                    .allow_drag(false)
                    .show_axes([false, false])
                    .height(800.0)
                    .width(800.0)
                    .show(ui, |plot_ui| {
                        plot_ui.image(plot_image_a.name("image_a"));
                        if plot_ui.plot_clicked() {
                            self.points_a
                                .insert(0, plot_ui.pointer_coordinate().unwrap());
                            if self.points_a.len() > 4 {
                                self.points_a.pop();
                            }
                        }
                    });
                plot_b
                    .allow_drag(false)
                    .show_axes([false, false])
                    .height(800.0)
                    .width(800.0)
                    .show(ui, |plot_ui| {
                        plot_ui.image(plot_image_b.name("image_b"));
                        if plot_ui.plot_clicked() {
                            self.points_b
                                .insert(0, plot_ui.pointer_coordinate().unwrap());
                            if self.points_b.len() > 4 {
                                self.points_b.pop();
                            }
                        }
                    });
            });
            if ui.button("Merge").clicked(){
                println!("Click");
            }
            egui::warn_if_debug_build(ui);
        });
    }
}
