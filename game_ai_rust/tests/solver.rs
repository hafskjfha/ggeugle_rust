use game_ai_rust::{
    NodeType,
    edge_map::EdgeMap,
    graph::BipartiteDiGraph,
    pairs::PairManager,
    solver::{
        DistributionCalculation, DistributionDisplay, DistributionKey, DistributionSort,
        GraphSolver, get_comparison_data,
    },
};

fn graph(words: &[(&str, &str, usize)]) -> BipartiteDiGraph {
    let mut graph = BipartiteDiGraph::new();
    for &(head, tail, count) in words {
        graph.set_edge(1, head, tail, count);
    }
    for node in graph.nodes(0) {
        if graph.has_node(1, &node) {
            graph.set_edge(0, &node, &node, 1);
        }
    }
    graph
}

#[test]
fn both_flows_classify_chain_and_count_word_depths() {
    for flow in [0, 1] {
        let solver = GraphSolver::new(
            graph(&[("사", "과", 1), ("과", "자", 1), ("자", "두", 1)]),
            flow,
        );
        assert_eq!(solver.get_node_type("사", 1, None), NodeType::Win);
        assert_eq!(solver.get_node_type("과", 0, None), NodeType::Lose);
        assert_eq!(solver.get_node_type("자", 0, None), NodeType::Win);
        assert_eq!(solver.depth_map[1]["사"], 3);
        assert_eq!(solver.depth_map[0]["과"], 2);
        assert_eq!(
            solver.get_winning_optimal_move(1, "사", None),
            Some(("사".into(), "과".into()))
        );
        assert_eq!(
            solver.get_losing_optimal_move(0, "과", None),
            Some(("과".into(), "자".into()))
        );
    }
}

#[test]
fn parallel_forced_loops_keep_parity_and_word_index_offsets() {
    for count in [2, 3, 4, 5] {
        let solver = GraphSolver::new(graph(&[("가", "가", count)]), 0);
        let removed = count - count % 2;
        assert_eq!(
            solver
                .graphs
                .get_edge_idx_range("가", "가", Some("removed")),
            (0, removed)
        );
        assert_eq!(
            solver
                .graphs
                .get_edge_idx_range("가", "가", Some("winlose")),
            (removed, count)
        );
        assert_eq!(
            solver.get_node_type("가", 1, None),
            if count % 2 == 0 {
                NodeType::Lose
            } else {
                NodeType::LoopWin
            }
        );
        assert_eq!(
            solver.pair_manager.get_pair_idx("가", "가", 0),
            Some(("가".into(), "가".into(), removed / 2))
        );
    }
}

#[test]
fn reciprocal_words_become_symmetric_removed_pairs() {
    let solver = GraphSolver::new(graph(&[("가", "나", 2), ("나", "가", 1)]), 0);
    assert_eq!(
        solver.graphs.get_graph("removed").get_edge_num("가", "나"),
        1
    );
    assert_eq!(
        solver.graphs.get_graph("removed").get_edge_num("나", "가"),
        1
    );
    assert_eq!(
        solver.pair_manager.get_pair_idx("가", "나", 0),
        Some(("나".into(), "가".into(), 0))
    );
    assert_eq!(
        solver.pair_manager.get_pair_idx("나", "가", 0),
        Some(("가".into(), "나".into(), 0))
    );
}

#[test]
fn pair_manager_combines_even_loop_and_cycle_offsets() {
    let manager = PairManager::from_data(
        &[("가".into(), "나".into(), 4)],
        &[
            (("가".into(), "나".into()), ("다".into(), "라".into()), 2),
            (("가".into(), "나".into()), ("마".into(), "바".into()), 1),
        ],
    );
    assert_eq!(manager.get_removed_word_num("가", "나"), 7);
    assert_eq!(
        manager.get_pair_idx("가", "나", 3),
        Some(("가".into(), "나".into(), 1))
    );
    assert_eq!(
        manager.get_pair_idx("가", "나", 5),
        Some(("다".into(), "라".into(), 1))
    );
    assert_eq!(
        manager.get_pair_idx("가", "나", 6),
        Some(("마".into(), "바".into(), 0))
    );
    assert_eq!(manager.get_pair_idx("가", "나", 7), None);
}

#[test]
fn winning_optimal_ties_choose_last_pair_after_lifo_pruning() {
    let solver = GraphSolver::new(graph(&[("가", "나", 1), ("가", "다", 1)]), 0);
    // The original engine's LIFO transfers put 다 before 나 in winlose.
    assert_eq!(
        solver.graphs.get_graph("winlose").successors(1, "가"),
        ["다", "나"]
    );
    assert_eq!(
        solver.get_winning_optimal_move(1, "가", None),
        Some(("가".into(), "나".into()))
    );
}

