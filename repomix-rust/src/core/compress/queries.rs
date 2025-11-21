pub const QUERY_RUST: &str = r#"
(line_comment) @comment
(block_comment) @comment

; Import statements
(use_declaration
  (scoped_identifier) @name.reference.module) @definition.import

(use_declaration
  (identifier) @name.reference.module) @definition.import

(extern_crate_declaration
  (identifier) @name.reference.module) @definition.import

; ADT definitions

(struct_item
    name: (type_identifier) @name.definition.class) @definition.class

(enum_item
    name: (type_identifier) @name.definition.class) @definition.class

(union_item
    name: (type_identifier) @name.definition.class) @definition.class

; type aliases

(type_item
    name: (type_identifier) @name.definition.class) @definition.class

; method definitions

(declaration_list
    (function_item
        name: (identifier) @name.definition.method)) @definition.method

; function definitions

(function_item
    name: (identifier) @name.definition.function) @definition.function

; trait definitions
(trait_item
    name: (type_identifier) @name.definition.interface) @definition.interface

; module definitions
(mod_item
    name: (identifier) @name.definition.module) @definition.module

; macro definitions

(macro_definition
    name: (identifier) @name.definition.macro) @definition.macro

; references

(call_expression
    function: (identifier) @name.reference.call) @reference.call

(call_expression
    function: (field_expression
        field: (field_identifier) @name.reference.call)) @reference.call

(macro_invocation
    macro: (identifier) @name.reference.call) @reference.call

; implementations

(impl_item
    trait: (type_identifier) @name.reference.implementation) @reference.implementation

(impl_item
    type: (type_identifier) @name.reference.implementation
    !trait) @reference.implementation
"#;

pub const QUERY_TYPESCRIPT: &str = r#"
(import_statement
  (import_clause (identifier) @name.reference.module)) @definition.import

(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @name.reference.module))) @definition.import)

(comment) @comment

(function_signature
  name: (identifier) @name.definition.function) @definition.function

(method_signature
  name: (property_identifier) @name.definition.method) @definition.method

(abstract_method_signature
  name: (property_identifier) @name.definition.method) @definition.method

(abstract_class_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(module
  name: (identifier) @name.definition.module) @definition.module

(interface_declaration
  name: (type_identifier) @name.definition.interface) @definition.interface

(type_annotation
  (type_identifier) @name.reference.type) @reference.type

(new_expression
  constructor: (identifier) @name.reference.class) @reference.class

(function_declaration
  name: (identifier) @name.definition.function) @definition.function

(method_definition
  name: (property_identifier) @name.definition.method) @definition.method

(class_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(interface_declaration
  name: (type_identifier) @name.definition.class) @definition.class

(type_alias_declaration
  name: (type_identifier) @name.definition.type) @definition.type

(enum_declaration
  name: (identifier) @name.definition.enum) @definition.enum

(lexical_declaration
    (variable_declarator
      name: (identifier) @name.definition.function
      value: (arrow_function)
    )
  ) @definition.function

(variable_declaration
    (variable_declarator
      name: (identifier) @name.definition.function
      value: (arrow_function)
    )
) @definition.function

(assignment_expression
    left: [(identifier) @name.definition.function]
    right: (arrow_function)
) @definition.function
"#;

pub const QUERY_JAVASCRIPT: &str = r#"
(comment) @comment

(method_definition
  name: (property_identifier) @name.definition.method) @definition.method

(class
  name: (_) @name.definition.class) @definition.class
(class_declaration
  name: (_) @name.definition.class) @definition.class

(function_declaration
  name: (identifier) @name.definition.function) @definition.function
(generator_function
  name: (identifier) @name.definition.function) @definition.function
(generator_function_declaration
  name: (identifier) @name.definition.function) @definition.function

(lexical_declaration
  (variable_declarator
    name: (identifier) @name.definition.function
    value: [(arrow_function) (function_declaration)]) @definition.function)

(variable_declaration
  (variable_declarator
    name: (identifier) @name.definition.function
    value: [(arrow_function) (function_declaration)]) @definition.function)

(assignment_expression
  left: [
    (identifier) @name.definition.function
    (member_expression
      property: (property_identifier) @name.definition.function)
  ]
  right: [(arrow_function) (function_declaration)]
) @definition.function

(pair
  key: (property_identifier) @name.definition.function
  value: [(arrow_function) (function_declaration)]) @definition.function

(call_expression
  function: (identifier) @name.reference.call) @reference.call

(call_expression
  function: (member_expression
    property: (property_identifier) @name.reference.call)
  arguments: (_) @reference.call)

(new_expression
  constructor: (_) @name.reference.class) @reference.class
"#;

pub const QUERY_PYTHON: &str = r#"
(comment) @comment

(expression_statement
  (string) @comment) @docstring

; Import statements
(import_statement
  name: (dotted_name) @name.reference.module) @definition.import

(import_from_statement
  module_name: (dotted_name) @name.reference.module) @definition.import

(import_from_statement
  name: (dotted_name) @name.reference.module) @definition.import

(class_definition
  name: (identifier) @name.definition.class) @definition.class

(function_definition
  name: (identifier) @name.definition.function) @definition.function

(call
  function: [
      (identifier) @name.reference.call
      (attribute
        attribute: (identifier) @name.reference.call)
  ]) @reference.call

(assignment
  left: (identifier) @name.definition.type_alias) @definition.type_alias
"#;

pub const QUERY_GO: &str = r#"
; For repomix
(comment) @comment
(package_clause) @definition.package
(import_declaration) @definition.import
(import_spec) @definition.import
(var_declaration) @definition.variable
(const_declaration) @definition.constant

; tree-sitter-go
(function_declaration
  name: (identifier) @name) @definition.function

(method_declaration
  name: (field_identifier) @name) @definition.method

(call_expression
  function: [
    (identifier) @name
    (parenthesized_expression (identifier) @name)
    (selector_expression field: (field_identifier) @name)
    (parenthesized_expression (selector_expression field: (field_identifier) @name))
  ]) @reference.call

(type_spec
  name: (type_identifier) @name) @definition.type

(type_identifier) @name @reference.type

(package_clause "package" (package_identifier) @name)

(type_declaration (type_spec name: (type_identifier) @name type: (interface_type)))

(type_declaration (type_spec name: (type_identifier) @name type: (struct_type)))

; Import statements
(import_declaration
  (import_spec_list
    (import_spec
      path: (interpreted_string_literal) @name.reference.module))) @definition.import

(import_declaration
  (import_spec
    path: (interpreted_string_literal) @name.reference.module)) @definition.import

(package_clause
  (package_identifier) @name.reference.module) @definition.package

(var_declaration (var_spec name: (identifier) @name))

(const_declaration (const_spec name: (identifier) @name))
"#;
