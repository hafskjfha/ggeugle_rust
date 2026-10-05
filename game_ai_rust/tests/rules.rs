use game_ai_rust::rules::{
    ChangeRule, FileWordsOption, ManualWordsOption, RuleForm, SelectedWordsOption, WordSource,
    dictionary_urls, get_head_tail, get_idx, load_words,
};

#[test]
fn standard_changes_keep_final_consonants_and_predecessor_order() {
    assert_eq!(ChangeRule(1).forward("량"), ["량", "양"]);
    assert_eq!(ChangeRule(1).forward("락"), ["락", "낙"]);
    assert_eq!(ChangeRule(1).forward("니"), ["니", "이"]);
    assert_eq!(ChangeRule(1).backward("영"), ["영", "녕", "령"]);
    assert_eq!(ChangeRule(1).forward("얘"), ["얘"]);
}

#[test]
fn forced_and_reverse_rules_exclude_only_transformable_identities() {
    assert_eq!(ChangeRule(2).forward("량"), ["양"]);
    assert!(ChangeRule(2).backward("량").is_empty());
    assert_eq!(ChangeRule(2).backward("영"), ["영", "녕", "령"]);
    assert_eq!(ChangeRule(3).forward("영"), ["영", "녕", "령"]);
    assert_eq!(ChangeRule(3).backward("량"), ["량", "양"]);
    assert_eq!(ChangeRule(4).forward("영"), ["녕", "령"]);
    assert!(ChangeRule(4).backward("영").is_empty());
    assert_eq!(ChangeRule(4).backward("량"), ["량", "양"]);
}

#[test]
fn freedom_rules_preserve_order_and_expand_both_consonants() {
    assert_eq!(ChangeRule(5).forward("름"), ["름", "늠", "음"]);
    assert_eq!(ChangeRule(5).backward("음"), ["음", "늠", "름"]);
    assert_eq!(ChangeRule(6).forward("니"), ["리", "니", "이"]);
    assert_eq!(
        ChangeRule(9).forward("량"),
        ["랼", "랸", "량", "냘", "냔", "냥", "얄", "얀", "양"]
    );
    assert_eq!(ChangeRule(9).backward("강"), ["갈", "간", "강"]);
}

#[test]
fn reversal_and_roblox_rules_keep_original_direction_quirks() {
    assert_eq!(ChangeRule(7).forward("량"), ["량", "양", "령"]);
    assert_eq!(ChangeRule(7).backward("량"), ["량", "양", "령"]);
    assert_eq!(ChangeRule(8).forward("강"), ["강", "악"]);
    assert_eq!(ChangeRule(8).backward("강"), ["강", "악"]);
    assert_eq!(ChangeRule(8).forward("밖"), ["밖", "깝"]);
    assert_eq!(ChangeRule(8).forward("각"), ["각"]);
    assert_eq!(ChangeRule(8).forward("값"), ["값"]);
    assert_eq!(ChangeRule(10).forward("름"), ["름", "늠", "음"]);
    assert_eq!(ChangeRule(10).backward("음"), ["음", "름"]);
    assert_eq!(ChangeRule(10).forward("뤠"), ["뤠", "웨"]);
}

#[test]
fn input_filter_removal_wins_over_additions_and_preserves_first_occurrence() {
    let mut rule = RuleForm::manual(&["사과", "과자", "사과", "a"], 0)
        .content
        .word_rule;
    rule.regex_filter = "[가-힣]{2}".into();
    rule.added_words = "자두 과자 사과".into();
    rule.removed_words = "과자".into();
    assert_eq!(load_words(&rule).unwrap(), ["사과", "자두"]);
    rule.regex_filter = "(?=사)사과".into();
    assert_eq!(load_words(&rule).unwrap(), ["사과"]);
    rule.regex_filter = "[".into();
    assert!(load_words(&rule).is_err());
}

#[test]
fn character_indices_are_one_based_from_either_end_and_validate_bounds() {
    assert_eq!(get_idx(1, 0).unwrap(), 0);
    assert_eq!(get_idx(2, 1).unwrap(), -2);
    assert_eq!(
        get_head_tail("가론강", 0, -1).unwrap(),
        ("가".into(), "강".into())
    );
    assert_eq!(
        get_head_tail("가론강", -2, 1).unwrap(),
        ("론".into(), "론".into())
    );
    assert!(get_idx(0, 0).is_err());
    assert!(get_idx(1, 2).is_err());
    assert!(get_head_tail("가론강", 3, -1).is_err());
}