#[test]
fn route_scc_and_distribution_queries_keep_analysis_semantics() {
    let solver = GraphSolver::new(
        graph(&[("가", "나", 1), ("나", "다", 1), ("다", "가", 1)]),
        0,
    );
    assert_eq!(solver.get_node_type("가", 0, None), NodeType::Route);
    assert_eq!(solver.get_move_type("가", "나", 0).connected, Some(true));
    assert_eq!(solver.get_word_type_num().type_num, [0, 3, 0, 0, 0, 0]);
    assert_eq!(
        solver.get_route_nodes(0),
        [["가", "나", "다"].map(str::to_owned).to_vec(), vec![]]
    );
    assert!(solver.get_scc_data(0, false).is_empty());
    let info = solver.get_max_route_info(0);
    assert_eq!(info.char_num, 3);
    assert_eq!(info.move_num, 3);
    assert_eq!(info.average_num, 1.0);
    assert_eq!(
        solver.get_distribution_map(NodeType::Route, 0, 0)["가"],
        [0, 1, 0, 0, 0, 0]
    );
}

#[test]
fn empty_graph_and_missing_nodes_return_safe_analysis() {
    let solver = GraphSolver::new(BipartiteDiGraph::new(), 0);
    assert_eq!(solver.get_node_type("없", 0, None), NodeType::Lose);
    assert_eq!(solver.get_winning_optimal_move(0, "없", None), None);
    assert_eq!(solver.get_losing_optimal_move(0, "없", None), None);
    assert_eq!(solver.get_max_route_info(0).average_num, 0.0);
    assert_eq!(solver.get_word_type_num().type_num, [0; 6]);
}

#[test]
fn route_move_class_keeps_connectivity_group_indices() {
    let solver = GraphSolver::new(
        graph(&[("가", "나", 1), ("나", "다", 1), ("다", "가", 1)]),
        0,
    );
    let moves = EdgeMap::from_array(vec![("가".into(), "나".into(), vec![0])]);
    let classes = solver.classify_moves(&moves);
    assert_eq!(classes[1][&0].get("가", "나").unwrap().word_idx, vec![0]);
    assert!(classes[1].get(&1).is_none());
}

#[test]
fn scc_bridges_and_directional_distributions_match_source_engine() {
    let solver = GraphSolver::new(
        graph(&[
            ("가", "나", 1),
            ("나", "다", 1),
            ("다", "가", 1),
            ("라", "마", 1),
            ("마", "바", 1),
            ("바", "라", 1),
            ("다", "라", 1),
        ]),
        0,
    );
    for pos in [0, 1] {
        let scc = solver.get_scc_data(pos, false);
        assert_eq!(scc.len(), 2);
        assert_eq!(scc[0].nodes, ["다", "가", "나"]);
        assert_eq!(scc[1].nodes, ["라", "바", "마"]);
        assert_eq!(scc[0].succ[0].by, vec![("다".into(), "라".into(), (0, 1))]);
        assert!(scc[1].succ.is_empty());
        assert_eq!(solver.get_scc_data(pos, true)[0].nodes, scc[1].nodes);
    }
    assert_eq!(solver.get_move_type("다", "라", 0).connected, Some(false));
    let distribution = solver.get_distribution_with_calc(
        NodeType::Route,
        0,
        [DistributionKey::Total, DistributionKey::Total],
        false,
        DistributionCalculation::Difference,
    );
    assert_eq!(distribution.first().unwrap().char, "다");
    assert_eq!(distribution.first().unwrap().num, [1.0, 2.0, -1.0]);
    assert_eq!(distribution.last().unwrap().char, "라");
    assert_eq!(distribution.last().unwrap().num, [2.0, 1.0, 1.0]);
    let fractions = solver.get_distribution(
        NodeType::Route,
        0,
        0,
        DistributionSort {
            key: DistributionKey::Total,
            desc: false,
        },
        DistributionDisplay::Fraction,
    );
    assert!(fractions.iter().all(|row| row.num[1] == 1.0));
}

#[test]
fn comparing_a_changed_graph_reports_before_and_after_types() {
    let before = GraphSolver::new(
        graph(&[("가", "나", 1), ("나", "다", 1), ("다", "가", 1)]),
        0,
    );
    let after = GraphSolver::new(graph(&[("가", "나", 1), ("나", "다", 1)]), 0);
    let mapping = before.get_comparison_map(&after, game_ai_rust::rules::ChangeRule(0));
    assert_eq!(mapping[0]["다"], (NodeType::Route, NodeType::Lose));
    assert_eq!(mapping[0]["나"], (NodeType::Route, NodeType::Win));
    let grouped = get_comparison_data(&mapping);
    assert_eq!(grouped[0].len(), 12);
    assert!(
        grouped[0]
            .iter()
            .any(|(from, to, nodes)| *from == NodeType::Route
                && *to == NodeType::Lose
                && nodes.contains(&"다".into()))
    );
}
