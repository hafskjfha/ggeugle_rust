use game_ai_rust::*;

fn solver(words: &[&str]) -> WordSolver {
    get_wc_data(&RuleForm::manual(words, 0), 0).unwrap()
}

fn history(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn history_snapshot_queries_the_position_after_the_played_word() {
    let source = solver(&["가나", "나가"]);
    assert_eq!(
        source.get_syllable_info("나", 0).unwrap().node_type,
        NodeType::Lose
    );

    for flow in [0, 1] {
        let snapshot = source.with_history(&history(&["가나"]), flow).unwrap();
        let info = snapshot.get_syllable_info("나", 0).unwrap();
        assert_eq!(info.node_type, NodeType::Win);
        assert_eq!(info.winning_move, Some(("나".into(), "가".into())));
        assert_eq!(info.winning_words, ["나가"]);
        assert_eq!(snapshot.word_map.get_all_words(), ["나가"]);
        assert_eq!(
            snapshot
                .graph_solver
                .graphs
                .union()
                .get_edge_num("가", "나"),
            0
        );
        assert_eq!(
            snapshot
                .graph_solver
                .graphs
                .union()
                .get_edge_num("나", "가"),
            1
        );
        assert_eq!(snapshot.flow, flow);
        assert_eq!(snapshot.graph_solver.flow, flow);
        assert_eq!(snapshot.head_idx, 0);
        assert_eq!(snapshot.tail_idx, -1);
    }
}

#[test]
fn history_snapshot_removes_one_word_and_preserves_order_for_matching_pairs() {
    let source = solver(&["가나", "가시나", "가이아나", "나가"]);
    let snapshot = source.with_history(&history(&["가시나"]), 0).unwrap();
    assert_eq!(
        snapshot.word_map.get_all_words(),
        ["가나", "가이아나", "나가"]
    );
    assert_eq!(snapshot.word_map.get_word("가", "나", 0), Some("가나"));
    assert_eq!(snapshot.word_map.get_word("가", "나", 1), Some("가이아나"));
    assert_eq!(
        snapshot
            .graph_solver
            .graphs
            .union()
            .get_edge_num("가", "나"),
        2
    );

    let only_first_used = solver(&["가나", "가시나", "나가"])
        .with_history(&history(&["가나"]), 0)
        .unwrap();
    assert_eq!(only_first_used.word_map.get_all_words(), ["가시나", "나가"]);
    assert_eq!(
        only_first_used
            .graph_solver
            .graphs
            .union()
            .get_edge_num("가", "나"),
        1
    );
}

#[test]
fn winning_word_offsets_refer_to_remaining_words_in_the_snapshot() {
    // One 가 -> 나 edge is paired with 나 -> 가; the winning edge is the
    // next word in the remaining pair, after removing the used middle word.
    let source = solver(&["가나", "가시나", "가이아나", "나가"]);
    let snapshot = source.with_history(&history(&["가시나"]), 0).unwrap();
    let info = snapshot.get_syllable_info("가", 0).unwrap();
    assert_eq!(info.node_type, NodeType::Win);
    assert_eq!(info.winning_move, Some(("가".into(), "나".into())));
    assert_eq!(info.winning_words, ["가이아나"]);
    assert!(!info.winning_words.iter().any(|word| word == "가시나"));
}

#[test]
fn repeated_history_words_are_consumed_once_for_a_stolen_opening() {
    let source = solver(&["가나", "가시나", "나가"]);
    let snapshot = source.with_history(&history(&["가나", "가나"]), 0).unwrap();
    assert_eq!(snapshot.word_map.get_all_words(), ["가시나", "나가"]);
    assert_eq!(
        snapshot
            .graph_solver
            .graphs
            .union()
            .get_edge_num("가", "나"),
        1
    );
}

#[test]
fn history_snapshots_validate_flow_and_unknown_words() {
    let source = solver(&["가나", "나가"]);
    for flow in [2, usize::MAX] {
        assert!(matches!(
            source.with_history(&[], flow),
            Err(Error::InvalidInput(_))
        ));
    }
    for used in [history(&["가다"]), history(&["가나", "가다", "가다"])] {
        assert!(matches!(
            source.with_history(&used, 0),
            Err(Error::InvalidInput(_))
        ));
    }
}

#[test]
fn history_snapshots_leave_the_original_solver_unchanged() {
    let source = solver(&["가나", "가시나", "나가"]);
    let before = serde_json::to_value(&source).unwrap();
    source.with_history(&history(&["가나"]), 1).unwrap();
    source.with_history(&[], 0).unwrap();
    assert_eq!(serde_json::to_value(&source).unwrap(), before);
}

#[test]
fn serialized_history_snapshots_keep_query_metadata_and_word_counts() {
    let snapshot = solver(&["가나", "가시나", "가이아나", "나가"])
        .with_history(&history(&["가시나"]), 0)
        .unwrap();
    let json = serde_json::to_value(&snapshot).unwrap();
    let restored: WordSolver = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), json);
    assert_eq!(
        restored.get_syllable_info("가", 0).unwrap().winning_words,
        ["가이아나"]
    );
    assert_eq!(
        restored.word_map.get_all_words(),
        ["가나", "가이아나", "나가"]
    );
    assert_eq!(
        restored
            .graph_solver
            .graphs
            .union()
            .get_edge_num("가", "나"),
        2
    );
    assert_eq!(
        restored
            .graph_solver
            .graphs
            .union()
            .get_edge_num("나", "가"),
        1
    );
}
