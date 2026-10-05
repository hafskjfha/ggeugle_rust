use game_ai_rust::edge_map::EdgeMap;
use game_ai_rust::graph::BipartiteDiGraph;
use game_ai_rust::partitions::GraphPartitions;
use game_ai_rust::rules::ChangeRule;
use indexmap::IndexSet;

fn pair(a: &str, b: &str) -> (String, String) {
    (a.into(), b.into())
}

#[test]
fn edge_map_preserves_order_and_removes_empty_sources() {
    let mut map = EdgeMap::new();
    map.set("b", "c", 2);
    map.set("a", "d", 3);
    map.set("b", "e", 4);
    map.set("b", "c", 5);
    assert_eq!(
        map.to_array(),
        vec![
            ("b".into(), "c".into(), 5),
            ("b".into(), "e".into(), 4),
            ("a".into(), "d".into(), 3)
        ]
    );
    map.remove("b", Some("c"));
    map.remove("b", Some("e"));
    assert_eq!(map.to_array(), vec![("a".into(), "d".into(), 3)]);
    assert_eq!(*map.get_or_insert("x", "y", 7), 7);
    assert_eq!(map.get("x", "y"), Some(&7));
}

#[test]
fn graph_mutation_keeps_reverse_edges_and_distinct_degrees() {
    let mut graph = BipartiteDiGraph::new();
    graph.set_edge(1, "a", "b", 3);
    graph.set_edge(1, "a", "c", 1);
    graph.set_edge(0, "b", "a", 1);
    assert_eq!(graph.out_degree(1, "a"), 2);
    assert_eq!(graph.in_degree(0, "b"), 1);
    assert_eq!(graph.get_all_move_num(), 4);
    graph.decrease_edge("a", "b", 2).unwrap();
    assert_eq!(graph.get_edge_num("a", "b"), 1);
    assert!(graph.decrease_edge("a", "b", 2).is_err());
    assert_eq!(graph.get_edge_num("a", "b"), 1);
    graph.decrease_edge("a", "b", 1).unwrap();
    assert_eq!(graph.predecessors(0, "b"), Vec::<String>::new());
    assert!(graph.has_node(0, "b"));
    assert_eq!(
        graph.remove_node(0, "b"),
        [vec![("b".into(), "a".into(), 1)], vec![]]
    );
    assert_eq!(graph.predecessors(1, "a"), Vec::<String>::new());
    assert_eq!(graph.remove_node(0, "missing"), [vec![], vec![]]);
}

fn chain_graph() -> BipartiteDiGraph {
    let mut words = EdgeMap::new();
    words.set("a", "b", vec!["ab".into(), "abb".into()]);
    words.set("b", "c", vec!["bc".into()]);
    words.set("c", "b", vec!["cb".into()]);
    words.set("z", "z", vec!["zz".into()]);
    words.set("empty", "ignored", vec![]);
    BipartiteDiGraph::from_word_map(&words, ChangeRule(0))
}

#[test]
fn move_views_follow_alternating_transformation_and_word_edges() {
    let graph = chain_graph();
    assert_eq!(graph.nodes(1), vec!["a", "b", "c", "z"]);
    assert_eq!(graph.nodes(0), vec!["b", "c", "z"]);
    assert_eq!(
        graph.get_moves_from_node("b", 0, 0, None),
        vec![pair("b", "c")]
    );
    assert_eq!(
        graph.get_moves_from_node("b", 0, 1, None),
        vec![pair("a", "b"), pair("c", "b")]
    );
    assert_eq!(
        graph.get_moves_from_node("a", 1, 0, None),
        vec![pair("a", "b")]
    );
    assert_eq!(
        graph.get_move_view_nodes(0, "a", 0, Some(ChangeRule(0))),
        vec!["a"]
    );
    assert_eq!(graph.next_words_limit_nodes(1, 2), vec!["b", "c", "z"]);
}

