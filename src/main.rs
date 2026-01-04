mod button;
mod read_pdf;

use clap::Parser;
use std::sync::Arc;

use button::Button;
use gpui::{
    App, Application, Bounds, Context, Image, ImageFormat, ImageSource, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use hayro::RenderSettings;

use crate::read_pdf::Pdf;

struct PdfRenderer {
    pdf: read_pdf::Pdf,
    image: Arc<Image>,
    index: usize,
}

impl PdfRenderer {
    pub fn prev(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        if self.index == 1 {
            return;
        }
        self.index -= 1;
        self.image = pdf_to_image(&self.pdf, self.index - 1);
        window.refresh();
    }

    pub fn next(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        if self.index >= self.pdf.nth_pages() {
            return;
        }
        self.index += 1;
        self.image = pdf_to_image(&self.pdf, self.index - 1);
        window.refresh();
    }
}

impl Render for PdfRenderer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let source = ImageSource::Image(self.image.clone());
        let i = gpui::img(source);
        let nb_pages = self.pdf.nth_pages();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("prev", "<".into())
                            .on_click(_cx.listener(|this, _, win, cx| this.prev(win, cx))),
                    )
                    .child(format!("{} / {}", self.index, nb_pages))
                    .child(
                        Button::new("next", ">".into())
                            .on_click(_cx.listener(|this, _, win, cx| this.next(win, cx))),
                    ),
            )
            .child(div().child(i))
    }
}

fn pdf_to_image(pdf: &Pdf, index: usize) -> Arc<Image> {
    let scale = 1.3;
    let page = &pdf.pages()[index];
    let render_settings = RenderSettings {
        x_scale: scale,
        y_scale: scale,
        ..RenderSettings::default()
    };
    let png = read_pdf::render_page(pdf.interpreter_settings(), &render_settings, page);
    let image = Image::from_bytes(ImageFormat::Png, png);
    let image = Arc::new(image);
    image
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    name: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pdf = read_pdf::Pdf::from_file(&args.name)?;
    let image = pdf_to_image(&pdf, 0);

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(1200.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| PdfRenderer {
                    pdf,
                    image,
                    index: 1,
                })
            },
        )
        .unwrap();
    });
    Ok(())
}
