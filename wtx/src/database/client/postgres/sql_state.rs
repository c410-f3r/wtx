_create_enum! {
  /// SQLSTATE error code
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum SqlState<u16> {
    /// successful_completion
    E00000 = (0, "00000"),
    /// warning
    E01000 = (1, "01000"),
    /// null_value_eliminated_in_set_function
    E01003 = (2, "01003"),
    /// string_data_right_truncation
    E01004 = (3, "01004"),
    /// privilege_not_revoked
    E01006 = (4, "01006"),
    /// privilege_not_granted
    E01007 = (5, "01007"),
    /// implicit_zero_bit_padding
    E01008 = (6, "01008"),
    /// dynamic_result_sets_returned
    E0100C = (7, "0100C"),
    /// deprecated_feature
    E01P01 = (8, "01P01"),
    /// no_data
    E02000 = (9, "02000"),
    /// no_additional_dynamic_result_sets_returned
    E02001 = (10, "02001"),
    /// sql_statement_not_yet_complete
    E03000 = (11, "03000"),
    /// connection_exception
    E08000 = (12, "08000"),
    /// sqlclient_unable_to_establish_sqlconnection
    E08001 = (13, "08001"),
    /// connection_does_not_exist
    E08003 = (14, "08003"),
    /// sqlserver_rejected_establishment_of_sqlconnection
    E08004 = (15, "08004"),
    /// connection_failure
    E08006 = (16, "08006"),
    /// transaction_resolution_unknown
    E08007 = (17, "08007"),
    /// protocol_violation
    E08P01 = (18, "08P01"),
    /// triggered_action_exception
    E09000 = (19, "09000"),
    /// feature_not_supported
    E0A000 = (20, "0A000"),
    /// invalid_transaction_initiation
    E0B000 = (21, "0B000"),
    /// locator_exception
    E0F000 = (22, "0F000"),
    /// invalid_locator_specification
    E0F001 = (23, "0F001"),
    /// invalid_grantor
    E0L000 = (24, "0L000"),
    /// invalid_grant_operation
    E0LP01 = (25, "0LP01"),
    /// invalid_role_specification
    E0P000 = (26, "0P000"),
    /// diagnostics_exception
    E0Z000 = (27, "0Z000"),
    /// stacked_diagnostics_accessed_without_active_handler
    E0Z002 = (28, "0Z002"),
    /// case_not_found
    E20000 = (29, "20000"),
    /// cardinality_violation
    E21000 = (30, "21000"),
    /// data_exception
    E22000 = (31, "22000"),
    /// string_data_right_truncation
    E22001 = (32, "22001"),
    /// null_value_no_indicator_parameter
    E22002 = (33, "22002"),
    /// numeric_value_out_of_range
    E22003 = (34, "22003"),
    /// null_value_not_allowed
    E22004 = (35, "22004"),
    /// error_in_assignment
    E22005 = (36, "22005"),
    /// invalid_datetime_format
    E22007 = (37, "22007"),
    /// datetime_field_overflow
    E22008 = (38, "22008"),
    /// invalid_time_zone_displacement_value
    E22009 = (39, "22009"),
    /// escape_character_conflict
    E2200B = (40, "2200B"),
    /// invalid_use_of_escape_character
    E2200C = (41, "2200C"),
    /// invalid_escape_octet
    E2200D = (42, "2200D"),
    /// zero_length_character_string
    E2200F = (43, "2200F"),
    /// most_specific_type_mismatch
    E2200G = (44, "2200G"),
    /// sequence_generator_limit_exceeded
    E2200H = (45, "2200H"),
    /// not_an_xml_document
    E2200L = (46, "2200L"),
    /// invalid_xml_document
    E2200M = (47, "2200M"),
    /// invalid_xml_content
    E2200N = (48, "2200N"),
    /// invalid_xml_comment
    E2200S = (49, "2200S"),
    /// invalid_xml_processing_instruction
    E2200T = (50, "2200T"),
    /// invalid_indicator_parameter_value
    E22010 = (51, "22010"),
    /// substring_error
    E22011 = (52, "22011"),
    /// division_by_zero
    E22012 = (53, "22012"),
    /// invalid_preceding_or_following_size
    E22013 = (54, "22013"),
    /// invalid_argument_for_ntile_function
    E22014 = (55, "22014"),
    /// interval_field_overflow
    E22015 = (56, "22015"),
    /// invalid_argument_for_nth_value_function
    E22016 = (57, "22016"),
    /// invalid_character_value_for_cast
    E22018 = (58, "22018"),
    /// invalid_escape_character
    E22019 = (59, "22019"),
    /// invalid_regular_expression
    E2201B = (60, "2201B"),
    /// invalid_argument_for_logarithm
    E2201E = (61, "2201E"),
    /// invalid_argument_for_power_function
    E2201F = (62, "2201F"),
    /// invalid_argument_for_width_bucket_function
    E2201G = (63, "2201G"),
    /// invalid_row_count_in_limit_clause
    E2201W = (64, "2201W"),
    /// invalid_row_count_in_result_offset_clause
    E2201X = (65, "2201X"),
    /// character_not_in_repertoire
    E22021 = (66, "22021"),
    /// indicator_overflow
    E22022 = (67, "22022"),
    /// invalid_parameter_value
    E22023 = (68, "22023"),
    /// unterminated_c_string
    E22024 = (69, "22024"),
    /// invalid_escape_sequence
    E22025 = (70, "22025"),
    /// string_data_length_mismatch
    E22026 = (71, "22026"),
    /// trim_error
    E22027 = (72, "22027"),
    /// array_subscript_error
    E2202E = (73, "2202E"),
    /// invalid_tablesample_repeat
    E2202G = (74, "2202G"),
    /// invalid_tablesample_argument
    E2202H = (75, "2202H"),
    /// duplicate_json_object_key_value
    E22030 = (76, "22030"),
    /// invalid_argument_for_sql_json_datetime_function
    E22031 = (77, "22031"),
    /// invalid_json_text
    E22032 = (78, "22032"),
    /// invalid_sql_json_subscript
    E22033 = (79, "22033"),
    /// more_than_one_sql_json_item
    E22034 = (80, "22034"),
    /// no_sql_json_item
    E22035 = (81, "22035"),
    /// non_numeric_sql_json_item
    E22036 = (82, "22036"),
    /// non_unique_keys_in_a_json_object
    E22037 = (83, "22037"),
    /// singleton_sql_json_item_required
    E22038 = (84, "22038"),
    /// sql_json_array_not_found
    E22039 = (85, "22039"),
    /// sql_json_member_not_found
    E2203A = (86, "2203A"),
    /// sql_json_number_not_found
    E2203B = (87, "2203B"),
    /// sql_json_object_not_found
    E2203C = (88, "2203C"),
    /// too_many_json_array_elements
    E2203D = (89, "2203D"),
    /// too_many_json_object_members
    E2203E = (90, "2203E"),
    /// sql_json_scalar_required
    E2203F = (91, "2203F"),
    /// sql_json_item_cannot_be_cast_to_target_type
    E2203G = (92, "2203G"),
    /// floating_point_exception
    E22P01 = (93, "22P01"),
    /// invalid_text_representation
    E22P02 = (94, "22P02"),
    /// invalid_binary_representation
    E22P03 = (95, "22P03"),
    /// bad_copy_file_format
    E22P04 = (96, "22P04"),
    /// untranslatable_character
    E22P05 = (97, "22P05"),
    /// nonstandard_use_of_escape_character
    E22P06 = (98, "22P06"),
    /// 23000
    E23000 = (99, "23000"),
    /// restrict_violation
    E23001 = (100, "23001"),
    /// not_null_violation
    E23502 = (101, "23502"),
    /// foreign_key_violation
    E23503 = (102, "23503"),
    /// unique_violation
    E23505 = (103, "23505"),
    /// check_violation
    E23514 = (104, "23514"),
    /// exclusion_violation
    E23P01 = (105, "23P01"),
    /// invalid_cursor_state
    E24000 = (106, "24000"),
    /// invalid_transaction_state
    E25000 = (107, "25000"),
    /// active_sql_transaction
    E25001 = (108, "25001"),
    /// branch_transaction_already_active
    E25002 = (109, "25002"),
    /// inappropriate_access_mode_for_branch_transaction
    E25003 = (110, "25003"),
    /// inappropriate_isolation_level_for_branch_transaction
    E25004 = (111, "25004"),
    /// no_active_sql_transaction_for_branch_transaction
    E25005 = (112, "25005"),
    /// read_only_sql_transaction
    E25006 = (113, "25006"),
    /// schema_and_data_statement_mixing_not_supported
    E25007 = (114, "25007"),
    /// held_cursor_requires_same_isolation_level
    E25008 = (115, "25008"),
    /// no_active_sql_transaction
    E25P01 = (116, "25P01"),
    /// in_failed_sql_transaction
    E25P02 = (117, "25P02"),
    /// idle_in_transaction_session_timeout
    E25P03 = (118, "25P03"),
    /// invalid_sql_statement_name
    E26000 = (119, "26000"),
    /// triggered_data_change_violation
    E27000 = (120, "27000"),
    /// invalid_authorization_specification
    E28000 = (121, "28000"),
    /// invalid_password
    E28P01 = (122, "28P01"),
    /// dependent_privilege_descriptors_still_exist
    E2B000 = (123, "2B000"),
    /// dependent_objects_still_exist
    E2BP01 = (124, "2BP01"),
    /// invalid_transaction_termination
    E2D000 = (125, "2D000"),
    /// sql_routine_exception
    E2F000 = (126, "2F000"),
    /// modifying_sql_data_not_permitted
    E2F002 = (127, "2F002"),
    /// prohibited_sql_statement_attempted
    E2F003 = (128, "2F003"),
    /// reading_sql_data_not_permitted
    E2F004 = (129, "2F004"),
    /// function_executed_no_return_statement
    E2F005 = (130, "2F005"),
    /// invalid_cursor_name
    E34000 = (131, "34000"),
    /// external_routine_exception
    E38000 = (132, "38000"),
    /// containing_sql_not_permitted
    E38001 = (133, "38001"),
    /// modifying_sql_data_not_permitted
    E38002 = (134, "38002"),
    /// prohibited_sql_statement_attempted
    E38003 = (135, "38003"),
    /// reading_sql_data_not_permitted
    E38004 = (136, "38004"),
    /// external_routine_invocation_exception
    E39000 = (137, "39000"),
    /// invalid_sqlstate_returned
    E39001 = (138, "39001"),
    /// null_value_not_allowed
    E39004 = (139, "39004"),
    /// trigger_protocol_violated
    E39P01 = (140, "39P01"),
    /// srf_protocol_violated
    E39P02 = (141, "39P02"),
    /// event_trigger_protocol_violated
    E39P03 = (142, "39P03"),
    /// savepoint_exception
    E3B000 = (143, "3B000"),
    /// invalid_savepoint_specification
    E3B001 = (144, "3B001"),
    /// invalid_catalog_name
    E3D000 = (145, "3D000"),
    /// invalid_schema_name
    E3F000 = (146, "3F000"),
    /// transaction_rollback
    E40000 = (147, "40000"),
    /// serialization_failure
    E40001 = (148, "40001"),
    /// transaction_integrity_constraint_violation
    E40002 = (149, "40002"),
    /// statement_completion_unknown
    E40003 = (150, "40003"),
    /// deadlock_detected
    E40P01 = (151, "40P01"),
    /// syntax_error_or_access_rule_violation
    E42000 = (152, "42000"),
    /// insufficient_privilege
    E42501 = (153, "42501"),
    /// syntax_error
    E42601 = (154, "42601"),
    /// invalid_name
    E42602 = (155, "42602"),
    /// invalid_column_definition
    E42611 = (156, "42611"),
    /// name_too_long
    E42622 = (157, "42622"),
    /// duplicate_column
    E42701 = (158, "42701"),
    /// ambiguous_column
    E42702 = (159, "42702"),
    /// undefined_column
    E42703 = (160, "42703"),
    /// undefined_object
    E42704 = (161, "42704"),
    /// duplicate_object
    E42710 = (162, "42710"),
    /// duplicate_alias
    E42712 = (163, "42712"),
    /// duplicate_function
    E42723 = (164, "42723"),
    /// ambiguous_function
    E42725 = (165, "42725"),
    /// grouping_error
    E42803 = (166, "42803"),
    /// datatype_mismatch
    E42804 = (167, "42804"),
    /// wrong_object_type
    E42809 = (168, "42809"),
    /// invalid_foreign_key
    E42830 = (169, "42830"),
    /// cannot_coerce
    E42846 = (170, "42846"),
    /// undefined_function
    E42883 = (171, "42883"),
    /// generated_always
    E428C9 = (172, "428C9"),
    /// reserved_name
    E42939 = (173, "42939"),
    /// undefined_table
    E42P01 = (174, "42P01"),
    /// undefined_parameter
    E42P02 = (175, "42P02"),
    /// duplicate_cursor
    E42P03 = (176, "42P03"),
    /// duplicate_database
    E42P04 = (177, "42P04"),
    /// duplicate_prepared_statement
    E42P05 = (178, "42P05"),
    /// duplicate_schema
    E42P06 = (179, "42P06"),
    /// duplicate_table
    E42P07 = (180, "42P07"),
    /// ambiguous_parameter
    E42P08 = (181, "42P08"),
    /// ambiguous_alias
    E42P09 = (182, "42P09"),
    /// invalid_column_reference
    E42P10 = (183, "42P10"),
    /// invalid_cursor_definition
    E42P11 = (184, "42P11"),
    /// invalid_database_definition
    E42P12 = (185, "42P12"),
    /// invalid_function_definition
    E42P13 = (186, "42P13"),
    /// invalid_prepared_statement_definition
    E42P14 = (187, "42P14"),
    /// invalid_schema_definition
    E42P15 = (188, "42P15"),
    /// invalid_table_definition
    E42P16 = (189, "42P16"),
    /// invalid_object_definition
    E42P17 = (190, "42P17"),
    /// indeterminate_datatype
    E42P18 = (191, "42P18"),
    /// invalid_recursion
    E42P19 = (192, "42P19"),
    /// windowing_error
    E42P20 = (193, "42P20"),
    /// collation_mismatch
    E42P21 = (194, "42P21"),
    /// indeterminate_collation
    E42P22 = (195, "42P22"),
    /// with_check_option_violation
    E44000 = (196, "44000"),
    /// insufficient_resources
    E53000 = (197, "53000"),
    /// disk_full
    E53100 = (198, "53100"),
    /// out_of_memory
    E53200 = (199, "53200"),
    /// too_many_connections
    E53300 = (200, "53300"),
    /// configuration_limit_exceeded
    E53400 = (201, "53400"),
    /// program_limit_exceeded
    E54000 = (202, "54000"),
    /// statement_too_complex
    E54001 = (203, "54001"),
    /// too_many_columns
    E54011 = (204, "54011"),
    /// too_many_arguments
    E54023 = (205, "54023"),
    /// object_not_in_prerequisite_state
    E55000 = (206, "55000"),
    /// object_in_use
    E55006 = (207, "55006"),
    /// cant_change_runtime_param
    E55P02 = (208, "55P02"),
    /// lock_not_available
    E55P03 = (209, "55P03"),
    /// unsafe_new_enum_value_usage
    E55P04 = (210, "55P04"),
    /// operator_intervention
    E57000 = (211, "57000"),
    /// query_canceled
    E57014 = (212, "57014"),
    /// admin_shutdown
    E57P01 = (213, "57P01"),
    /// crash_shutdown
    E57P02 = (214, "57P02"),
    /// cannot_connect_now
    E57P03 = (215, "57P03"),
    /// database_dropped
    E57P04 = (216, "57P04"),
    /// idle_session_timeout
    E57P05 = (217, "57P05"),
    /// system_error
    E58000 = (218, "58000"),
    /// io_error
    E58030 = (219, "58030"),
    /// undefined_file
    E58P01 = (220, "58P01"),
    /// duplicate_file
    E58P02 = (221, "58P02"),
    /// snapshot_too_old
    E72000 = (222, "72000"),
    /// config_file_error
    EF0000 = (223, "F0000"),
    /// lock_file_exists
    EF0001 = (224, "F0001"),
    /// fdw_error
    EHV000 = (225, "HV000"),
    /// fdw_out_of_memory
    EHV001 = (226, "HV001"),
    /// fdw_dynamic_parameter_value_needed
    EHV002 = (227, "HV002"),
    /// fdw_invalid_data_type
    EHV004 = (228, "HV004"),
    /// fdw_column_name_not_found
    EHV005 = (229, "HV005"),
    /// fdw_invalid_data_type_descriptors
    EHV006 = (230, "HV006"),
    /// fdw_invalid_column_name
    EHV007 = (231, "HV007"),
    /// fdw_invalid_column_number
    EHV008 = (232, "HV008"),
    /// fdw_invalid_use_of_null_pointer
    EHV009 = (233, "HV009"),
    /// fdw_invalid_string_format
    EHV00A = (234, "HV00A"),
    /// fdw_invalid_handle
    EHV00B = (235, "HV00B"),
    /// fdw_invalid_option_index
    EHV00C = (236, "HV00C"),
    /// fdw_invalid_option_name
    EHV00D = (237, "HV00D"),
    /// fdw_option_name_not_found
    EHV00J = (238, "HV00J"),
    /// fdw_reply_handle
    EHV00K = (239, "HV00K"),
    /// fdw_unable_to_create_execution
    EHV00L = (240, "HV00L"),
    /// fdw_unable_to_create_reply
    EHV00M = (241, "HV00M"),
    /// fdw_unable_to_establish_connection
    EHV00N = (242, "HV00N"),
    /// fdw_no_schemas
    EHV00P = (243, "HV00P"),
    /// fdw_schema_not_found
    EHV00Q = (244, "HV00Q"),
    /// fdw_table_not_found
    EHV00R = (245, "HV00R"),
    /// fdw_function_sequence_error
    EHV010 = (246, "HV010"),
    /// fdw_too_many_handles
    EHV014 = (247, "HV014"),
    /// fdw_inconsistent_descriptor_information
    EHV021 = (248, "HV021"),
    /// fdw_invalid_attribute_value
    EHV024 = (249, "HV024"),
    /// fdw_invalid_string_length_or_buffer_length
    EHV090 = (250, "HV090"),
    /// fdw_invalid_descriptor_field_identifier
    EHV091 = (251, "HV091"),
    /// plpgsql_error
    EP0000 = (252, "P0000"),
    /// raise_exception
    EP0001 = (253, "P0001"),
    /// no_data_found
    EP0002 = (254, "P0002"),
    /// too_many_rows
    EP0003 = (255, "P0003"),
    /// assert_failure
    EP0004 = (256, "P0004"),
    /// internal_error
    EXX000 = (257, "XX000"),
    /// data_corrupted
    EXX001 = (258, "XX001"),
    /// index_corrupted
    EXX002 = (259, "XX002"),
  }
}