#[test]
fn original_camel_case_rules_deserialize_into_rust_fields() {
    let rule: RuleForm = serde_json::from_str(r#"{"id":"test","metadata":{"title":"test","updatedAt":0,"color":""},"content":{"wordRule":{"words":{"type":"manual","option":{"content":"사과 과자"}},"regexFilter":".*","removedWords":"","addedWords":""},"wordConnectionRule":{"changeFuncIdx":1,"rawHeadIdx":1,"headDir":0,"rawTailIdx":1,"tailDir":1},"postprocessing":{"manner":{"type":3,"nextWordsLimit":2},"addedWords":"","removedWords":""}}}"#).unwrap();
    assert_eq!(rule.content.word_connection_rule.change_func_idx, 1);
    assert_eq!(rule.content.postprocessing.manner.next_words_limit, Some(2));
    assert_eq!(
        load_words(&rule.content.word_rule).unwrap(),
        ["사과", "과자"]
    );
}

#[test]
fn compatibility_jamo_endpoints_from_real_dictionary_survive_changes() {
    for rule in 0..11 {
        assert_eq!(ChangeRule(rule).forward("ㄷ"), ["ㄷ"]);
        assert_eq!(ChangeRule(rule).forward("ㅎ"), ["ㅎ"]);
    }
    assert_eq!(ChangeRule(7).forward("ㅏ"), ["ㅏ", "ㅓ"]);
    assert_eq!(ChangeRule(10).backward("ㅇ"), ["ㅇ", "ㄹ"]);
}

#[test]
fn file_source_reads_a_portable_real_dictionary_fixture() {
    let mut rule = RuleForm::manual(&[], 0).content.word_rule;
    rule.words = WordSource::File(FileWordsOption {
        path: format!("{}/tests/fixtures/words.txt", env!("CARGO_MANIFEST_DIR")),
    });
    let words = load_words(&rule).unwrap();
    assert!(words.len() >= 150);
    for required in [
        "사과",
        "과자",
        "자두",
        "가나",
        "가시나",
        "나니가",
        "나무바다",
        "다가",
        "가론강",
        "악어",
        "역량",
        "양력",
        "ㄷ형강",
        "삿ㅎ",
    ] {
        assert!(
            words.iter().any(|word| word == required),
            "missing {required}"
        );
    }
}

#[test]
fn selected_dictionary_urls_deduplicate_ignored_categories_and_keep_cross_product() {
    let mut option = SelectedWordsOption {
        dict: 0,
        pos: [("명사".into(), 1), ("부사".into(), 1)]
            .into_iter()
            .collect(),
        cate: [("일반어".into(), 1), ("방언".into(), 1)]
            .into_iter()
            .collect(),
    };
    assert_eq!(
        dictionary_urls(&option).unwrap(),
        [
            "https://singrum.github.io/KoreanDict/oldict/db/명사",
            "https://singrum.github.io/KoreanDict/oldict/db/부사"
        ]
    );
    option.dict = 2;
    assert_eq!(
        dictionary_urls(&option).unwrap(),
        [
            "https://singrum.github.io/KoreanDict/opendict/db/일반어/명사",
            "https://singrum.github.io/KoreanDict/opendict/db/방언/명사",
            "https://singrum.github.io/KoreanDict/opendict/db/일반어/부사",
            "https://singrum.github.io/KoreanDict/opendict/db/방언/부사"
        ]
    );
    option.dict = 9;
    assert_eq!(
        dictionary_urls(&option).unwrap(),
        ["https://singrum.github.io/KoreanDict/roblox"]
    );
    option.dict = 12;
    assert!(dictionary_urls(&option).is_err());
}

#[test]
fn built_in_presets_keep_connection_indices_filtering_and_removal_settings() {
    let presets = game_ai_rust::presets::sample_rules();
    let backwards = presets
        .iter()
        .find(|r| r.id == "앞말잇기")
        .expect("backwards preset");
    assert_eq!(
        get_idx(
            backwards.content.word_connection_rule.raw_head_idx,
            backwards.content.word_connection_rule.head_dir
        )
        .unwrap(),
        -1
    );
    assert_eq!(
        get_idx(
            backwards.content.word_connection_rule.raw_tail_idx,
            backwards.content.word_connection_rule.tail_dir
        )
        .unwrap(),
        0
    );
    let mut three = presets
        .iter()
        .find(|r| r.id == "천도룰")
        .expect("three-character preset")
        .content
        .word_rule
        .clone();
    three.words = WordSource::Manual(ManualWordsOption {
        content: "사과 가론강 가시나".into(),
    });
    assert_eq!(load_words(&three).unwrap(), ["가론강", "가시나"]);
    let no_rule = presets.iter().find(|r| r.id == "노룰").unwrap();
    assert_eq!(no_rule.content.word_connection_rule.change_func_idx, 0);
    assert_eq!(no_rule.content.postprocessing.manner.r#type, 2);
    let connected = presets.iter().find(|r| r.id == "연결룰").unwrap();
    assert_eq!(
        connected.content.word_rule.removed_words,
        "붕어톱 궤휼 잎뽕"
    );
}

#[test]
fn original_precedence_maps_and_dictionary_defaults_are_available() {
    let maps = game_ai_rust::presets::sample_precedence_maps();
    assert_eq!(
        maps.get("구엜룰").expect("old dictionary priorities").edge["송"]["욱"],
        -1.0
    );
    assert_eq!(maps["천도룰"].node["틴"], -10.0);
    assert_eq!(maps["신엜룰"].node["쇄"], -1.0);
    let dicts = game_ai_rust::presets::sample_dicts();
    let option = SelectedWordsOption {
        dict: 2,
        pos: dicts.get(2).expect("opendict defaults").default_pos.clone(),
        cate: dicts[2].default_cate.clone(),
    };
    assert_eq!(dictionary_urls(&option).unwrap().len(), 4);
    assert_eq!(
        game_ai_rust::presets::sample_change_func_info()
            .last()
            .unwrap()
            .title,
        "로블록스 한국 끝말잇기 두음법칙"
    );
}
