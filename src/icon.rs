use std::{fs::remove_file, path::PathBuf};

use anyhow::Context;
use log::debug;
use rust_embed::RustEmbed;

use crate::{config::IconStyle, AppState};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/icons"]
struct Icons;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/fonts"]
struct Fonts;


pub fn build_icon(style: IconStyle, name: &str, app_state: &AppState) -> anyhow::Result<PathBuf> {
    if !app_state.project_dirs.cache_dir().exists() {
        std::fs::create_dir_all(app_state.project_dirs.cache_dir())?;
    }
    if !app_state.project_dirs.data_dir().exists() {
        std::fs::create_dir_all(app_state.project_dirs.data_dir())?;
    }
    let icon_file = match style {
        IconStyle::Default => Icons::get("default.svg").expect("Missing ICON"),
        IconStyle::Cargo => Icons::get("default.svg").expect("Missing ICON"),
    };
    // Get First letter of name
    let letter = name
        .chars()
        .next()
        .unwrap_or('?')
        .to_ascii_uppercase()
        .to_string();
    let cached_svg_icon = app_state.project_dirs.cache_dir().join(format!(
        "{}-{}.svg",
        letter,
        AsRef::<str>::as_ref(&style)
    ));
    let svg = if !cached_svg_icon.exists() {
        let icon_file = String::from_utf8(icon_file.data.to_vec()).context("Invalid icon file")?;
        let icon_file = icon_file.replace(r#"{{INITIAL}}"#, letter.as_str());
        std::fs::write(&cached_svg_icon, &icon_file)?;
        icon_file
    } else {
        debug!("Found cached icon at {:?}", cached_svg_icon);
        std::fs::read_to_string(&cached_svg_icon).context("Unable to read cached icon")?
    };
    let ico_path = app_state
        .project_dirs
        .data_dir()
        .join(format!("{}.ico", name));
    if ico_path.exists() {
        remove_file(&ico_path)?;
    }
    let icon = ico::svg_to_ico(svg)?;
    std::fs::write(&ico_path, icon)?;
    Ok(ico_path)
}

mod ico {
    use anyhow::{Context, Result};
    use once_cell::sync::Lazy;
    use tiny_skia::{IntSize, Pixmap};
    use usvg::{fontdb::Database, TreeParsing, TreeTextToPath};

    use super::Fonts;
    pub(super) fn load_fonts() -> Database {
        let mut fontdb = Database::new();
        for file in Fonts::iter() {
            if file.ends_with(".ttf") {
                let file = Fonts::get(file.as_ref()).unwrap().data.to_vec();
                fontdb.load_font_data(file);
            }
        }

        fontdb
    }
    static FONTDB: Lazy<Database> = once_cell::sync::Lazy::new(|| load_fonts());
    /// Convert SVG to ICO
    pub(super) fn svg_to_ico(svg: String) -> Result<Vec<u8>> {
        let opt = usvg::Options {
            dpi: 150f32,
            ..Default::default()
        };

        let mut svg = usvg::Tree::from_str(&svg, &opt)?;

        svg.convert_text(&FONTDB);
        let result = rasterize(&svg, 256)?;
        let ico = create_ico(result)?;
        Ok(ico)
    }

    fn rasterize(svg: &usvg::Tree, height_in_pixels: u32) -> Result<tiny_skia::Pixmap> {
        let target_size = usvg::Size::from_wh(height_in_pixels as f32, height_in_pixels as f32)
            .context("Unsigned values should always be valid")?;

        let pixmap_size =
            IntSize::from_wh(height_in_pixels, height_in_pixels).context("Invalid Pixmap Size")?;

        let sx = f64::from(pixmap_size.width()) / svg.size.width() as f64;
        let sy = f64::from(pixmap_size.height()) / svg.size.height() as f64;
        let transform = tiny_skia::Transform::from_scale(sx as f32, sy as f32);

        tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .map(|mut pixmap| {
                let mut pixmap_mut = pixmap.as_mut();
                resvg::Tree::from_usvg(svg).render(transform, &mut pixmap_mut);
                pixmap
            })
            .context("Unable to create pixmap")
    }

    fn create_ico(png: Pixmap) -> Result<Vec<u8>> {
        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
        let image = ico::IconImage::from_rgba_data(png.width(), png.height(), png.take());
        icon_dir.add_entry(ico::IconDirEntry::encode(&image)?);
        let mut buf = Vec::new();
        icon_dir.write(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::Context;
    use rust_embed::RustEmbed;
    use usvg::fontdb::{Database, Family, Weight};

    use super::Icons;

    #[test]
    fn load_fonts() {
        let fonts = super::ico::load_fonts();
        fonts.faces().for_each(|face| {
            println!("Font families: {:?}", face.families);
            println!("Font Style: {:?}", face.style);
            println!("Font Weight: {:?}", face.weight);
            println!("Font Stretch: {:?}", face.stretch);
            println!("Font post_script_name: {:?}", face.post_script_name);
        });
        let font = fonts
            .query(&usvg::fontdb::Query {
                families: &[Family::Name("Fira Sans")],
                style: usvg::fontdb::Style::Italic,
                weight: Weight(900),
                stretch: usvg::fontdb::Stretch::Normal,
            })
            .unwrap();
        let font = fonts.face(font).unwrap();
        assert_eq!(font.post_script_name, "FiraSans-Black");
    }
    #[test]
    fn test() {
        std::env::set_var("RUST_LOG", "TRACE");
        pretty_env_logger::init();
        let image_tests_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("image_tests");
        if !image_tests_directory.exists() {
            std::fs::create_dir_all(&image_tests_directory).unwrap();
        }
        for ele in <Icons as RustEmbed>::iter() {
            let name = ele.as_ref();
            if !name.ends_with(".svg") {
                continue;
            }
            let data = Icons::get(name).unwrap();
            let string = String::from_utf8(data.data.to_vec()).unwrap();
            // Iterator A-Z
            for letter in 'A'..='Z' {
                let letter = letter as char;
                let svg = string.replace(r#"{{INITIAL}}"#, letter.to_string().as_str());
                let path = image_tests_directory.join(format!("{}-{}.ico", letter, name));
                if path.exists() {
                    std::fs::remove_file(&path).unwrap();
                }
                let icon = super::ico::svg_to_ico(svg).expect("Unable to convert SVG to ICO");
                std::fs::write(&path, icon).expect("Unable to write icon");
            }
        }
    }
}
