; Keywords
[
  "abort"
  "acquires"
  "as"
  "const"
  "copy"
  "else"
  "entry"
  "false"
  "friend"
  "fun"
  "if"
  "invariant"
  "let"
  "loop"
  "module"
  "move"
  "native"
  "public"
  "return"
  "script"
  "spec"
  "struct"
  "true"
  "use"
  "while"
  "has"
  "inline"
  "enum"
  "match"
  "for"
  "in"
  "where"
] @keyword

(line_comment) @comment
(block_comment) @comment

(line_comment (doc_comment)) @comment.documentation
(block_comment (doc_comment)) @comment.documentation

; Punctuation
[
  "::"
  ";"
  ","
  "."
  ":"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

; Angle brackets (for generics)
[
  "<"
  ">"
] @punctuation.bracket

; Attributes
(attribute
  attribute: (_) @attribute)

; Special identifiers
((identifier) @keyword.storage.type
  (#match? @keyword.storage.type "^(Self|vector|signer|address|u8|u16|u32|u64|u128|u256|bool)$"))

; Builtin functions
((identifier) @function.builtin
  (#match? @function.builtin "^(assert|move_to|move_from|borrow_global|borrow_global_mut|exists)$"))
