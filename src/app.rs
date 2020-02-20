use arrayfire::{homography, print, Array, Dim4};
use eframe::{
    egui::{
        self,
        plot::{Plot, PlotImage, Value},
    },
    epaint::ColorImage,
    epi,
};
use egui_extras::RetainedImage;
use image::{
    imageops::resize, io::Reader, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel,
    RgbaImage,
};
use imageproc::geometric_transformations::{warp_into, Projection};

pub struct MosaicApp {
    image_a: RetainedImage,
    image_b: RetainedImage,
    image_a_orig: DynamicImage,
    image_b_orig: DynamicImage,
    points_a: Vec<Value>,
    points_b: Vec<Value>,
    warped: Option<RetainedImage>,
    warped_orig: Option<DynamicImage>,
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
            warped_orig: None,
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

fn distance_alpha((a_x, a_y): (f64, f64), (b_x, b_y): (f64, f64), max: u32) -> u8 {
    255 - (((a_x - b_x).powf(2.0) + (a_y - b_y).powf(2.0)).sqrt() / max as f64) as u8
}

fn overlay_into(a: &DynamicImage, b: &mut DynamicImage, center: (f64, f64)) {
    let mut b_n = b.clone();
    println!("a-------");
    for y in 0..a.height() {
        for x in 0..a.width() {
            let mut p = a.get_pixel(x, y);
            let mut q = b.get_pixel(x, y);
            //p.0[3] = distance_alpha(center, (x as f64, y as f64), b.width());
            if p.0[3] == 0 {
                p = q;
            } else if q.0[3] != 0 {
                q.0[3] = 125;
                p.0[3] = 125;
                p.blend(&q);
            }
            b_n.put_pixel(x, y, p);
        }
    }
    b_n.save("dbg.jpg");
    let b_n_a: DynamicImage = image::DynamicImage::ImageRgba8(resize(
        &b_n,
        b_n.width() / 8,
        b_n.height() / 8,
        image::imageops::FilterType::Nearest,
    ));
    println!("b-----");
    b_n_a.blur(100.0);
    let b_n_b: DynamicImage = image::DynamicImage::ImageRgba8(resize(
        &b_n_a,
        b_n_a.width() / 8,
        b_n_a.height() / 8,
        image::imageops::FilterType::Nearest,
    ));
    b_n_b.blur(200.0);
    println!("b-----");
    let b_n_a: DynamicImage = image::DynamicImage::ImageRgba8(resize(
        &b_n_a,
        b.width(),
        b.height(),
        image::imageops::FilterType::Nearest,
    ));
    let b_n_b: DynamicImage = image::DynamicImage::ImageRgba8(resize(
        &b_n_b,
        b.width(),
        b.height(),
        image::imageops::FilterType::Nearest,
    ));
    println!("c------");
    for y in 0..b.height() {
        for x in 0..b.width() {
            let mut p = b_n.get_pixel(x, y);
            let mut p_a = b_n_a.get_pixel(x, y);
            let mut p_b = b_n_b.get_pixel(x, y);
            let mut q = b.get_pixel(x, y);

            let mut r = if x < a.width() && y < a.height() {
                a.get_pixel(x, y)
            } else {
                image::Rgba([0, 0, 0, 0])
            };
            //if r.0[3] == 0 && q.0[3] != 0{
            //    p = q
            //}else if r.0[3] != 0 && q.0[3] == 0{
            //    p = r;
            //}else{
            p_a.0[3] = 185;
            // Smallest
            p_b.0[3] = 125;

            // Blend all three photos together
            p_a.blend(&p_b);
None            p.blend(&p_a);
            // Set alpha according to distance from center
            p.0[3] = distance_alpha(center, (x as f64, y as f64), b.width());

            // Blend first photo and all merged photos
            if r.0[3] != 0 {
                r.0[3] = 150;
                p.blend(&r);
            }
            p.0[3] = 255;

            
            //}

            b.put_pixel(x, y, p);
        }
    }
    println!("d------");
}

fn find_homography(a: Vec<Value>, b: Vec<Value>) -> [f32; 9] {
    let mut v = [1.0; 9];
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
    let x_src = Array::new(&x_src, Dim4::new(&[4, 1, 1, 1]));
    let y_src = Array::new(&y_src, Dim4::new(&[4, 1, 1, 1]));
    let x_dst = Array::new(&x_dst, Dim4::new(&[4, 1, 1, 1]));
    let y_dst = Array::new(&y_dst, Dim4::new(&[4, 1, 1, 1]));
    let (h, i): (Array<f32>, i32) = homography(
        &x_src,
        &y_src,
        &x_dst,
        &y_dst,
        arrayfire::HomographyType::RANSAC,
        100000.0,
        10,
    );

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
                                self.points_a.insert(0, coord);
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
                                self.points_b.insert(0, coord);
                                if self.points_b.len() > 4 {
                                    self.points_b.pop();
                                }
                            }
                        });
                });
                if self.warped.is_some() {
                    if ui.button("save").clicked() {
                       self.warped_orig.clone().unwrap().save("out.jpg");

                    }
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
                    let mut canvas: RgbaImage =
                        ImageBuffer::new(self.image_a_orig.width() * 2, self.image_a_orig.height());
                    warp_into(
                        &self.image_b_orig.to_rgba8(),
                        &projection,
                        imageproc::geometric_transformations::Interpolation::Nearest,
                        white,
                        &mut canvas,
                    );
                    let mut canvas = image::DynamicImage::ImageRgba8(canvas);
                    let x = self
                        .points_a
                        .clone()
                        .iter()
                        .fold(0.0, |cntr, curr| cntr + curr.x)
                        / 4.0;
                    let y = self
                        .points_a
                        .clone()
                        .iter()
                        .fold(0.0, |cntr, curr| cntr + curr.y)
                        / 4.0;
                    overlay_into(&self.image_a_orig, &mut canvas, (x, y));
                    self.warped_orig = Some(canvas.clone());
                    self.warped = Some(to_retained("w", canvas));
                    self.points_a = vec![];
                    self.points_b = vec![];
                }
            }
            egui::warn_if_debug_build(ui);
        });
    }
}
