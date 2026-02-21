import {
  classify_json as wasmClassifyJson,
  decide_transition as wasmDecideTransition,
  get_all_protocols as wasmGetAllProtocols,
  is_terminal as wasmIsTerminal,
  normalize_snapshot as wasmNormalizeSnapshot,
  wasm_event_type_to_transition as wasmEventTypeToTransition,
  wasm_transition_to_string as wasmTransitionToString,
  wasm_transition_target as wasmTransitionTarget,
} from '../wasm-pkg/defi_tracker_lifecycle';

export {
  wasmClassifyJson,
  wasmDecideTransition,
  wasmGetAllProtocols,
  wasmIsTerminal,
  wasmNormalizeSnapshot,
  wasmEventTypeToTransition,
  wasmTransitionToString,
  wasmTransitionTarget,
};
