import {
  decide_transition as wasmDecideTransition,
  get_all_protocols as wasmGetAllProtocols,
  is_terminal as wasmIsTerminal,
  lookup_variant as wasmLookupVariant,
  normalize_snapshot as wasmNormalizeSnapshot,
  wasm_event_type_to_transition as wasmEventTypeToTransition,
  wasm_transition_to_string as wasmTransitionToString,
  wasm_transition_target as wasmTransitionTarget,
} from '../wasm-pkg/defi_tracker_lifecycle';

export {
  wasmDecideTransition,
  wasmGetAllProtocols,
  wasmIsTerminal,
  wasmLookupVariant,
  wasmNormalizeSnapshot,
  wasmEventTypeToTransition,
  wasmTransitionToString,
  wasmTransitionTarget,
};