#[test]
fn reachable_induced_and_scc_keep_components_separate() {
    let graph = chain_graph();
    let reachable = graph.get_reachable_nodes(1, "a", None);
    assert_eq!(
        reachable[0].iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["b", "c"]
    );
    assert_eq!(
        reachable[1].iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    let induced = graph.get_induced_subgraph(&reachable);
    assert_eq!(
        induced.edges(1),
        vec![
            ("a".into(), "b".into(), 2),
            ("b".into(), "c".into(), 1),
            ("c".into(), "b".into(), 1)
        ]
    );
    let scc = graph.get_scc_map();
    assert_eq!(scc[0]["b"], scc[1]["c"]);
    assert_eq!(scc[0]["c"], scc[1]["b"]);
    assert_ne!(scc[1]["a"], scc[0]["b"]);
    assert_ne!(scc[0]["z"], scc[0]["b"]);
    let (dag, _, members) = graph.condensation(&scc, 1);
    assert_eq!(dag.edges().len(), 1);
    assert_eq!(members.len(), 3);
    let nodes = dag.sort_by_distance_from_sink();
    assert_eq!(members[nodes[0]], vec!["a"]);
}

#[test]
fn removable_loops_and_cycles_respect_shared_word_multiplicities() {
    let mut graph = BipartiteDiGraph::new();
    graph.set_edge(0, "x", "x", 1);
    graph.set_edge(0, "y", "y", 1);
    graph.set_edge(1, "x", "x", 5);
    graph.set_edge(1, "x", "y", 3);
    graph.set_edge(1, "y", "x", 2);
    assert_eq!(graph.get_even_loops(), vec![("x".into(), "x".into(), 4)]);
    assert_eq!(
        graph.get_two_cycles(),
        vec![(pair("x", "y"), pair("y", "x"), 2)]
    );
    graph.set_edge(1, "x", "x", 1);
    graph.set_edge(1, "x", "y", 1);
    graph.set_edge(1, "y", "x", 1);
    assert_eq!(
        graph.get_critical_edges(),
        vec![pair("x", "y"), pair("y", "x")]
    );
}

#[test]
fn partition_transfers_preserve_global_counts_offsets_and_union() {
    let mut graph = BipartiteDiGraph::new();
    graph.set_edge(0, "b", "b", 1);
    graph.set_edge(1, "a", "b", 5);
    graph.set_edge(1, "b", "b", 1);
    let mut partitions = GraphPartitions::new(graph);
    assert_eq!(partitions.keys, vec!["removed", "winlose", "route"]);
    partitions
        .transfer_edge("route", "removed", "a", "b", 2)
        .unwrap();
    partitions
        .transfer_edge("route", "winlose", "a", "b", 1)
        .unwrap();
    assert_eq!(partitions.get_edge_num_and_offset("a", "b", None), (5, 0));
    assert_eq!(
        partitions.get_edge_idx_range("a", "b", Some("route")),
        (3, 5)
    );
    assert_eq!(
        partitions.get_moves_from_node("b", 0, 1, None, None),
        vec![pair("a", "b"), pair("b", "b")]
    );
    assert_eq!(partitions.union().get_edge_num("a", "b"), 5);
    partitions
        .transfer_edge("winlose", "route", "a", "b", 1)
        .unwrap();
    partitions.transfer_node("route", "winlose", 0, "b");
    assert!(!partitions.get_graph("route").has_node(0, "b"));
    assert_eq!(partitions.union().get_edge_num("a", "b"), 5);
    let one: [IndexSet<String>; 2] = [
        IndexSet::from_iter(["b".into()]),
        IndexSet::from_iter(["b".into()]),
    ];
    assert_eq!(
        partitions
            .union()
            .get_induced_subgraph(&one)
            .get_edge_num("b", "b"),
        1
    );
}

#[test]
fn partition_union_uses_route_first_while_offsets_use_removed_first() {
    let mut graph = BipartiteDiGraph::new();
    graph.set_edge(1, "a", "b", 2);
    graph.set_edge(1, "z", "b", 1);
    let mut partitions = GraphPartitions::new(graph);
    partitions
        .transfer_edge("route", "removed", "a", "b", 2)
        .unwrap();
    assert_eq!(
        partitions.union().edges(1),
        vec![("z".into(), "b".into(), 1), ("a".into(), "b".into(), 2)]
    );
    assert_eq!(
        partitions.get_edge_idx_range("a", "b", Some("removed")),
        (0, 2)
    );
    assert_eq!(
        partitions.get_edge_idx_range("a", "b", Some("route")),
        (2, 2)
    );
}
