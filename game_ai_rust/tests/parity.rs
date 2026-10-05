use game_ai_rust::*;
use serde_json::{Value, json};

fn golden() -> Value {
    serde_json::from_str(include_str!("fixtures/golden.json")).unwrap()
}
fn equivalent(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| equivalent(a, b)))
        }
        _ => actual == expected,
    }
}
fn matches(actual: Value, expected: &Value, case: &str, field: &str) {
    if !equivalent(&actual, expected) {
        let first = actual
            .as_array()
            .zip(expected.as_array())
            .and_then(|(a, b)| {
                a.iter()
                    .zip(b)
                    .position(|(left, right)| !equivalent(left, right))
                    .map(|i| json!({"index":i,"actual":a[i],"expected":b[i]}))
            });
        panic!(
            "{case}: {field} differs: {}",
            first.unwrap_or_else(|| json!({"actual":actual,"expected":expected}))
        );
    }
}

#[test]
fn graph_and_word_results_match_original_typescript_on_real_dictionary_samples() {
    for case in golden()["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let rule: RuleForm = serde_json::from_value(case["rule"].clone()).unwrap();
        let flow = case["flow"].as_u64().unwrap() as usize;
        let solver = get_wc_data(&rule, flow).unwrap();
        let graph = &solver.graph_solver;
        matches(
            json!(solver.word_map.get_all_words()),
            &case["words"],
            name,
            "words",
        );
        let types: Vec<_> = graph
            .type_map
            .iter()
            .map(|map| {
                let mut nodes: Vec<_> = map.iter().map(|(n, t)| (n.clone(), *t)).collect();
                nodes.sort_by(|a, b| a.0.cmp(&b.0));
                nodes
            })
            .collect();
        let depths: Vec<_> = graph
            .depth_map
            .iter()
            .map(|map| {
                let mut nodes: Vec<_> = map.iter().map(|(n, d)| (n.clone(), *d)).collect();
                nodes.sort_by(|a, b| a.0.cmp(&b.0));
                nodes
            })
            .collect();
        matches(json!(types), &case["types"], name, "types");
        matches(json!(depths), &case["depths"], name, "depths");
        for key in ["removed", "winlose", "route"] {
            let mut edges = graph.graphs.get_graph(key).edges(1);
            edges.sort();
            matches(json!(edges), &case["partitions"][key], name, key);
        }
        matches(
            json!(graph.get_word_type_num()),
            &case["wordTypes"],
            name,
            "wordTypes",
        );
        matches(
            json!([graph.get_route_nodes(0), graph.get_route_nodes(1)]),
            &case["routeNodes"],
            name,
            "routeNodes",
        );
        matches(
            json!([graph.get_max_route_info(0), graph.get_max_route_info(1)]),
            &case["maxRoute"],
            name,
            "maxRoute",
        );
        if let Some(queries) = case["wordCards"].as_array() {
            for query in queries {
                let node = query["name"].as_str().unwrap();
                let view = query["view"].as_u64().unwrap() as usize;
                let direction = query["direction"].as_u64().unwrap() as usize;
                matches(
                    json!(solver.get_words_cards_from_char(node, view, direction, None)),
                    &query["cards"],
                    name,
                    &format!("cards ({view},{node},{direction})"),
                );
            }
            matches(
                json!(solver.get_removed_words_file()),
                &case["removedWords"],
                name,
                "removedWords",
            );
            matches(
                json!([solver.get_win_words_file(0), solver.get_win_words_file(1)]),
                &case["winWords"],
                name,
                "winWords",
            );
            matches(
                json!([solver.get_bangdan_file(0), solver.get_bangdan_file(1)]),
                &case["bangdan"],
                name,
                "bangdan",
            );
            matches(
                json!([
                    solver.get_essential_win_words_file(0),
                    solver.get_essential_win_words_file(1)
                ]),
                &case["essentialWinWords"],
                name,
                "essentialWinWords",
            );
            matches(
                json!([
                    [
                        solver.get_route_words_file(0, 0),
                        solver.get_route_words_file(0, 1)
                    ],
                    [
                        solver.get_route_words_file(1, 0),
                        solver.get_route_words_file(1, 1)
                    ]
                ]),
                &case["routeWords"],
                name,
                "routeWords",
            );
            matches(
                json!([solver.get_scc_data(0, false), solver.get_scc_data(1, false)]),
                &case["sccData"],
                name,
                "sccData",
            );
        }
        if let Some(queries) = case["nextWords"].as_array() {
            for query in queries {
                let history: Vec<String> =
                    serde_json::from_value(query["history"].clone()).unwrap();
                matches(
                    json!(solver.get_next_words(&history).unwrap()),
                    &query["words"],
                    name,
                    "nextWords",
                );
                matches(
                    json!(is_game_end(&solver, &history, false).unwrap()),
                    &query["terminal"],
                    name,
                    "terminal",
                );
                matches(
                    json!(is_game_end(&solver, &history, true).unwrap()),
                    &query["stealableTerminal"],
                    name,
                    "stealableTerminal",
                );
            }
        }
        if let Some(searches) = case["search"].as_array() {
            for search in searches {
                let movement: SingleMove = serde_json::from_value(search["move"].clone()).unwrap();
                matches(
                    json!(
                        search_is_win(&graph.graphs.union(), &movement, &PrecInfo::default(), None)
                            .unwrap()
                            .is_win
                    ),
                    &search["isWin"],
                    name,
                    "search",
                );
                matches(
                    json!(
                        search_is_win(&graph.graphs.union(), &movement, &PrecInfo::default(), None)
                            .unwrap()
                            .optimal_path
                    ),
                    &search["optimalPath"],
                    name,
                    "optimalPath",
                );
            }
        }
    }
}

#[test]
fn hangul_transformations_match_original_typescript_for_all_sampled_syllable_components() {
    for change in golden()["changes"].as_array().unwrap() {
        let index = change["index"].as_u64().unwrap() as usize;
        let input = change["input"].as_str().unwrap();
        let rule = ChangeRule(index);
        matches(
            json!(rule.forward(input)),
            &change["forward"],
            input,
            &format!("change {index} forward"),
        );
        matches(
            json!(rule.backward(input)),
            &change["backward"],
            input,
            &format!("change {index} backward"),
        );
    }
}

#[test]
fn solver_json_round_trip_keeps_methods_and_word_history() {
    let solver = get_wc_data(&RuleForm::manual(&["가나", "가시나", "나니가"], 0), 0).unwrap();
    let restored: WordSolver =
        serde_json::from_str(&serde_json::to_string(&solver).unwrap()).unwrap();
    let history = vec!["가나".into(), "나니가".into()];
    assert_eq!(
        restored.get_next_words(&history).unwrap(),
        vec!["가시나".to_string()]
    );
    assert_eq!(
        restored
            .after_history(&history, 0)
            .unwrap()
            .graphs
            .union()
            .get_edge_num("가", "나"),
        1
    );
}
