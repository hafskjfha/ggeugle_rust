use game_ai_rust::*;
use std::time::Duration;

fn solver(words: &[&str], change_func_idx: usize) -> GraphSolver {
    get_wc_data(&RuleForm::manual(words, change_func_idx), 0)
        .unwrap()
        .graph_solver
}

#[test]
fn a_head_absent_from_the_tail_nodes_is_still_a_winning_start() {
    let solver = solver(&["사과"], 0);
    assert!(!solver.graphs.union().has_node(0, "사"));
    let prec = PrecInfo::default();
    let plan = prepare_syllable_search(&solver, "사", 0, &prec, None).unwrap();
    assert_eq!(plan.movement, ("__none".into(), "사".into()));
    let previous = plan.result.unwrap();
    assert!(!previous.is_win);
    assert_eq!(previous.optimal_path, [("__none".into(), "사".into())]);
    assert_eq!(previous.visited, 1);

    let current = search_syllable(&solver, "사", 0, &prec, None).unwrap();
    assert!(current.is_win);
    assert!(current.optimal_path.is_empty());
    assert_eq!(current.visited, 0);
    assert!(current.duration.is_finite() && current.duration >= 0.0);
}

#[test]
fn chain_queries_report_the_current_players_outcome() {
    let solver = solver(&["사과", "과자", "자두"], 0);
    for (syllable, is_win) in [("사", true), ("과", false), ("자", true), ("두", false)] {
        let result = search_syllable(&solver, syllable, 0, &PrecInfo::default(), None).unwrap();
        assert_eq!(result.is_win, is_win, "syllable={syllable}");
        assert!(result.optimal_path.is_empty());
        assert_eq!(result.visited, 0);
    }
}

#[test]
fn a_missing_syllable_with_no_matching_head_is_a_loss() {
    let solver = solver(&["사과"], 0);
    let plan = prepare_syllable_search(&solver, "없", 0, &PrecInfo::default(), None).unwrap();
    assert!(plan.result.unwrap().is_win);
    let result = search_syllable(&solver, "없", 0, &PrecInfo::default(), None).unwrap();
    assert!(!result.is_win);
    assert!(result.optimal_path.is_empty());
    assert_eq!(result.visited, 0);
}

#[test]
fn an_odd_loop_is_a_win_without_consuming_a_dictionary_word() {
    let solver = solver(&["가가"], 0);
    assert_eq!(solver.get_node_type("가", 0, None), NodeType::LoopWin);
    let result = search_syllable(&solver, "가", 0, &PrecInfo::default(), None).unwrap();
    assert!(result.is_win);
    assert!(result.optimal_path.is_empty());
    assert_eq!(result.visited, 0);
}

#[test]
fn a_route_search_matches_the_legal_first_moves_and_parallel_plan() {
    let solver = solver(&["가나", "나다", "다가", "가라", "라나"], 0);
    assert_eq!(solver.get_node_type("가", 0, None), NodeType::Route);
    let prec = PrecInfo::default();
    let plan = prepare_syllable_search(&solver, "가", 0, &prec, None).unwrap();
    assert!(
        plan.result.is_none(),
        "must exercise actual search branches"
    );
    let mut moves = plan.moves.clone();
    moves.sort();
    assert_eq!(
        moves,
        [("가".into(), "나".into()), ("가".into(), "라".into())]
    );

    let branches: Vec<_> = plan
        .moves
        .iter()
        .map(|movement| {
            Some(search_root_branch(&plan.graph, movement, &prec, None, |_| {}).unwrap())
        })
        .collect();
    let merged = finish_syllable_search(&plan, &branches, 0.0).unwrap();
    let current = search_syllable(&solver, "가", 0, &prec, None).unwrap();
    assert!(current.is_win, "the five-word forced cycle has odd length");
    assert_eq!(current.is_win, merged.result.is_win);
    assert_eq!(current.optimal_path, merged.result.optimal_path);
    assert_eq!(current.visited, merged.result.visited);
    assert_eq!(merged.winning_move.as_ref(), current.optimal_path.first());
    assert!(moves.contains(current.optimal_path.first().unwrap()));
    assert!(
        current
            .optimal_path
            .iter()
            .all(|movement| movement.0 != "__none")
    );

    let graph = solver.graphs.union();
    let any_winning_first_move = moves
        .iter()
        .any(|movement| search_is_win(&graph, movement, &prec, None).unwrap().is_win);
    assert_eq!(current.is_win, any_winning_first_move);
}

