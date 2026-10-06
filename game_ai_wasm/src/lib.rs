//! Browser bindings. Each Web Worker creates its own independent WASM instance.
use game_ai_rust as core;
use js_sys::Function;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;
use wasm_bindgen::prelude::*;

fn js_error(error: core::Error) -> JsValue {
    let name = if matches!(error, core::Error::Timeout) {
        "TimeoutError"
    } else {
        "Error"
    };
    let result = js_sys::Error::new(&error.to_string());
    result.set_name(name);
    result.into()
}

fn decode<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value)
        .map_err(|error| js_error(core::Error::InvalidInput(error.to_string())))
}

fn encode(value: &impl Serialize) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|error| js_error(core::Error::InvalidInput(error.to_string())))
}

fn timeout(milliseconds: Option<f64>) -> core::Result<Option<Duration>> {
    milliseconds
        .map(|value| {
            if !value.is_finite() || value < 0.0 {
                return Err(core::Error::InvalidInput(
                    "timeoutMillis must be finite and nonnegative".into(),
                ));
            }
            Duration::try_from_secs_f64(value / 1000.0)
                .map_err(|_| core::Error::InvalidInput("timeoutMillis is too large".into()))
        })
        .transpose()
}

fn dispatch(callback: &Function, event: &impl Serialize, callback_error: &mut Option<JsValue>) {
    if callback_error.is_none() {
        let result =
            encode(event).and_then(|payload| callback.call1(&JsValue::UNDEFINED, &payload));
        if let Err(error) = result {
            *callback_error = Some(error);
        }
    }
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct BrowserAiOptions {
    difficulty: usize,
    calculating_duration: f64,
    stealable: bool,
    precedence: core::PrecInfo,
    flow: usize,
}

impl Default for BrowserAiOptions {
    fn default() -> Self {
        Self {
            difficulty: 2,
            calculating_duration: 1000.0,
            stealable: false,
            precedence: core::PrecInfo::default(),
            flow: 0,
        }
    }
}

impl BrowserAiOptions {
    fn into_core(self) -> core::Result<core::AiOptions> {
        Ok(core::AiOptions {
            difficulty: self.difficulty,
            calculating_duration: timeout(Some(self.calculating_duration))?.unwrap(),
            stealable: self.stealable,
            precedence: self.precedence,
            flow: self.flow,
        })
    }
}

#[wasm_bindgen(js_name = getWcData)]
pub fn get_wc_data(rule: JsValue, flow: usize) -> Result<JsValue, JsValue> {
    let rule = decode::<core::RuleForm>(rule)?;
    let solver = core::get_wc_data(&rule, flow).map_err(js_error)?;
    encode(&solver)
}

#[wasm_bindgen(js_name = updateSolver)]
pub fn update_solver(graphs: JsValue, moves: JsValue, flow: usize) -> Result<JsValue, JsValue> {
    let graphs = decode::<core::GraphPartitions>(graphs)?;
    let moves = decode::<Vec<core::Edge>>(moves)?;
    encode(&core::update_solver(&graphs, &moves, flow).map_err(js_error)?)
}

#[wasm_bindgen(js_name = getGraph)]
pub fn get_graph(graphs: JsValue) -> Result<JsValue, JsValue> {
    let graphs = decode::<core::GraphPartitions>(graphs)?;
    encode(&graphs.union())
}

#[wasm_bindgen(js_name = getNextWords)]
pub fn get_next_words(solver: JsValue, history: JsValue) -> Result<JsValue, JsValue> {
    let solver = decode::<core::WordSolver>(solver)?;
    let history = decode::<Vec<String>>(history)?;
    encode(&solver.get_next_words(&history).map_err(js_error)?)
}

#[wasm_bindgen(js_name = afterHistory)]
pub fn after_history(solver: JsValue, history: JsValue, flow: usize) -> Result<JsValue, JsValue> {
    let solver = decode::<core::WordSolver>(solver)?;
    let history = decode::<Vec<String>>(history)?;
    if flow > 1 {
        return Err(js_error(core::Error::InvalidInput(
            "flow must be 0 or 1".into(),
        )));
    }
    encode(&solver.after_history(&history, flow).map_err(js_error)?)
}

#[wasm_bindgen(js_name = chooseMove)]
pub fn choose_move(
    solver: JsValue,
    history: JsValue,
    options: JsValue,
    callback: Option<Function>,
) -> Result<JsValue, JsValue> {
    let solver = decode::<core::WordSolver>(solver)?;
    let history = decode::<Vec<String>>(history)?;
    let options = if options.is_null() || options.is_undefined() {
        BrowserAiOptions::default()
    } else {
        decode::<BrowserAiOptions>(options)?
    };
    let options = options.into_core().map_err(js_error)?;
    let mut callback_error = None;
    let result = core::choose_move_with_callback(&solver, &history, &options, |event| {
        if let Some(callback) = &callback {
            dispatch(callback, &event, &mut callback_error);
        }
    });
    if let Some(error) = callback_error {
        return Err(error);
    }
    encode(&result.map_err(js_error)?)
}

#[wasm_bindgen(js_name = isGameEnd)]
pub fn is_game_end(solver: JsValue, history: JsValue, stealable: bool) -> Result<bool, JsValue> {
    let solver = decode::<core::WordSolver>(solver)?;
    let history = decode::<Vec<String>>(history)?;
    core::is_game_end(&solver, &history, stealable).map_err(js_error)
}

#[wasm_bindgen(js_name = searchIsWin)]
pub fn search_is_win(
    graph: JsValue,
    movement: JsValue,
    prec: JsValue,
    timeout_millis: Option<f64>,
) -> Result<JsValue, JsValue> {
    let graph = decode::<core::BipartiteDiGraph>(graph)?;
    let movement = decode::<core::SingleMove>(movement)?;
    let prec = decode::<core::PrecInfo>(prec)?;
    let limit = timeout(timeout_millis).map_err(js_error)?;
    encode(&core::search_is_win(&graph, &movement, &prec, limit).map_err(js_error)?)
}

#[wasm_bindgen(js_name = prepareRootSearch)]
pub fn prepare_root_search(
    graph: JsValue,
    movement: JsValue,
    prec: JsValue,
    timeout_millis: Option<f64>,
) -> Result<JsValue, JsValue> {
    let graph = decode::<core::BipartiteDiGraph>(graph)?;
    let movement = decode::<core::SingleMove>(movement)?;
    let prec = decode::<core::PrecInfo>(prec)?;
    let limit = timeout(timeout_millis).map_err(js_error)?;
    encode(&core::prepare_root_search(&graph, &movement, &prec, limit).map_err(js_error)?)
}

#[wasm_bindgen(js_name = prepareSyllableSearch)]
pub fn prepare_syllable_search(
    graph_solver: JsValue,
    syllable: &str,
    change_func_idx: usize,
    prec: JsValue,
    timeout_millis: Option<f64>,
) -> Result<JsValue, JsValue> {
    let solver = decode::<core::GraphSolver>(graph_solver)?;
    let prec = decode::<core::PrecInfo>(prec)?;
    let limit = timeout(timeout_millis).map_err(js_error)?;
    encode(
        &core::prepare_syllable_search(&solver, syllable, change_func_idx, &prec, limit)
            .map_err(js_error)?,
    )
}

#[wasm_bindgen(js_name = searchSyllable)]
pub fn search_syllable(
    graph_solver: JsValue,
    syllable: &str,
    change_func_idx: usize,
    prec: JsValue,
    timeout_millis: Option<f64>,
) -> Result<JsValue, JsValue> {
    let solver = decode::<core::GraphSolver>(graph_solver)?;
    let prec = decode::<core::PrecInfo>(prec)?;
    let limit = timeout(timeout_millis).map_err(js_error)?;
    encode(
        &core::search_syllable(&solver, syllable, change_func_idx, &prec, limit)
            .map_err(js_error)?,
    )
}

#[wasm_bindgen(js_name = searchRootBranch)]
pub fn search_root_branch(
    graph: JsValue,
    movement: JsValue,
    prec: JsValue,
    timeout_millis: Option<f64>,
    callback: Option<Function>,
) -> Result<JsValue, JsValue> {
    let graph = decode::<core::BipartiteDiGraph>(graph)?;
    let movement = decode::<core::SingleMove>(movement)?;
    let prec = decode::<core::PrecInfo>(prec)?;
    let limit = timeout(timeout_millis).map_err(js_error)?;
    let mut callback_error = None;
    let result = core::search_root_branch(&graph, &movement, &prec, limit, |event| {
        if let Some(callback) = &callback {
            dispatch(callback, &event, &mut callback_error);
        }
    });
    if let Some(error) = callback_error {
        return Err(error);
    }
    encode(&result.map_err(js_error)?)
}

#[wasm_bindgen(js_name = finishRootSearch)]
pub fn finish_root_search(
    plan: JsValue,
    results: JsValue,
    duration: f64,
) -> Result<JsValue, JsValue> {
    let plan = decode::<core::RootSearchPlan>(plan)?;
    let results = decode::<Vec<Option<core::RootBranchResult>>>(results)?;
    encode(&core::finish_root_search(&plan, &results, duration).map_err(js_error)?)
}

#[wasm_bindgen(js_name = startStreamingSingleThreadSearch)]
pub fn start_streaming_single_thread_search(
    graph: JsValue,
    movement: JsValue,
    prec: JsValue,
    timeout_millis: Option<f64>,
    callback: Function,
) -> Result<JsValue, JsValue> {
    let graph = decode::<core::BipartiteDiGraph>(graph)?;
    let movement = decode::<core::SingleMove>(movement)?;
    let prec = decode::<core::PrecInfo>(prec)?;
    let limit = timeout(timeout_millis).map_err(js_error)?;
    let mut callback_error = None;
    let branch = core::search_root_branch(&graph, &movement, &prec, limit, |event| {
        dispatch(&callback, &event, &mut callback_error);
    });
    if let Some(error) = callback_error {
        return Err(error);
    }
    let result = branch.map_err(js_error)?.result;
    dispatch(
        &callback,
        &core::SearchEvent::Done {
            payload: result.clone(),
        },
        &mut callback_error,
    );
    if let Some(error) = callback_error {
        return Err(error);
    }
    encode(&result)
}

#[wasm_bindgen(js_name = startStreamingCriticalWordsInfo)]
pub fn start_streaming_critical_words_info(
    graph: JsValue,
    view: usize,
    flow: usize,
    callback: Function,
) -> Result<(), JsValue> {
    let graph = decode::<core::BipartiteDiGraph>(graph)?;
    let mut callback_error = None;
    let result = core::start_streaming_critical_words_info(&graph, view, flow, |event| {
        dispatch(&callback, &event, &mut callback_error);
    });
    if let Some(error) = callback_error {
        return Err(error);
    }
    result.map_err(js_error)
}

#[wasm_bindgen(js_name = dictionaryUrls)]
pub fn dictionary_urls(option: JsValue) -> Result<JsValue, JsValue> {
    let option = decode::<core::rules::SelectedWordsOption>(option)?;
    encode(&core::rules::dictionary_urls(&option).map_err(js_error)?)
}

#[wasm_bindgen(js_name = getPresets)]
pub fn get_presets() -> Result<JsValue, JsValue> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Presets<R, P, C, D> {
        rules: R,
        precedence_maps: P,
        change_func_info: C,
        dicts: D,
    }
    encode(&Presets {
        rules: core::presets::sample_rules(),
        precedence_maps: core::presets::sample_precedence_maps(),
        change_func_info: core::presets::sample_change_func_info(),
        dicts: core::presets::sample_dicts(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timeout_values_preserve_zero_and_reject_invalid_browser_numbers() {
        assert_eq!(timeout(None).unwrap(), None);
        assert_eq!(timeout(Some(0.0)).unwrap(), Some(Duration::ZERO));
        assert_eq!(
            timeout(Some(1500.5)).unwrap(),
            Some(Duration::from_micros(1_500_500))
        );
        for value in [-1.0, f64::NAN, f64::INFINITY, f64::MAX] {
            assert!(timeout(Some(value)).is_err(), "value={value}");
        }
    }

    #[test]
    fn ai_options_use_millisecond_deadlines_and_reject_invalid_limits() {
        let defaults = BrowserAiOptions::default().into_core().unwrap();
        assert_eq!(defaults.calculating_duration, Duration::from_secs(1));
        let custom = BrowserAiOptions {
            calculating_duration: 50.5,
            ..Default::default()
        }
        .into_core()
        .unwrap();
        assert_eq!(custom.calculating_duration, Duration::from_micros(50_500));
        assert!(
            BrowserAiOptions {
                calculating_duration: -1.0,
                ..Default::default()
            }
            .into_core()
            .is_err()
        );
    }
}
