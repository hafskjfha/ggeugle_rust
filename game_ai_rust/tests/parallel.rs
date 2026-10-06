use game_ai_rust::*;
use std::time::Duration;

fn merge_every_branch(plan: &RootSearchPlan, prec: &PrecInfo) -> SearchResult {
    let branches: Vec<_> = plan
        .moves
        .iter()
        .map(|movement| {
            Some(search_root_branch(&plan.graph, movement, prec, None, |_| {}).unwrap())
        })
        .collect();
    finish_root_search(plan, &branches, 12.5).unwrap()
}

#[test]
fn preparation_consumes_the_incoming_word_once_without_mutating_the_source() {
    let solver = get_wc_data(&RuleForm::manual(&["가가", "가오가"], 0), 0).unwrap();
    let graph = solver.graph_solver.graphs.union();
    let before = serde_json::to_string(&graph).unwrap();
    let movement = ("가".into(), "가".into());
    let plan = prepare_root_search(&graph, &movement, &PrecInfo::default(), None).unwrap();
    let result = plan
        .result
        .as_ref()
        .expect("one remaining loop is determined");
    assert!(!result.is_win);
    assert_eq!(result.optimal_path, vec![movement]);
    assert_eq!(result.visited, 1);
    assert_eq!(serde_json::to_string(&graph).unwrap(), before);
}

#[test]
fn planning_respects_an_immediate_deadline() {
    let graph = BipartiteDiGraph::new();
    let result = prepare_root_search(
        &graph,
        &("__none".into(), "가".into()),
        &PrecInfo::default(),
        Some(Duration::ZERO),
    );
    assert!(matches!(result, Err(Error::Timeout)));
}

#[test]
fn merging_preserves_sequential_outcome_path_and_visits_for_small_multigraphs() {
    let mut split_count = 0;
    let mut recursive_markers = 0;
    // Exhaustive three-node graphs exercise cycles, pruning, and branch ties.
    for mask in 0..512usize {
        let mut graph = BipartiteDiGraph::new();
        for node in ["가", "나", "다"] {
            graph.set_edge(0, node, node, 1);
        }
        for (index, (head, tail)) in ["가", "나", "다"]
            .into_iter()
            .flat_map(|head| ["가", "나", "다"].into_iter().map(move |tail| (head, tail)))
            .enumerate()
        {
            if mask & (1 << index) != 0 {
                graph.set_edge(1, head, tail, 1 + usize::from(mask % 7 == 0));
            }
        }
        for (head, tail, _) in graph.edges(1) {
            let movement = (head, tail);
            let prec = PrecInfo::default();
            let sequential = search_is_win(&graph, &movement, &prec, None).unwrap();
            let plan = prepare_root_search(&graph, &movement, &prec, None).unwrap();
            let mut branches = Vec::new();
            for reply in &plan.moves {
                let branch = search_root_branch(&plan.graph, reply, &prec, None, |_| {}).unwrap();
                if (branch.outcome == NodeType::Win) == branch.result.is_win {
                    recursive_markers += 1;
                }
                branches.push(Some(branch));
            }
            split_count += usize::from(!plan.moves.is_empty());
            let merged = finish_root_search(&plan, &branches, 12.5).unwrap();
            assert_eq!(
                merged.is_win, sequential.is_win,
                "mask={mask}, move={movement:?}"
            );
            assert_eq!(
                merged.optimal_path, sequential.optimal_path,
                "mask={mask}, move={movement:?}"
            );
            assert_eq!(
                merged.visited, sequential.visited,
                "mask={mask}, move={movement:?}"
            );
            assert_eq!(merged.duration, 12.5);
        }
    }
    assert!(split_count > 0, "must exercise real root splitting");
    assert!(
        recursive_markers > 0,
        "must exercise nonterminal backtracking markers"
    );
}

#[test]
fn merging_matches_original_typescript_golden_paths() {
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/golden.json")).unwrap();
    for case in golden["cases"].as_array().unwrap() {
        let Some(queries) = case["search"].as_array() else {
            continue;
        };
        let rule: RuleForm = serde_json::from_value(case["rule"].clone()).unwrap();
        let solver = get_wc_data(&rule, case["flow"].as_u64().unwrap() as usize).unwrap();
        for query in queries {
            let movement = serde_json::from_value(query["move"].clone()).unwrap();
            let prec = PrecInfo::default();
            let plan =
                prepare_root_search(&solver.graph_solver.graphs.union(), &movement, &prec, None)
                    .unwrap();
            let result = merge_every_branch(&plan, &prec);
            assert_eq!(serde_json::json!(result.is_win), query["isWin"]);
            assert_eq!(serde_json::json!(result.optimal_path), query["optimalPath"]);
        }
    }
}

#[test]
fn missing_ordered_prefix_is_rejected_but_work_after_a_proven_win_is_unnecessary() {
    let mut graph = BipartiteDiGraph::new();
    for node in ["가", "나", "다"] {
        graph.set_edge(0, node, node, 1);
    }
    let plan = RootSearchPlan {
        graph,
        movement: ("__none".into(), "가".into()),
        moves: vec![("가".into(), "나".into()), ("가".into(), "다".into())],
        result: None,
    };
    let branch = RootBranchResult {
        result: SearchResult {
            is_win: true,
            duration: 0.0,
            optimal_path: vec![plan.moves[0].clone()],
            visited: 1,
        },
        outcome: NodeType::Lose,
    };
    assert!(finish_root_search(&plan, &[None, Some(branch.clone())], 0.0).is_err());
    let result = finish_root_search(&plan, &[Some(branch), None], 0.0).unwrap();
    assert!(!result.is_win);
    assert_eq!(result.visited, 2);
    assert!(finish_root_search(&plan, &[], 0.0).is_err());
    assert!(finish_root_search(&plan, &[None, None], f64::NAN).is_err());
}