#[test]
fn an_absent_tail_uses_the_requested_initial_sound_rule() {
    let solver = solver(&["이름"], 1);
    assert!(!solver.graphs.union().has_node(0, "리"));
    let prec = PrecInfo::default();
    assert!(
        search_syllable(&solver, "리", 1, &prec, None)
            .unwrap()
            .is_win
    );
    assert!(
        !search_syllable(&solver, "리", 0, &prec, None)
            .unwrap()
            .is_win
    );
}

#[test]
fn an_existing_tail_retains_its_transformation_edges() {
    let solver = solver(&["이름", "나리"], 1);
    let graph = solver.graphs.union();
    assert!(graph.has_node(0, "리"));
    assert!(graph.has_edge(0, "리", "이"));
    assert!(
        search_syllable(&solver, "리", 0, &PrecInfo::default(), None)
            .unwrap()
            .is_win
    );
}

#[test]
fn preparing_and_searching_leave_the_source_solver_unchanged() {
    let solver = solver(&["이름", "가나", "나다", "다가"], 1);
    let before = serde_json::to_string(&solver).unwrap();
    let prec = PrecInfo::default();
    prepare_syllable_search(&solver, "리", 1, &prec, None).unwrap();
    search_syllable(&solver, "가", 1, &prec, None).unwrap();
    search_syllable_with_witness(&solver, "가", 1, &prec, None).unwrap();
    assert_eq!(serde_json::to_string(&solver).unwrap(), before);
}

#[test]
fn queries_require_one_unicode_scalar_and_a_known_change_rule() {
    let solver = solver(&["사과"], 0);
    let prec = PrecInfo::default();
    for syllable in ["", "사과", "사", "🍎🍐"] {
        assert!(matches!(
            prepare_syllable_search(&solver, syllable, 0, &prec, None),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            search_syllable(&solver, syllable, 0, &prec, None),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            search_syllable_with_witness(&solver, syllable, 0, &prec, None),
            Err(Error::InvalidInput(_))
        ));
    }
    for index in [11, usize::MAX] {
        assert!(matches!(
            prepare_syllable_search(&solver, "사", index, &prec, None),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            search_syllable(&solver, "사", index, &prec, None),
            Err(Error::InvalidInput(_))
        ));
        assert!(matches!(
            search_syllable_with_witness(&solver, "사", index, &prec, None),
            Err(Error::InvalidInput(_))
        ));
    }
    // One scalar is the public contract, including supplementary-plane input.
    assert!(
        !search_syllable(&solver, "🍎", 0, &prec, None)
            .unwrap()
            .is_win
    );
    search_syllable(&solver, "사", 10, &prec, None).unwrap();
}

#[test]
fn both_syllable_search_apis_respect_an_immediate_deadline() {
    let solver = solver(&["사과"], 0);
    let prec = PrecInfo::default();
    assert!(matches!(
        prepare_syllable_search(&solver, "사", 0, &prec, Some(Duration::ZERO)),
        Err(Error::Timeout)
    ));
    assert!(matches!(
        search_syllable(&solver, "사", 0, &prec, Some(Duration::ZERO)),
        Err(Error::Timeout)
    ));
    assert!(matches!(
        search_syllable_with_witness(&solver, "사", 0, &prec, Some(Duration::ZERO)),
        Err(Error::Timeout)
    ));
}

#[test]
fn a_winning_identity_query_starts_its_path_with_a_winning_move() {
    let solver = solver(&["가가", "가나", "나나", "나다", "다가"], 0);
    let prec = PrecInfo::default();
    let result = search_syllable(&solver, "나", 0, &prec, None).unwrap();
    assert!(result.is_win);
    let first = result.optimal_path.first().unwrap();
    assert!(
        search_is_win(&solver.graphs.union(), first, &prec, None)
            .unwrap()
            .is_win
    );
    assert_eq!(first, &("나".into(), "다".into()));
}

#[test]
fn a_winning_standard_query_starts_its_path_with_a_winning_move() {
    let solver = solver(&["라니", "라가니", "나라", "니나"], 1);
    let prec = PrecInfo::default();
    let result = search_syllable(&solver, "라", 1, &prec, None).unwrap();
    assert!(result.is_win);
    let first = result.optimal_path.first().unwrap();
    assert!(
        search_is_win(&solver.graphs.union(), first, &prec, None)
            .unwrap()
            .is_win
    );
    assert_eq!(first, &("나".into(), "라".into()));
}

