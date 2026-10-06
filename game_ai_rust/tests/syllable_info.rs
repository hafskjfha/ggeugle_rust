use game_ai_rust::*;

fn solver(words: &[&str], change_func_idx: usize) -> WordSolver {
    get_wc_data(&RuleForm::manual(words, change_func_idx), 0).unwrap()
}

#[test]
fn chain_classification_returns_a_winning_word_only_for_winning_starts() {
    let solver = solver(&["사과", "과자", "자두"], 0);
    for (syllable, kind, movement, words) in [
        ("사", NodeType::Win, Some(("사", "과")), vec!["사과"]),
        ("과", NodeType::Lose, None, vec![]),
        ("자", NodeType::Win, Some(("자", "두")), vec!["자두"]),
        ("두", NodeType::Lose, None, vec![]),
    ] {
        let info = solver.get_syllable_info(syllable, 0).unwrap();
        assert_eq!(info.node_type, kind, "syllable={syllable}");
        assert_eq!(
            info.winning_move,
            movement.map(|(head, tail)| (head.into(), tail.into())),
            "syllable={syllable}"
        );
        assert_eq!(info.winning_words, words, "syllable={syllable}");
    }
    assert!(!solver.graph_solver.graphs.union().has_node(0, "사"));
}

#[test]
fn odd_loop_words_skip_the_removed_pair_offset() {
    let solver = solver(&["가가", "가나가", "가다가"], 0);
    let info = solver.get_syllable_info("가", 0).unwrap();
    assert_eq!(info.node_type, NodeType::LoopWin);
    assert_eq!(info.winning_move, Some(("가".into(), "가".into())));
    assert_eq!(info.winning_words, ["가다가"]);
}

#[test]
fn a_route_classification_does_not_claim_a_winning_word() {
    let solver = solver(&["가나", "나다", "다가"], 0);
    let info = solver.get_syllable_info("가", 0).unwrap();
    assert_eq!(info.node_type, NodeType::Route);
    assert!(info.winning_move.is_none());
    assert!(info.winning_words.is_empty());
}

#[test]
fn a_missing_tail_uses_the_requested_initial_sound_rule() {
    let solver = solver(&["이름"], 1);
    let with_rule = solver.get_syllable_info("리", 1).unwrap();
    assert_eq!(with_rule.node_type, NodeType::Win);
    assert_eq!(with_rule.winning_move, Some(("이".into(), "름".into())));
    assert_eq!(with_rule.winning_words, ["이름"]);
    let without_rule = solver.get_syllable_info("리", 0).unwrap();
    assert_eq!(without_rule.node_type, NodeType::Lose);
    assert!(without_rule.winning_move.is_none());
    assert!(without_rule.winning_words.is_empty());
}

#[test]
fn a_missing_tail_finds_its_winning_head_among_losing_transformations() {
    let solver = solver(&["리이", "이나", "나이"], 1);
    let before = serde_json::to_string(&solver).unwrap();
    let info = solver.get_syllable_info("리", 1).unwrap();
    assert_eq!(info.node_type, NodeType::Win);
    assert_eq!(info.winning_move, Some(("리".into(), "이".into())));
    assert_eq!(info.winning_words, ["리이"]);
    assert!(
        search_is_win(
            &solver.graph_solver.graphs.union(),
            &info.winning_move.unwrap(),
            &PrecInfo::default(),
            None
        )
        .unwrap()
        .is_win
    );
    assert_eq!(serde_json::to_string(&solver).unwrap(), before);
}

#[test]
fn classification_queries_leave_the_source_solver_unchanged() {
    let solver = solver(&["이름", "가나", "나다", "다가"], 1);
    let before = serde_json::to_string(&solver).unwrap();
    solver.get_syllable_info("리", 1).unwrap();
    solver.get_syllable_info("가", 1).unwrap();
    assert_eq!(serde_json::to_string(&solver).unwrap(), before);
}

#[test]
fn classification_queries_validate_a_single_scalar_and_known_rule() {
    let solver = solver(&["사과"], 0);
    for syllable in ["", "사과", "사", "🍎🍐"] {
        assert!(matches!(
            solver.get_syllable_info(syllable, 0),
            Err(Error::InvalidInput(_))
        ));
    }
    for index in [11, usize::MAX] {
        assert!(matches!(
            solver.get_syllable_info("사", index),
            Err(Error::InvalidInput(_))
        ));
    }
    assert_eq!(
        solver.get_syllable_info("🍎", 0).unwrap().node_type,
        NodeType::Lose
    );
}
