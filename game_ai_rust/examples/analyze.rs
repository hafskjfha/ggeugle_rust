use game_ai_rust::{
    RuleForm, get_wc_data,
    rules::{FileWordsOption, WordSource},
};
use std::time::Instant;

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
    rule.content.word_rule.words = WordSource::File(FileWordsOption { path });
    let start = Instant::now();
    let solver = get_wc_data(&rule, flow)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "words":solver.word_map.get_size(),
            "distinctPairs":solver.word_map.to_array().len(),
            "change":change,"flow":flow,"seconds":start.elapsed().as_secs_f64(),
            "nodeTypes":[solver.graph_solver.get_node_type_num(0),solver.graph_solver.get_node_type_num(1)],
            "wordTypes":solver.graph_solver.get_word_type_num(),
            "route":solver.graph_solver.get_max_route_info(0)
        }))?
    );
    Ok(())
}