#[test]
fn a_cached_loss_requiring_more_reduction_rounds_needs_no_search() {
    let solver = solver(
        &[
            "다가", "핍니", "이핍", "라다", "니라", "결이", "가결", "가다",
        ],
        1,
    );
    assert_eq!(solver.get_node_type("가", 0, None), NodeType::Lose);
    let prec = PrecInfo::default();
    let plan = prepare_syllable_search(&solver, "가", 1, &prec, None).unwrap();
    assert!(
        plan.result.is_some(),
        "the existing loss classification is sufficient"
    );
    assert!(plan.moves.is_empty());
    let result = search_syllable(&solver, "가", 1, &prec, None).unwrap();
    assert!(!result.is_win);
    assert!(result.optimal_path.is_empty());
    assert_eq!(result.visited, 0);
}

#[test]
fn serial_and_merged_witnesses_select_the_first_proven_winning_branch() {
    for (words, rule, syllable, expected) in [
        (
            vec!["가가", "가나", "나나", "나다", "다가"],
            0,
            "나",
            ("나", "다"),
        ),
        (
            vec!["라니", "라가니", "나라", "니나"],
            1,
            "라",
            ("나", "라"),
        ),
    ] {
        let solver = solver(&words, rule);
        let prec = PrecInfo::default();
        let plan = prepare_syllable_search(&solver, syllable, rule, &prec, None).unwrap();
        let branches: Vec<_> = plan
            .moves
            .iter()
            .map(|movement| {
                Some(search_root_branch(&plan.graph, movement, &prec, None, |_| {}).unwrap())
            })
            .collect();
        assert!(!branches[0].as_ref().unwrap().result.is_win);
        assert!(branches[1].as_ref().unwrap().result.is_win);
        assert_eq!(
            branches[0].as_ref().unwrap().outcome,
            branches[1].as_ref().unwrap().outcome,
            "legacy backtracking markers cannot identify the winning branch"
        );
        let merged = finish_syllable_search(&plan, &branches, 0.0).unwrap();
        let serial = search_syllable_with_witness(&solver, syllable, rule, &prec, None).unwrap();
        assert!(serial.result.is_win);
        assert_eq!(
            serial.winning_move,
            Some((expected.0.into(), expected.1.into()))
        );
        assert_eq!(serial.winning_move, merged.winning_move);
        assert_eq!(serial.result.optimal_path, merged.result.optimal_path);
        assert_eq!(serial.result.visited, merged.result.visited);
        assert_eq!(
            serial.result.optimal_path.first(),
            serial.winning_move.as_ref()
        );
        assert!(
            search_is_win(
                &solver.graphs.union(),
                serial.winning_move.as_ref().unwrap(),
                &prec,
                None
            )
            .unwrap()
            .is_win
        );
    }
}

#[test]
fn a_losing_route_query_has_no_winning_witness() {
    let solver = solver(&["가나", "나다", "다라", "라가"], 0);
    assert_eq!(solver.get_node_type("가", 0, None), NodeType::Route);
    let result =
        search_syllable_with_witness(&solver, "가", 0, &PrecInfo::default(), None).unwrap();
    assert!(!result.result.is_win);
    assert!(result.winning_move.is_none());
    assert!(result.result.visited > 0);
}

#[test]
fn witness_results_keep_search_fields_at_the_top_level() {
    let solver = solver(&["가나", "나다", "다라", "라가"], 0);
    let result =
        search_syllable_with_witness(&solver, "가", 0, &PrecInfo::default(), None).unwrap();
    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["isWin"], false);
    assert!(json["optimalPath"].is_array());
    assert!(json["visited"].is_number());
    assert!(json["winningMove"].is_null());
    assert!(json.get("result").is_none());
}

#[test]
fn finishing_a_syllable_search_requires_the_completed_ordered_prefix() {
    let solver = solver(&["가가", "가나", "나나", "나다", "다가"], 0);
    let prec = PrecInfo::default();
    let plan = prepare_syllable_search(&solver, "나", 0, &prec, None).unwrap();
    let winning = search_root_branch(&plan.graph, &plan.moves[1], &prec, None, |_| {}).unwrap();
    assert!(matches!(
        finish_syllable_search(&plan, &[None, Some(winning)], 0.0),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        finish_syllable_search(&plan, &[], f64::NAN),
        Err(Error::InvalidInput(_))
    ));
}
