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
    let mut scripts = Vec::new();
    for (weight, arabic, hebrew) in [
        (
            "regular",
            &include_bytes!("../fonts/noto_sans_arabic_regular.ttf")[..],
            &include_bytes!("../fonts/noto_sans_hebrew_regular.ttf")[..],
        ),
        (
            "medium",
            &include_bytes!("../fonts/noto_sans_arabic_medium.ttf")[..],
            &include_bytes!("../fonts/noto_sans_hebrew_medium.ttf")[..],
        ),
        (
            "bold",
            &include_bytes!("../fonts/noto_sans_arabic_bold.ttf")[..],
            &include_bytes!("../fonts/noto_sans_hebrew_bold.ttf")[..],
        ),
    ] {
        let arabic_name = format!("noto-arabic-{weight}");
        let hebrew_name = format!("noto-hebrew-{weight}");
        fonts
            .font_data
            .insert(arabic_name.clone(), FontData::from_static(arabic).into());
        fonts
            .font_data
            .insert(hebrew_name.clone(), FontData::from_static(hebrew).into());
        scripts.push((weight, [arabic_name, hebrew_name]));
    }
    let script_fonts = |weight: &str| -> Vec<String> {
        scripts
            .iter()
            .find(|(name, _)| *name == weight)
            .map(|(_, names)| names.to_vec())
            .unwrap_or_default()
    };
    let fallbacks = fonts.families[&FontFamily::Proportional].clone();
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, SEMIBOLD.to_owned());
    for (name, first, weight) in [
        ("archivo", SEMIBOLD, "medium"),
        (REGULAR, REGULAR, "regular"),
        (MEDIUM, MEDIUM, "medium"),
        (BOLD, BOLD, "bold"),
    ] {
        let mut family = vec![first.to_owned()];
        family.extend(script_fonts(weight));
        family.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }
    add_system_fallbacks(&mut fonts);
    fonts
}

#[cfg(target_os = "android")]
fn add_system_fallbacks(fonts: &mut FontDefinitions) {
    const FILES: &[&str] = &[
        "NotoSansCJK-Regular.ttc",
        "SECCJK-Regular.ttc",
        "NotoSansThai-Regular.ttf",
        "NotoSansDevanagari-VF.ttf",
        "NotoSansBengali-VF.ttf",
        "NotoSansTamil-VF.ttf",
        "NotoSansTelugu-VF.ttf",
        "NotoSansKannada-VF.ttf",
        "NotoSansMalayalam-VF.ttf",
        "NotoSansGujarati-Regular.ttf",
        "NotoSansGurmukhi-VF.ttf",
        "NotoSansGeorgian-VF.ttf",
        "NotoSansArmenian-VF.ttf",
        "NotoSansKhmer-VF.ttf",
        "NotoSansLao-Regular.ttf",
        "NotoSansMyanmar-Regular.ttf",
        "NotoSansSinhala-VF.ttf",
        "NotoSansEthiopic-VF.ttf",
    ];
    static MAPPED: std::sync::OnceLock<Vec<(String, &'static [u8])>> = std::sync::OnceLock::new();
    let mapped = MAPPED.get_or_init(|| {
        let mut found = Vec::new();
        let mut has_cjk = false;
        for file in FILES {
            let cjk = file.contains("CJK");
            if cjk && has_cjk {
                continue;
            }
            let Ok(handle) = std::fs::File::open(format!("/system/fonts/{file}")) else {
                continue;
            };
            let Ok(map) = (unsafe { memmap2::Mmap::map(&handle) }) else {
                continue;
            };
            let bytes: &'static [u8] = Box::leak(Box::new(map));
            has_cjk |= cjk;
            found.push((format!("system-{file}"), bytes));
        }
        found
    });
    for (name, bytes) in mapped {
        fonts
            .font_data
            .insert(name.clone(), FontData::from_static(bytes).into());
    }
    for family in fonts.families.values_mut() {
        family.extend(mapped.iter().map(|(name, _)| name.clone()));
    }
}

#[cfg(not(target_os = "android"))]
fn add_system_fallbacks(_fonts: &mut FontDefinitions) {}
