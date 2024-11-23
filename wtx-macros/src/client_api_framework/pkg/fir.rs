//! First Intermediate Representation (FIR)
//!
//! Performs basic input validation, extracts inner elements or modifies simple structures.

#[macro_use]
pub(crate) mod fir_custom_item_values;
#[macro_use]
pub(crate) mod fir_hook_item_values;

pub(crate) mod fir_after_sending_item_values;
pub(crate) mod fir_aux_field_attr;
pub(crate) mod fir_aux_item_values;
pub(crate) mod fir_before_sending_item_values;
pub(crate) mod fir_custom_field_attr;
pub(crate) mod fir_custom_field_field_attr;
pub(crate) mod fir_item_attr;
pub(crate) mod fir_items_values;
pub(crate) mod fir_params_items_values;
pub(crate) mod fir_pkg_attr;
pub(crate) mod fir_req_item_values;
pub(crate) mod fir_res_item_values;
