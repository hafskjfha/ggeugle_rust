use game_ai_rust::{ai::*, engine::*, rules::*, types::*};
use std::time::Duration;

// All dictionary words in these structural cases were extracted from all_words.txt.
fn rule(words: &[&str], change: usize) -> RuleForm {
    serde_json::from_value(serde_json::json!({
        "id":"test", "metadata":{"title":"test","color":"","updatedAt":0},
        "content": {
            "wordRule": {"words":{"type":"manual","option":{"content":words.join(" ")}},
                "regexFilter":".*", "addedWords":"", "removedWords":""},
            "wordConnectionRule": {"changeFuncIdx":change,"rawHeadIdx":1,"headDir":0,"rawTailIdx":1,"tailDir":1},
            "postprocessing":{"manner":{"type":0},"addedWords":"","removedWords":""}
        }
    })).unwrap()
}
fn history(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| s.to_string()).collect()
}

#[test]
fn history_reclassification_keeps_original_partition_order_for_optimal_ties() {
    // Same edge structure as the review repro; every word here occurs in all_words.txt.
    let words = [
        "가가",
        "가나",
        "가시나",
        "가이아나",
        "나니가",
        "나락가",
        "나례가",
        "나가르주나",
        "나가세나",
        "나라",
        "나바라",
        "나찰나라",
        "다이아나",
        "라가",
        "라마야나",
        "라마크리슈나",
        "라다",
    ];
    let solver = get_wc_data(&rule(&words, 0), 0).unwrap();
    let used = history(&["나라"]);
    let expected = Some(("라".to_string(), "가".to_string()));
    assert_eq!(
        solver
            .after_history(&used, 0)
            .unwrap()
            .get_losing_optimal_move(0, "라", None),
        expected
    );
    assert_eq!(
        update_solver(
            &solver.graph_solver.graphs,
            &[("나".into(), "라".into(), 1)],
            0
        )
        .unwrap()
        .get_losing_optimal_move(0, "라", None),
        expected
    );
    assert_eq!(
        choose_move(
            &solver,
            &used,
            &AiOptions {
                difficulty: 1,
                ..AiOptions::default()
            }
        )
        .unwrap(),
        Some("라가".to_string())
    );
}

#[test]
fn opening_without_route_chooses_first_word_on_equal_maximum_depth() {
    let solver = get_wc_data(&rule(&["사과", "자두"], 0), 0).unwrap();
    for difficulty in [1, 2] {
        assert_eq!(
            choose_move(
                &solver,
                &[],
                &AiOptions {
                    difficulty,
                    ..AiOptions::default()
                }
            )
            .unwrap(),
            Some("사과".to_string())
        );
    }
}

#[test]
fn exact_words_are_consumed_individually_and_stealing_does_not_consume_twice() {
    let solver = get_wc_data(&rule(&["가나", "가시나", "나니가"], 0), 0).unwrap();
    assert_eq!(
        solver
            .get_next_words(&history(&["가나", "나니가"]))
            .unwrap(),
        history(&["가시나"])
    );
    assert!(!is_game_end(&solver, &history(&["가나", "나니가"]), false).unwrap());
    assert!(is_game_end(&solver, &history(&["가나", "나니가", "가시나"]), true).unwrap());
    let updated = solver
        .after_history(&history(&["가나", "가나"]), 0)
        .unwrap();
    assert_eq!(updated.graphs.union().get_edge_num("가", "나"), 1);
    assert_eq!(solver.word_map.get_size(), 3);
}

#[test]
fn search_reports_the_movers_outcome_and_does_not_mutate_the_source() {
    let solver = get_wc_data(&rule(&["사과", "과자", "자두"], 0), 0).unwrap();
    let graph = solver.graph_solver.graphs.union();
    let first = search_is_win(
        &graph,
        &("사".into(), "과".into()),
        &PrecInfo::default(),
        None,
    )
    .unwrap();
    let second = search_is_win(
        &graph,
        &("과".into(), "자".into()),
        &PrecInfo::default(),
        None,
    )
    .unwrap();
    assert!(first.is_win);
    assert!(!second.is_win);
    assert_eq!(graph.get_edge_num("사", "과"), 1);
}

#[test]
fn streaming_stack_snapshots_survive_search_completion() {
    let solver = get_wc_data(&rule(&["사과", "과자", "자두"], 0), 0).unwrap();
    let mut events = Vec::new();
    start_streaming_single_thread_search(
        &solver.graph_solver.graphs.union(),
        &("사".into(), "과".into()),
        &PrecInfo::default(),
        None,
        |e| events.push(e),
    )
    .unwrap();
    assert!(
        matches!(&events[0], SearchEvent::Stack { payload } if payload == &vec![("사".into(), "과".into())])
    );
    assert!(matches!(events.last().unwrap(), SearchEvent::Done { payload } if payload.is_win));
}

