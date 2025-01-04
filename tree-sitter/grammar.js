/**
 * @file Flang interpreted language
 * @author Flora Hill <imflo.pink@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports =  grammar({
  name: "flang",
  
  conflicts: $ => [
    [$.term_excl, $.fn_call],
    [$.dyadic, $.terms],
    [$.terms],
    [$.dyadic],
    [$.term_excl, $.args],
    [$.term, $.index],
    [$.index]
  ],

  rules: {
    program: $ => seq(
      repeat($.thing),
    ),

    thing: $ => choice(
      $.return,
      seq(choice($.export, $.uses, $.expr), optional($.semicolon))
    ),

    return: $ => seq(
      'return',
      $.expr,
      optional($.semicolon)
    ),

    export: $ => seq(
      'export',
      $.var_decl
    ),

    uses: $ => seq(
      'uses',
      choice(
        $.identifier,
        '*',
        seq('{', commaSep1($.identifier), '}')
      ),
      'from',
      $.package
    ),

    package: $ => seq(
      $.identifier,
      repeat(seq('::', $.identifier))
    ),

    expr: $ => choice(
      $.struct_inst,
      $.fn_decl,
      $.var_decl,
      $.var_assign,
      $.monadic,
      $.dyadic,
      $.terms
    ),

    struct_inst: $ => seq(
      $.identifier,
      $.lbrace,
      optional(seq($.named_var, repeat(seq($.comma, $.named_var)))),
      $.rbrace
    ),

    fn_outline: $ => seq(
      $.lparen,
      optional($.typed_args),
      $.rparen,
      optional($.typed)
    ),

    fn_decl: $ => seq(
      $.fn_outline,
      '=>',
      $.block
    ),

    var_decl: $ => seq(
      $.declaration,
      $.identifier,
      optional($.typed),
      $.assignment,
      $.expr
    ),

    var_assign: $ => seq(
      $.identifier,
      $.assignment,
      $.expr
    ),

    monadic: $ => seq(
      choice($.negative, $.negate),
      $.term
    ),

    dyadic: $ => seq(
      $.term,
      repeat1(seq(
        choice($.pow, $.equality, $.add, $.subtract, $.multiply, $.divide, $.or, $.and, $.gt, $.lt, $.gte, $.lte),
        $.term
      ))
    ),

    terms: $ => repeat1($.term),

    term: $ => choice(
      $.index,
      $.term_excl
    ),

    term_excl: $ => choice(
      $.fn_call,
      $.literal,
      $.identifier,
      seq($.lparen, $.expr, $.rparen)
    ),

    index: $ => seq(
      $.term_excl,
      repeat1(choice(
        seq('.', $.term),
        seq('[', $.term, ']')
      ))
    ),

    fn_call: $ => seq(
      $.identifier,
      $.lparen,
      optional($.args),
      $.rparen
    ),

    args: $ => seq(
      choice($.expr, $.identifier),
      repeat(seq($.comma, choice($.expr, $.identifier)))
    ),

    block: $ => seq(
      $.lbrace,
      repeat($.thing),
      $.rbrace
    ),

    named_var: $ => seq(
      $.identifier,
      $.colon,
      $.expr
    ),

    typed_var: $ => seq(
      $.identifier,
      $.typed
    ),

    typed_args: $ => seq(
      $.typed_var,
      repeat(seq($.comma, $.typed_var))
    ),

    typed: $ => seq(
      $.colon,
      $.identifier
    ),

    literal: $ => choice(
      $.number,
      $.string,
      $.boolean,
      $.null
    ),

    number: $ => token(choice(
      /\d+/,
      seq(optional(/\d*/), '.', /\d+/)
    )),

    string: $ => seq(
      '"',
      repeat(choice(
        token.immediate(prec(1, /[^"\\]+/)),
        $.escape_sequence
      )),
      '"'
    ),

    escape_sequence: $ => token.immediate(seq(
      '\\',
      choice(
        /["\\\/bfnrt]/,
        seq('u', /[0-9a-fA-F]{4}/)
      )
    )),

    boolean: $ => choice(
      'true',
      'false'
    ),

    null: $ => 'null',

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    lparen: $ => '(',
    rparen: $ => ')',
    lbrace: $ => '{',
    rbrace: $ => '}',
    comma: $ => ',',
    colon: $ => ':',
    semicolon: $ => ';',
    assignment: $ => '=',

    declaration: $ => 'let',

    negate: $ => '!',
    negative: $ => '-',

    pow: $ => '**',
    equality: $ => '==',
    add: $ => '+',
    subtract: $ => '-',
    multiply: $ => '*',
    divide: $ => '/',
    or: $ => '||',
    and: $ => '&&',
    gt: $ => '>',
    lt: $ => '<',
    gte: $ => '>=',
    lte: $ => '<=',
  }

});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}