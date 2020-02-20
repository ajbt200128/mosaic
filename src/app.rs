use arrayfire::{homography, print, Array, Dim4};
use eframe::{
    egui::{
        self,
        plot::{Plot, PlotImage, Value},
    },
    epaint::ColorImage,
    epi,
};
use egui_extras::{image::load_image_bytes, RetainedImage};
use image::{io::Reader, DynamicImage, EncodableLayout, GenericImageView, Pixel, ImageBuffer, RgbaImage, GenericImage};
use imageproc::{geometric_transformations::{warp, Projection, warp_into}, map::map_colors2};

pub struct MosaicApp {
    image_a: RetainedImage,
    image_b: RetainedImage,
    image_a_orig: DynamicImage,
    image_b_orig: DynamicImage,
    points_a: Vec<Value>,
    points_b: Vec<Value>,
    warped: Option<RetainedImage>,
}

impl Default for MosaicApp {
    fn default() -> Self {
        let im1 = Reader::open("imgs/a.jpg").unwrap().decode().unwrap();
        let im2 = Reader::open("imgs/b.jpg").unwrap().decode().unwrap();
        Self {
            image_a: to_retained("image_a", im1.clone()),
            image_b: to_retained("image_b", im2.clone()),
            image_a_orig: im1,
            image_b_orig: im2,
            points_a: vec![],
            points_b: vec![],
            warped: None,
        }
    }
}

fn to_retained(debug_name: impl Into<String>, im: DynamicImage) -> RetainedImage {
    let size = [im.width() as _, im.height() as _];
    let mut pixels = im.to_rgba8();
    let pixels = pixels.as_flat_samples_mut();
    RetainedImage::from_color_image(
        debug_name,
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
    )
}

fn clamp_add(a: u8, b: u8, max: u8) -> u8 {
    if (a as u16 + b as u16) > max.into() {
        max
    } else {
        a + b
    }
}
fn overlay_into(a: &DynamicImage, b: &mut DynamicImage){
    for y in 0..a.height(){
        for x in 0..a.width(){
            let mut p = a.get_pixel(x, y);
            let q = b.get_pixel(x, y);
            if p.0[3] == 0{
                p = q;
            }else if q.0[3] != 0{
                p.apply2(&q, |c1, c2| ((c1 as u16 + c2 as u16)/2).min(u8::MAX.into()) as u8);
            }
            b.put_pixel(x, y, p);
        }
    }
}

fn find_homography(a: Vec<Value>, b: Vec<Value>) -> [f32; 9] {
    let mut v = [1.0; 9];
    //let x_delta = a[0].x - b[0].x;
    //let y_delta = a[0].y - b[0].y;
    let mut x_src = [0.0; 4];
    let mut y_src = [0.0; 4];
    let mut x_dst = [0.0; 4];
    let mut y_dst = [0.0; 4];
    for i in 0..a.len() {
        x_src[i] = a[i].x as f32;
        y_src[i] = a[i].y as f32;
        x_dst[i] = b[i].x as f32;
        y_dst[i] = b[i].y as f32;
    }
    //println!(
    //    "X_S: {:?}, X_D: {:?}\n Y_S: {:?} Y_D: {:?}",
    //    x_src, x_dst, y_src, y_dst
    //);
    let x_src = Array::new(&x_src, Dim4::new(&[4, 1, 1, 1]));
    let y_src = Array::new(&y_src, Dim4::new(&[4, 1, 1, 1]));
    let x_dst = Array::new(&x_dst, Dim4::new(&[4, 1, 1, 1]));
    let y_dst = Array::new(&y_dst, Dim4::new(&[4, 1, 1, 1]));
    //print(&x_src);
    let (h, i): (Array<f32>, i32) = homography(
        &x_src,
        &y_src,
        &x_dst,
        &y_dst,
        arrayfire::HomographyType::RANSAC,
        100000.0,
        10,
    );
    //println!("I: {}", i);

    print(&h);
    h.host(&mut v);
    v
}

impl epi::App for MosaicApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
            let plot_c = Plot::new("image_c_plot");
            //let img_plot = PlotImage::new(texture_id, center_position, size)

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    plot_a
                        .allow_drag(false)
                        .show_axes([false, false])
                        .height(800.0)
                        .width(800.0)
                        .show(ui, |plot_ui| {
                            plot_ui.image(plot_image_a.name("image_a"));
                            if plot_ui.plot_clicked() {
                                let mut coord = plot_ui.pointer_coordinate().unwrap();
                                coord.y = self.image_a_orig.height() as f64 - coord.y;
                                self.points_a
                                    .insert(0, coord);
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
                                let mut coord = plot_ui.pointer_coordinate().unwrap();
                                coord.y = self.image_b_orig.height() as f64 - coord.y;
                                self.points_b
                                    .insert(0,coord);
                                if self.points_b.len() > 4 {
                                    self.points_b.pop();
                                }
                            }
                        });
                });
                if self.warped.is_some(){
                    let plot_image_c = PlotImage::new(
                        self.warped.as_ref().unwrap().texture_id(ctx),
                        egui::plot::Value {
                            x: (self.image_b.size_vec2().x / 2.0) as f64,
                            y: (self.image_b.size_vec2().y / 2.0) as f64,
                        },
                        self.image_b.size_vec2(),
                    );
                    plot_c
                        .allow_drag(false)
                        .show_axes([false, false])
                        .height(800.0)
                        .width(1600.0)
                        .show(ui, |plot_ui| {
                            plot_ui.image(plot_image_c.name("image_c"));
                        });
                }
            });
            if ui.button("Merge").clicked() {
                if self.points_a.len() == 4 && self.points_b.len() == 4 {
                    let h = find_homography(self.points_b.clone(), self.points_a.clone());
                    let projection = Projection::from_matrix(h).unwrap();
                    let white: image::Rgba<u8> = image::Rgba([0, 0, 0, 0]);
                    let mut canvas: RgbaImage = ImageBuffer::new(self.image_b_orig.width()*2, self.image_b_orig.height());
                    warp_into(
                        &self.image_b_orig.to_rgba8(),
                        &projection,
                        imageproc::geometric_transformations::Interpolation::Nearest,
                        white,
                        &mut canvas
                    );
                    let mut canvas = image::DynamicImage::ImageRgba8(canvas);
                    overlay_into(&self.image_a_orig,&mut canvas);
                    self.warped = Some(to_retained("w", canvas));
                    self.points_a = vec![];
                    self.points_b = vec![];
                }
            }
            egui::warn_if_debug_build(ui);
        });
    }
}
