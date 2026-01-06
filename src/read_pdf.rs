use hayro::{RenderSettings, render};
use hayro_interpret::{
    InterpreterSettings,
    hayro_syntax::page::{Page, Pages},
};
use hayro_svg::convert;
use std::{path::Path, sync::Arc};

pub struct Pdf(hayro::Pdf, InterpreterSettings);

impl Pdf {
    pub fn interpreter_settings(&self) -> &InterpreterSettings {
        &self.1
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Pdf> {
        let data = std::fs::read(path)?;
        let data = Arc::new(data);
        let pdf = hayro::Pdf::new(data).map_err(|e| anyhow::anyhow!("pdf failed {:?}", e))?;
        let interpreter_settings = InterpreterSettings::default();

        Ok(Pdf(pdf, interpreter_settings))
    }

    pub fn nth_pages(&self) -> usize {
        self.0.pages().len()
    }

    pub fn pages<'a>(&'a self) -> &'a Pages<'a> {
        self.0.pages()
    }
}

pub fn render_page_png<'a>(
    interpreter_settings: &InterpreterSettings,
    render_settings: &RenderSettings,
    page: &Page<'a>,
) -> Vec<u8> {
    let pixmap = render(page, interpreter_settings, render_settings);
    pixmap.take_png()
}

pub fn render_page_svg<'a>(interpreter_settings: &InterpreterSettings, page: &Page<'a>) -> Vec<u8> {
    convert(page, interpreter_settings).as_bytes().to_vec()
}