#[test]
fn deadline_is_checked_before_even_an_easy_search() {
    let solver = get_wc_data(&rule(&["사과"], 0), 0).unwrap();
    let error = search_is_win(
        &solver.graph_solver.graphs.union(),
        &("사".into(), "과".into()),
        &PrecInfo::default(),
        Some(Duration::ZERO),
    )
    .unwrap_err();
    assert!(matches!(error, game_ai_rust::Error::Timeout));
    assert!(
        search_is_win(
            &solver.graph_solver.graphs.union(),
            &("사".into(), "과".into()),
            &PrecInfo::default(),
            None
        )
        .unwrap()
        .is_win
    );
}

#[test]
fn every_difficulty_selects_the_only_legal_remaining_word() {
    let solver = get_wc_data(&rule(&["사과", "과자", "자두"], 0), 0).unwrap();
    for difficulty in 0..=2 {
        let mut events = Vec::new();
        let options = AiOptions {
            difficulty,
            ..AiOptions::default()
        };
        assert_eq!(
            choose_move_with_callback(&solver, &history(&["사과"]), &options, |e| events.push(e))
                .unwrap(),
            Some("과자".into())
        );
        assert!(matches!(events.last(), Some(GameEvent::MessageEnd)));
    }
}

#[test]
fn hard_search_timeout_still_chooses_a_legal_cycle_word() {
    let solver = get_wc_data(&rule(&["가나", "나무바다", "다가"], 0), 0).unwrap();
    let options = AiOptions {
        difficulty: 2,
        calculating_duration: Duration::ZERO,
        ..AiOptions::default()
    };
    let selected = choose_move(&solver, &[], &options).unwrap().unwrap();
    assert!(["가나", "나무바다", "다가"].contains(&selected.as_str()));
    let selected = choose_move(&solver, &history(&["가나"]), &options)
        .unwrap()
        .unwrap();
    assert_eq!(selected, "나무바다");
}

#[test]
fn first_word_can_be_stolen_and_emits_terminal_win_events() {
    let solver = get_wc_data(&rule(&["사과"], 0), 0).unwrap();
    assert!(is_game_end(&solver, &history(&["사과"]), false).unwrap());
    assert!(!is_game_end(&solver, &history(&["사과"]), true).unwrap());
    let options = AiOptions {
        difficulty: 1,
        stealable: true,
        ..AiOptions::default()
    };
    let mut events = Vec::new();
    assert_eq!(
        choose_move_with_callback(&solver, &history(&["사과"]), &options, |e| events.push(e))
            .unwrap(),
        Some("사과".into())
    );
    let actions: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::Move { .. } => Some("move"),
            GameEvent::ComputerWin => Some("computerWin"),
            GameEvent::MessageEnd => Some("messageEnd"),
            _ => None,
        })
        .collect();
    assert_eq!(actions, vec!["move", "computerWin", "messageEnd"]);
}

#[test]
fn empty_and_terminal_positions_have_no_ai_move() {
    for words in [&[][..], &["사과"][..]] {
        let solver = get_wc_data(&rule(words, 0), 0).unwrap();
        let played = if words.is_empty() {
            vec![]
        } else {
            history(&["사과"])
        };
        for difficulty in 0..=2 {
            assert_eq!(
                choose_move(
                    &solver,
                    &played,
                    &AiOptions {
                        difficulty,
                        ..AiOptions::default()
                    }
                )
                .unwrap(),
                None
            );
        }
    }
}

#[test]
fn postprocessing_removes_once_or_recursively_and_additions_restore_words() {
    let mut settings = rule(&["사과", "과자", "자두"], 0);
    settings.content.postprocessing.manner.r#type = 1;
    assert_eq!(get_wc_data(&settings, 0).unwrap().word_map.get_size(), 2);
    settings.content.postprocessing.manner.r#type = 2;
    assert_eq!(get_wc_data(&settings, 0).unwrap().word_map.get_size(), 0);
    settings.content.postprocessing.added_words = "자두".into();
    assert_eq!(
        get_wc_data(&settings, 0).unwrap().word_map.get_all_words(),
        history(&["자두"])
    );
    settings.content.postprocessing.manner.r#type = 3;
    assert!(get_wc_data(&settings, 0).is_err());
}

#[test]
fn strategy_tree_rejects_route_and_expands_a_forced_win() {
    let solver = get_wc_data(&rule(&["사과", "과자", "자두"], 0), 0).unwrap();
    let tree = game_ai_rust::strategy::WcStrategyTree::new(&solver, 1, "사", None).unwrap();
    assert_eq!(tree.get_tree_data()[0].words, vec![history(&["사과"])]);
    let cycle = get_wc_data(&rule(&["가나", "나무바다", "다가"], 0), 0).unwrap();
    assert!(game_ai_rust::strategy::WcStrategyTree::new(&cycle, 0, "가", None).is_err());
}

#[test]
fn generic_strategy_tree_keeps_a_long_finite_path_complete() {
    use game_ai_rust::strategy::{StrategyRoot, StrategyTree};
    let tree = StrategyTree::new(
        StrategyRoot::Winning(0usize),
        |movement| Some(movement + 1),
        |movement| {
            if *movement < 100_001 {
                vec![*movement]
            } else {
                vec![]
            }
        },
    );
    assert_eq!(tree.winning_path.len(), 100_002);
    assert_eq!(tree.winning_path.last(), Some(&100_001));
}
