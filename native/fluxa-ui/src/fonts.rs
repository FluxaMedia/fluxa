use egui::{FontData, FontDefinitions, FontFamily, FontId};

const REGULAR: &str = "archivo-regular";
const SEMIBOLD: &str = "archivo-semibold";

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(REGULAR.into()))
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
    for (name, first) in [("archivo", SEMIBOLD), (REGULAR, REGULAR)] {
        let mut family = vec![first.to_owned()];
        family.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }
    fonts
}
