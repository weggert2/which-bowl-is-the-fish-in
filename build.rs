use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_assets = manifest_directory.join("assets");

    println!("cargo:rerun-if-changed=build.rs");
    emit_asset_rerun_directives(&source_assets)?;

    // OUT_DIR is target/<profile>/build/<package-hash>/out. Three parents up is
    // target/<profile>, the directory that contains Cargo's executable.
    let out_directory = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_profile_directory = out_directory
        .ancestors()
        .nth(3)
        .expect("Cargo OUT_DIR did not have the expected target/profile layout");
    let destination_assets = target_profile_directory.join("assets");

    copy_directory(&source_assets, &destination_assets)?;
    let shipped_prompt_catalog = destination_assets.join("CODEX_PROMPTS.md");
    if shipped_prompt_catalog.exists() {
        fs::remove_file(shipped_prompt_catalog)?;
    }
    generate_display_lids(&source_assets, &destination_assets)?;
    let source_icon = source_assets.join("app_icon.png");
    generate_window_icon_source(&source_icon, &out_directory.join("window_icon.rs"))?;

    #[cfg(windows)]
    {
        let generated_ico = out_directory.join("app_icon.ico");
        generate_windows_ico(&source_icon, &generated_ico)?;
        embed_windows_icon(&generated_ico)?;
    }

    Ok(())
}

/// Generate the exact RGBA arrays miniquad needs for window/taskbar icons.
fn generate_window_icon_source(
    source_icon: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = image::open(source_icon)?;
    let mut generated = String::new();

    for size in [16, 32, 64] {
        let rgba = source
            .resize_exact(size, size, image::imageops::FilterType::Lanczos3)
            .to_rgba8()
            .into_raw();
        writeln!(
            generated,
            "pub const APP_ICON_{size}: [u8; {}] = [",
            rgba.len()
        )?;
        for chunk in rgba.chunks(16) {
            generated.push_str("    ");
            for byte in chunk {
                write!(generated, "{byte}, ")?;
            }
            generated.push('\n');
        }
        generated.push_str("];\n");
    }

    fs::write(destination, generated)?;
    Ok(())
}

/// Explorer reads the executable resource icon. Use opaque BMP/DIB frames for
/// broad shell compatibility, while miniquad retains the transparent PNG data.
#[cfg(windows)]
fn generate_windows_ico(
    source_icon: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = image::open(source_icon)?;
    let mut icon_directory = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16, 32, 64, 128, 256] {
        let rgba = opaque_icon_frame(&source, size);
        let icon_image = ico::IconImage::from_rgba_data(size, size, rgba);
        icon_directory.add_entry(ico::IconDirEntry::encode_as_bmp(&icon_image)?);
    }

    let icon_file = fs::File::create(destination)?;
    icon_directory.write(icon_file)?;
    Ok(())
}

/// Composite transparent PNG pixels over a calm light-blue app background.
/// This avoids black transparent regions in Explorer's resource-icon renderer.
#[cfg(windows)]
fn opaque_icon_frame(source: &image::DynamicImage, size: u32) -> Vec<u8> {
    const BACKGROUND: [u8; 3] = [190, 224, 241];
    let foreground = source
        .resize_exact(size, size, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let mut opaque = Vec::with_capacity((size * size * 4) as usize);

    for pixel in foreground.as_raw().chunks_exact(4) {
        let alpha = pixel[3] as u16;
        let inverse_alpha = 255 - alpha;
        for channel in 0..3 {
            let blended =
                (pixel[channel] as u16 * alpha + BACKGROUND[channel] as u16 * inverse_alpha + 127)
                    / 255;
            opaque.push(blended as u8);
        }
        opaque.push(255);
    }

    opaque
}

/// Pre-filter large source lid art once at build time. The game then renders
/// these 150px textures at 1:1, avoiding subpixel sampling shimmer while lids
/// move during hover and reveal animations.
fn generate_display_lids(
    source_assets: &Path,
    destination_assets: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    const LIDS: [&str; 3] = ["lid1", "lid2", "lid3"];
    let source_bowls = source_assets.join("bowls");
    let destination_bowls = destination_assets.join("bowls");

    for lid_name in LIDS {
        let source = source_bowls.join(format!("{lid_name}.png"));
        let destination = destination_bowls.join(format!("{lid_name}_display.png"));
        let lid = image::open(source)?;
        let display_lid = lid.resize_exact(150, 150, image::imageops::FilterType::Lanczos3);
        display_lid.save(destination)?;
    }

    Ok(())
}

#[cfg(windows)]
fn embed_windows_icon(icon_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut resource = winres::WindowsResource::new();
    resource.set_icon(
        icon_path
            .to_str()
            .expect("Windows icon path was not valid UTF-8"),
    );
    resource.compile()?;
    Ok(())
}

fn emit_asset_rerun_directives(directory: &Path) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            emit_asset_rerun_directives(&path)?;
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else if source_path.file_name().and_then(|name| name.to_str()) == Some("CODEX_PROMPTS.md")
        {
            // Prompt catalog is repository authoring material, not a runtime
            // asset that should be shipped beside the game executable.
            continue;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}
