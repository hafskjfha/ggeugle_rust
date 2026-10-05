//! Print the initial dictionary's syllable and individual-word classifications.
use game_ai_rust::{
    NodeType, RuleForm, get_wc_data,
    rules::{FileWordsOption, WordSource},
};

fn node_label(kind: NodeType) -> &'static str {
    match kind {
        NodeType::Win => "승",
        NodeType::LoopWin => "루프승",
        NodeType::Lose => "패",
        NodeType::Route => "루트(미확정)",
    }
}

fn word_label(kind: usize) -> &'static str {
    match kind {
        0 => "공격(승)",
        1 => "루트(미확정)",
        2 => "돌림(짝 제거)",
        3 => "방어(패)",
        4 => "공뤁(루트 연결)",
        5 => "양보(패)",
        _ => unreachable!("the solver has six word classes"),
    }
}

fn depth_label(depth: Option<usize>) -> String {
    depth.map_or_else(|| "-".to_string(), |depth| depth.to_string())
}

fn main() -> game_ai_rust::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .unwrap_or_else(|| format!("{}/tests/fixtures/words.txt", env!("CARGO_MANIFEST_DIR")));
    let change = args
        .next()
        .unwrap_or_else(|| "1".into())
        .parse::<usize>()
        .map_err(|_| game_ai_rust::Error::InvalidInput("change index must be an integer".into()))?;
    let flow = args
        .next()
        .unwrap_or_else(|| "0".into())
        .parse::<usize>()
        .map_err(|_| game_ai_rust::Error::InvalidInput("flow must be an integer".into()))?;

    let mut rule = RuleForm::manual(&[], change);
    rule.content.word_rule.words = WordSource::File(FileWordsOption { path: path.clone() });
    let solver = get_wc_data(&rule, flow)?;
    let graph = &solver.graph_solver;

    println!("사전: {path}");
    println!(
        "단어: {}개 / 음절 변환: {change} / 분류 순서: {flow}",
        solver.word_map.get_size()
    );
    println!("음절 판정은 그 음절에서 차례를 시작하는 플레이어 기준입니다.");
    println!("단어 분류는 사용 이력이 없는 초기 사전의 정적 분석입니다.");
    println!(
        "루트는 추가 필승 탐색이 필요한 상태입니다. 깊이는 돌림 쌍을 제거한 승패 그래프의 단어 수 기준입니다."
    );
    println!("깊이 '-'는 정의된 값이 없다는 뜻입니다.");

    println!("\n단어 분류 요약");
    for (kind, count) in graph.get_word_type_num().type_num.into_iter().enumerate() {
        println!("{}: {count}개", word_label(kind));
    }

    for view in [0, 1] {
        println!(
            "\n{}",
            if view == 0 {
                "받는 음절 (앞 단어의 끝 음절)"
            } else {
                "제시 음절 (낼 단어의 첫 음절)"
            }
        );
        for kind in [
            NodeType::Win,
            NodeType::LoopWin,
            NodeType::Lose,
            NodeType::Route,
        ] {
            let count = graph.type_map[view]
                .values()
                .filter(|&&value| value == kind)
                .count();
            println!("{}: {count}개", node_label(kind));
        }
        println!("음절\t판정\t깊이");
        let mut nodes: Vec<_> = graph.type_map[view].iter().collect();
        nodes.sort_by_key(|(node, _)| *node);
        for (node, &kind) in nodes {
            println!(
                "{node}\t{}\t{}",
                node_label(kind),
                depth_label(graph.depth_map[view].get(node).copied())
            );
        }
    }

    println!("\n개별 단어 분석");
    println!("단어\t분류\t첫 음절 판정\t끝 음절 판정\t깊이\t돌림 짝");
    let mut rows = Vec::new();
    for (head, tail, words) in solver.word_map.to_array() {
        for (index, word) in words.into_iter().enumerate() {
            let info = graph.get_move_type(&head, &tail, index);
            let pair = info
                .pair
                .as_ref()
                .and_then(|(head, tail, index)| solver.word_map.get_word(head, tail, *index))
                .unwrap_or("-");
            rows.push((
                word,
                format!(
                    "{}\t{}\t{}\t{}\t{pair}",
                    word_label(info.move_type),
                    node_label(graph.get_node_type(&head, 1, None)),
                    node_label(graph.get_node_type(&tail, 0, None)),
                    depth_label(info.depth)
                ),
            ));
        }
    }
    rows.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (word, info) in rows {
        println!("{word}\t{info}");
    }
    Ok(())
}
