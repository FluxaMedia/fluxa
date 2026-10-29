use fluxa_streamkit::release::parse;

#[test]
fn episode_release_with_group() {
    let r = parse("sons.of.anarchy.s05e10.480p.BluRay.x264-GAnGSteR");
    assert_eq!(r.title.as_deref(), Some("sons of anarchy"));
    assert_eq!(r.seasons, vec![5]);
    assert_eq!(r.episodes, vec![10]);
    assert_eq!(r.resolution.as_deref(), Some("480p"));
    assert_eq!(r.quality.as_deref(), Some("BluRay"));
    assert_eq!(r.codec.as_deref(), Some("x264"));
    assert_eq!(r.group.as_deref(), Some("GAnGSteR"));
}

#[test]
fn movie_with_quality_only() {
    let r = parse("Da Vinci Code DVDRip");
    assert_eq!(r.title.as_deref(), Some("Da Vinci Code"));
    assert_eq!(r.quality.as_deref(), Some("DVDRip"));
}

#[test]
fn year_and_group() {
    let r = parse("Rogue One 2016 1080p BluRay x264-SPARKS");
    assert_eq!(r.title.as_deref(), Some("Rogue One"));
    assert_eq!(r.year.as_deref(), Some("2016"));
    assert_eq!(r.resolution.as_deref(), Some("1080p"));
    assert_eq!(r.group.as_deref(), Some("SPARKS"));
}

#[test]
fn year_inside_title_is_not_the_release_year() {
    let r = parse("2019 After The Fall Of New York 1983 REMASTERED BDRip x264-GHOULS");
    assert_eq!(r.title.as_deref(), Some("2019 After The Fall Of New York"));
    assert_eq!(r.year.as_deref(), Some("1983"));
    assert_eq!(r.editions, vec!["Remastered"]);
    assert_eq!(r.group.as_deref(), Some("GHOULS"));
}

#[test]
fn complete_season_in_brackets() {
    let r = parse("Game of Thrones - The Complete Season 3 [HDTV]");
    assert_eq!(r.title.as_deref(), Some("Game of Thrones"));
    assert_eq!(r.seasons, vec![3]);
    assert_eq!(r.quality.as_deref(), Some("HDTV"));
}

#[test]
fn season_list_with_ampersand() {
    let r = parse("The Sopranos: The Complete Series (Season 1,2,3,4,5&6) + Extras");
    assert_eq!(r.title.as_deref(), Some("The Sopranos"));
    assert_eq!(r.seasons, vec![1, 2, 3, 4, 5, 6]);
    assert!(r.complete);
}

#[test]
fn season_range() {
    let r = parse("Skins Season S01-S07 COMPLETE UK Soundtrack 720p WEB-DL");
    assert_eq!(r.seasons, (1..=7).collect::<Vec<_>>());
    assert!(r.complete);
    assert_eq!(r.resolution.as_deref(), Some("720p"));
}

#[test]
fn daily_show_date() {
    let r = parse("Stephen Colbert 2019 10 25 Eddie Murphy 480p x264-mSD [example]");
    assert_eq!(r.title.as_deref(), Some("Stephen Colbert"));
    assert_eq!(r.date.as_deref(), Some("2019-10-25"));
}

#[test]
fn anime_with_leading_group_and_code() {
    let r = parse("[Commie] Chihayafuru 3 - 21 [BD 720p AAC] [5F1911ED].mkv");
    assert_eq!(r.title.as_deref(), Some("Chihayafuru 3"));
    assert_eq!(r.episodes, vec![21]);
    assert_eq!(r.resolution.as_deref(), Some("720p"));
    assert_eq!(r.episode_code.as_deref(), Some("5F1911ED"));
    assert_eq!(r.group.as_deref(), Some("Commie"));
}

#[test]
fn anime_sxxexx_with_multi_subs() {
    let r = parse("[Erai-raws] Attack on Titan - S04E01 [1080p][Multiple Subtitle].mkv");
    assert_eq!(r.title.as_deref(), Some("Attack on Titan"));
    assert_eq!(r.seasons, vec![4]);
    assert_eq!(r.episodes, vec![1]);
    assert!(r.subbed);
    assert_eq!(r.group.as_deref(), Some("Erai-raws"));
}

#[test]
fn hdr_remux_style_name() {
    let r = parse("Dune.2021.2160p.UHD.BluRay.HDR10.DV.x265.DTS-HD.MA.5.1");
    assert_eq!(r.title.as_deref(), Some("Dune"));
    assert_eq!(r.year.as_deref(), Some("2021"));
    assert!(r.hdr.contains(&"DV".to_string()));
    assert_eq!(r.channels, vec!["5.1"]);
}
