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
    let previous = finish_root_search(&plan, &branches, 0.0).unwrap();
    let current = search_syllable(&solver, "가", 0, &prec, None).unwrap();
    assert!(current.is_win, "the five-word forced cycle has odd length");
    assert_eq!(current.is_win, !previous.is_win);
    assert_eq!(current.optimal_path, previous.optimal_path[1..]);
    assert_eq!(current.visited, previous.visited - 1);
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
}
