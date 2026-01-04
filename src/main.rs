mod button;
mod read_pdf;

use std::{ops::DerefMut, sync::Arc};

use button::Button;
use gpui::{
    App, Application, Bounds, Context, Image, ImageFormat, ImageSource, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use hayro::RenderSettings;

use crate::read_pdf::Pdf;

struct PdfRenderer {
    pdf: Option<read_pdf::Pdf>,
    image: Option<Arc<Image>>,
    index: usize,
}

impl PdfRenderer {
    pub fn prev(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        let Some(pdf) = &self.pdf else {
            return;
        };

        if self.index == 1 {
            return;
        }
        self.index -= 1;
        self.image = Some(pdf_to_image(pdf, self.index - 1));
        window.refresh();
    }

    pub fn next(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        let Some(pdf) = &self.pdf else {
            return;
        };

        if self.index >= pdf.nth_pages() {
            return;
        }
        self.index += 1;
        self.image = Some(pdf_to_image(pdf, self.index - 1));
        window.refresh();
    }

    pub fn set_file(&mut self, path: std::path::PathBuf, cx: &mut Context<Self>) {
        println!("Selected file: {:?}", path);
        let pdf = read_pdf::Pdf::from_file(path).unwrap();
        self.image = Some(pdf_to_image(&pdf, self.index - 1));
        self.pdf = Some(pdf);
        self.index = 1;
        cx.refresh_windows();
    }
}

fn prompt_for_file() -> Option<std::path::PathBuf> {
    // Note: rfd methods are blocking, so this need to be spawn in a background task to remain responsive
    rfd::FileDialog::new()
        .set_title("Select a file to open")
        .add_filter("Pdf Files", &["pdf"])
        .add_filter("All Files", &["*"])
        .pick_file()
}

impl Render for PdfRenderer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let nb_pages = if let Some(pdf) = &self.pdf {
            pdf.nth_pages()
        } else {
            0
        };
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
                        Button::new("open-file-button", "Open File".into()).on_click(_cx.listener(
                            |_this, _event, _win, cx| {
                                let this = cx.weak_entity();
                                let app = cx.deref_mut();
                                app.spawn(async move |cx| {
                                    if let Some(path) = prompt_for_file() {
                                        this.update(cx, |this, cx| this.set_file(path, cx))
                                            .unwrap();
                                    } else {
                                        println!("Dialog cancelled");
                                    }
                                })
                                .detach();
                            },
                        )),
                    )
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
            .when_some(self.image.clone(), |this, image| {
                let source = ImageSource::Image(image);
                let i = gpui::img(source);
                this.child(div().child(i))
            })
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

fn main() -> anyhow::Result<()> {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(1200.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| PdfRenderer {
                    pdf: None,
                    image: None,
                    index: 1,
                })
            },
        )
        .unwrap();
    });
    Ok(())
}
