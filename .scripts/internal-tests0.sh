#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt
$rt clippy -Aclippy::as_conversions,-Aclippy::cfg_not_test,-Aclippy::float_arithmetic,-Aclippy::modulo_arithmetic,-Aclippy::arbitrary_source_item_ordering,-Aclippy::doc-include-without-cfg,-Aclippy::little-endian-bytes,-Aclippy::panic-in-result-fn,-Aclippy::return_and_then,-Aclippy::used_underscore_items,-Aclippy::doc_paragraphs_missing_punctuation,-Aclippy::map_with_unused_argument_over_ranges

MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features http2,postgres,web-socket -p wtx

