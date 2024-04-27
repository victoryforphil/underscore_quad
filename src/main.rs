use eframe::{egui_glow, glow};
use egui::{CentralPanel, SidePanel};
use openh264::formats::YUVSource;
use openh264::formats::RGBSource;
use log::*;
use openh264::{decoder::{DecodedYUV, Decoder}, nal_units};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tello::*;
use three_d::*;
mod frame_input;
fn with_three_d<R>(gl: &std::sync::Arc<glow::Context>, f: impl FnOnce(&mut ThreeDApp) -> R) -> R {
    use std::cell::RefCell;
    thread_local! {
        pub static THREE_D: RefCell<Option<ThreeDApp>> = RefCell::new(None);
    }

    THREE_D.with(|three_d| {
        let mut three_d = three_d.borrow_mut();
        let three_d = three_d.get_or_insert_with(|| ThreeDApp::new(gl.clone()));
        f(three_d)
    })
}
type ThreadedImage = Arc<Mutex<[[u8; 3]; 800 * 800]>>;
struct MyApp {
    pub drone_rx: mpsc::Receiver<Message>,
    pub image: ThreadedImage,
    pub decoder: Decoder,
}

impl MyApp {
    fn new(rx: Receiver<Message>) -> Self {
        Self { drone_rx: rx , image: Arc::new(Mutex::new([[0; 3]; 800 * 800])), decoder: Decoder::new().unwrap()}
    }
    fn custom_painting(&mut self, ui: &mut egui::Ui, image: ThreadedImage) {
        let size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::drag());

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
                with_three_d(painter.gl(), |three_d| {
                    three_d.frame(frame_input::FrameInput::new(
                        &three_d.context,
                        &info,
                        painter
                    ),image.clone());
                });
            })),
        };
        ui.painter().add(callback);
        
    }
}

pub struct ThreeDApp {
    context: Context,
    camera: Camera,
    image: Arc<Texture2D>,

}

impl ThreeDApp {
    pub fn new(gl: std::sync::Arc<glow::Context>) -> Self {
        let context = Context::from_gl_context(gl).unwrap();
        // Create a camera
        let mut camera = Camera::new_2d(Viewport::new_at_origo(800, 800));

        let mut floor = Gm::new(
            Mesh::new(&context, &CpuMesh::square()),
            PhysicalMaterial::new_opaque(
                &context,
                &CpuMaterial {
                    albedo: Srgba::new(150, 150, 150, 255),
                    metallic: 0.1,
                    roughness: 0.3,
                    ..Default::default()
                },
            ),
        );

        floor.set_transformation(
            Mat4::from_translation(vec3(0.0, 0.0, -5.0))
                * Mat4::from_nonuniform_scale(10., 10.0, 0.1),
        );

        let mut tex = Texture2D::new_empty::<[u8; 3]>(
            &context,
            800,
            800,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let mut pixels = vec![[0; 3]; 800 * 800];

        //Set pixel to green
        for i in 0..800 {
            for j in 0..800 {
                pixels[i * 800 + j] = [0, 255, 0];
            }
        }

        tex.fill(&pixels);
        Self {
            context,
            camera,
            image: Arc::new(tex),
        }
    }

    pub fn frame(&mut self, frame_input: frame_input::FrameInput, image: ThreadedImage) -> Option<glow::Framebuffer> {
        // Ensure the viewport matches the current window viewport which changes if the window is resized
        self.camera.set_viewport(frame_input.viewport);


        let mut objects: Vec<&dyn Object> = vec![];

        let cam_up = self.camera.up().clone();
        let cam_pos = self.camera.position().clone();
        let taget = Vector3::zero();
        self.camera.set_view(cam_pos, taget, cam_up);
        let mut texture_transform_scale = 1.0;
        let mut texture_transform_x = 0.0;

        let mut texture_transform_y = 0.0;
       
      

        let yuv_pixels = *image.lock().unwrap();

        let mut tex = Texture2D::new_empty::<[u8; 3]>(
            &self.context,
            800,
            800,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        tex.fill(&yuv_pixels);

 
        let material = ColorMaterial {
            texture: Some(Texture2DRef {
                texture: Arc::new(tex),
                transformation: Mat3::from_scale(texture_transform_scale)
                    * Mat3::from_translation(vec2(texture_transform_x, texture_transform_y)),
            }),
            ..Default::default()
        };
        
  


        frame_input
            .screen
            .clear_partially(frame_input.scissor_box, ClearState::depth(1.0));
        frame_input
            .screen
            .apply_screen_material(&material, &self.camera, &[]);
        frame_input
            .screen
            // Clear the color and depth of the screen render target
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render_partially(
                frame_input.scissor_box,
                &self.camera,
                &objects,
                &[],
            );

        frame_input.screen.into_framebuffer() // Take back the screen fbo, we will continue to use it.
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui, self.image.clone());
            });
            ctx.request_repaint_after(Duration::from_millis(10));
        });

        SidePanel::left("Drone").show(ctx, |ui| {
            let mut messages = vec![];
            while let Ok(msg) = self.drone_rx.try_recv() {
               
                messages.push(msg);
               
            }
         
                        
            for msg in messages {
                match msg {
                    Message::Frame(frame_id, data) => {
                        
                       // On the first few frames this may fail, so you should check the result
                            // a few packets before giving up.
                            let maybe_some_yuv = self.decoder.decode(data.as_slice());
                            if let Ok(Some(yuv)) = maybe_some_yuv {
                                let mut pixels = self.image.lock().unwrap();
                                for i in 0..800 {
                                    for j in 0..800 {
                                        let y = yuv.y()[i * 800 + j];
                                        pixels[i * 800 + j] = [y, y, y];
                                    }
                                }
                                info!("Decoded frame {}", frame_id);
                            }else{
                                warn!("Failed to decode frame {}", frame_id);
                            }
                    },

                    _ => {}
                }
            }
        });
    }
}

fn main() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (tx, rx) = mpsc::channel();

    let drone_thread = thread::spawn(move || {
        let mut drone = Drone::new("192.168.10.1:8889");
        drone.connect(11111);
        drone.set_video_mode(VideoMode::M960x720);
        drone.start_video();
        loop {
            if let Some(msg) = drone.poll() {
                tx.send(msg).unwrap();
                drone.start_video();
            }
            ::std::thread::sleep(Duration::from_millis(50));
        }
    });
    let options = eframe::NativeOptions {
        stencil_buffer: 8,
        depth_buffer: 24,

        ..Default::default()
    };

    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:

            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<MyApp>::new(MyApp::new(rx))
        }),
    );
    drone_thread.join().unwrap();
}
