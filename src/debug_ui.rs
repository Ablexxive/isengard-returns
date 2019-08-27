pub use imgui::im_str;

use gfx_core::memory::Typed;
use ggez::*;
use ggez::event::winit_event::Event;
use imgui::{Context as ImguiContext, Io, Ui};
use imgui_gfx_renderer::{Renderer as ImguiRenderer, Shaders};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

pub struct DebugUi {
    imgui: ImguiContext,
    platform: WinitPlatform,
    renderer: ImguiRenderer<gfx::format::Rgba8, gfx_device_gl::Resources>,
}

impl DebugUi {
    pub fn new(ctx: &mut Context) -> Self {
        // NOTE: This init code is based on:
        // https://github.com/Gekkio/imgui-rs/blob/v0.1.0/imgui-gfx-examples/examples/support/mod.rs

        // Initialize Dear Imgui and its GFX renderer.
        let mut imgui = ImguiContext::create();
        // Convert Imgui's style colors from sRGB to linear since we're using an sRGB framebuffer.
        // NOTE: Make sure to do this after loading any themes.
        {
            fn imgui_gamma_to_linear(col: [f32; 4]) -> [f32; 4] {
                let x = col[0].powf(2.2);
                let y = col[1].powf(2.2);
                let z = col[2].powf(2.2);
                let w = 1.0 - (1.0 - col[3]).powf(2.2);
                [x, y, z, w]
            }

            let style = imgui.style_mut();
            for col in 0..style.colors.len() {
                style.colors[col] = imgui_gamma_to_linear(style.colors[col]);
            }
        }

        // TODO: Set imgui.ini file path.
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), graphics::window(ctx), HiDpiMode::Rounded);
        let renderer = {
            let (factory, device, _encoder, _depth_stencil_view, _render_target_view) =
                graphics::gfx_objects(ctx);
            let version = device.get_info().shading_language;
            let shaders = if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                if version.minor >= 2 {
                    Shaders::GlSl150
                } else {
                    Shaders::GlSl130
                }
            } else {
                Shaders::GlSl110
            };
            ImguiRenderer::init(&mut imgui, factory, shaders)
                .expect("Could not initialize imgui_gfx_renderer::Renderer")
        };

        Self {
            imgui,
            platform,
            renderer,
        }
    }

    pub fn handle_event(&mut self, ctx: &Context, event: &Event) {
        self.platform.handle_event(self.imgui.io_mut(), graphics::window(ctx), event);
    }

    pub fn prepare_frame(&mut self, ctx: &Context) -> GameResult {
        self.platform.prepare_frame(self.imgui.io_mut(), graphics::window(ctx))
            .map_err(|err| GameError::WindowError(err))?;
        // NOTE: Setting delta time directly instead of calling io_mut().update_delta_time().
        self.imgui.io_mut().delta_time = timer::duration_to_f64(timer::delta(ctx)) as f32;

        Ok(())
    }

    pub fn draw_ui<F>(&mut self, ctx: &mut Context, func: F)
        where F: FnOnce(&Ui)
    {
        // Build our Dear Imgui UI elements.
        let ui = self.imgui.frame();
        func(&ui);

        // Render our Dear Imgui UI elements.
        self.platform.prepare_render(&ui, graphics::window(ctx));
        let ui_draw_data = ui.render();
        {
            let (factory, _device, encoder, _depth_stencil_view, render_target_view) =
                graphics::gfx_objects(ctx);
            let mut target = gfx_core::handle::RenderTargetView::new(render_target_view);
            self.renderer.render(factory, encoder, &mut target, ui_draw_data)
                .expect("Could not render Dear Imgui UI");
        }
    }

    pub fn io(&self) -> &Io {
        self.imgui.io()
    }
}
