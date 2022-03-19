use std::time::Instant;

use glam::{vec2, vec3};
use glutin::{
    dpi::{PhysicalSize, Size},
    event::{Event, VirtualKeyCode, WindowEvent},
};

use crate::render::window::GameWindow;

use self::{
    camera::Camera, engine::GameEngine, network::client::{Client, handle, packet::ServerPlayPacket}, render_stages::RenderManager,
    world::{World, ChunkPosition, Block},
};

mod camera;
pub mod engine;
mod mesh;
mod network;
mod render_stages;
mod texture;
mod world;
pub struct CubeGame {
    /// Camera.
    camera: Camera,
    /// Network client.
    client: Client,
    /// World.
    world: World,
    /// Render manager.
    render_manager: RenderManager,

    /// Removed on run
    _engine: Option<GameEngine<Self>>,
}
impl CubeGame {
    pub fn set_block(&mut self, block: Block) {
        let mut world_render = self.render_manager.world.borrow_mut();
        self.world.set_block(block);
        world_render.build_chunk(self.world.clone(), block.position.to_chunk());
    }
    pub async fn new(window_size: impl Into<Size> + Copy) -> Self {
        let (client, package) = Client::connect("127.0.0.1:25565", String::from("Exo"))
            .await
            .unwrap();
        let mut engine = GameEngine::<CubeGame>::new(window_size);
        let ctx = engine.renderer().window.context();
        ctx.set_error_handler(|_source, _err_type, _id, _severity, message| {
            if _severity != gl::DEBUG_SEVERITY_NOTIFICATION {
                log::error!("Error: {:?}", message);
            }
        });
        ctx.enable(gl::DEPTH_TEST);
        ctx.enable(gl::CULL_FACE);
        engine.renderer().window.set_cursor_grabbed(true).unwrap();
        engine.add_event_handler(move_camera);
        engine.add_event_handler(|engine, window, cube, event| {
            if let Event::MainEventsCleared = event {
                if engine.input.keyboard.was_pressed(VirtualKeyCode::X) {
                    let grabbed = window.cursor_grabbed();
                    window.set_cursor_grabbed(!grabbed);
                }
            }
        });
        engine.add_event_handler(|engine, window, cube, event| {
            if let Event::MainEventsCleared = event {
                cube.client.update_position(cube.camera.position.x, cube.camera.position.y, cube.camera.position.z, 0, 0);
            }
        });
        engine.add_event_handler(|engine, window, cube, event| {
            if let Event::WindowEvent {
                event: WindowEvent::Resized(..),
                ..
            } = event
            {
                cube.camera
                    .update_wh(window.size().width as i32, window.size().height as i32);
            }
        });

        let mut frames_per_second = 0.0;
        let mut fps = 0;
        let mut last_time = Instant::now();

        engine.add_event_handler(move |engine, window, cube, event| {
            if let Event::MainEventsCleared = event {
                let mut current_time = Instant::now();
                frames_per_second += 1.0;
                if (current_time - last_time).as_secs_f32() > 1.0 {
                    last_time = current_time;
                    fps = frames_per_second as i32;
                    frames_per_second = 0.;
                }
                cube.render_manager.text.borrow_mut().render(
                    format!("FPS: {}", fps),
                    vec2(25., (window.size().height) as f32 - 40.),
                    0.5,
                    vec3(1., 1., 1.),
                );
            }
        });


        engine.add_event_handler(move |engine, window, cube, event| {
            if let Event::MainEventsCleared = event {
                for packet in cube.client.reader.try_iter().collect::<Vec<ServerPlayPacket>>() {
                    handle::handle_packet(cube, packet).unwrap();
                }
            }
        });


        let render_manager = RenderManager::new(&mut engine);
        let mut world = package.world;
        let mut world = World::from_file("big.cw").unwrap();
        let mut worldrender = render_manager.world.borrow_mut();
        for x in 0..(world.length() >> 4) + 1 {
            for y in 0..(world.height() >> 4) + 1 {
                for z in 0..(world.width() >> 4) + 1 {
                    worldrender.build_chunk(world.clone(), ChunkPosition::new(x, y, z));
                }
            }
        }
        drop(worldrender);
        let size: PhysicalSize<i32> = window_size.into().to_physical(1.);
        Self {
            client,
            world,
            _engine: Some(engine),
            camera: Camera::new(45.5, size.width, size.height),
            render_manager,
        }
    }
    pub fn run(mut self) {
        self._engine.take().unwrap().run(self);
    }
}

fn move_camera(
    engine: &mut GameEngine<CubeGame>,
    window: &mut GameWindow,
    cube: &mut CubeGame,
    event: &Event<()>,
) {
    if let Event::MainEventsCleared = event {
        let SPEED: f32 = 3. * 1000.;
        let SPEED = SPEED * engine.delta_time().as_secs_f32();
        if engine.input.keyboard.is_pressed(VirtualKeyCode::W) {
            cube.camera.position -= cube.camera.direction() * SPEED;
        }
        if engine.input.keyboard.is_pressed(VirtualKeyCode::S) {
            cube.camera.position += cube.camera.direction() * SPEED;
        }
        if engine.input.keyboard.is_pressed(VirtualKeyCode::A) {
            cube.camera.position += cube.camera.left() * SPEED;
        }
        if engine.input.keyboard.is_pressed(VirtualKeyCode::D) {
            cube.camera.position += cube.camera.right() * SPEED;
        }
        const SENSITIVITY: f32 = 0.1;
        let mouse = &mut engine.input.mouse;
        let camera = &mut cube.camera;
        if mouse.updated && window.cursor_grabbed() {
            let x_offset = mouse.x - mouse.last_x;
            let y_offset = mouse.last_y - mouse.y;
            camera.yaw -= x_offset as f32 * SENSITIVITY;
            camera.pitch -= y_offset as f32 * SENSITIVITY;
            camera.pitch = camera.pitch.min(89.);
            camera.pitch = camera.pitch.max(-89.);
            mouse.updated = false;
        }
    }
}
