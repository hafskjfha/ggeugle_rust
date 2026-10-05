use game_ai_rust::{AiOptions, GameEvent, RuleForm, choose_move_with_callback, get_wc_data};

fn main() -> game_ai_rust::Result<()> {
    // These words are present in the original data/all_words.txt.
    let solver = get_wc_data(&RuleForm::manual(&["사과", "과자", "자두"], 0), 0)?;
    let history = vec!["사과".to_string()];
    choose_move_with_callback(&solver, &history, &AiOptions::default(), |event| {
        if let GameEvent::Move { payload } = event {
            println!("AI: {payload}");
        }
    })?;
    Ok(())
}
