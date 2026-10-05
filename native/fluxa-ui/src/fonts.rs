use egui::{FontData, FontDefinitions, FontFamily, FontId};

const REGULAR: &str = "archivo-regular";
const MEDIUM: &str = "archivo-medium";
const SEMIBOLD: &str = "archivo-semibold";
const BOLD: &str = "archivo-bold";

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(REGULAR.into()))
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MEDIUM.into()))
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}

pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        SEMIBOLD.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../apps/android/app/src/main/res/font/archivo_semibold.ttf"
        ))
        .into(),
    );
    for (name, bytes) in [
        (
            MEDIUM,
            &include_bytes!("../../../apps/android/app/src/main/res/font/archivo_medium.ttf")[..],
        ),
        (
            BOLD,
            &include_bytes!("../../../apps/android/app/src/main/res/font/archivo_bold.ttf")[..],
        ),
    ] {
        fonts
            .font_data
            .insert(name.to_owned(), FontData::from_static(bytes).into());
    }
    fonts.font_data.insert(
        REGULAR.to_owned(),
        FontData::from_static(include_bytes!(
            "../../../apps/android/app/src/main/res/font/archivo_regular.ttf"
        ))
        .into(),
    );
    let fallbacks = fonts.families[&FontFamily::Proportional].clone();
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, SEMIBOLD.to_owned());
    for (name, first) in [
        ("archivo", SEMIBOLD),
        (REGULAR, REGULAR),
        (MEDIUM, MEDIUM),
        (BOLD, BOLD),
    ] {
        let mut family = vec![first.to_owned()];
        family.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }
    fonts
}
