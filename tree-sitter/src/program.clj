(program [0, 0] - [4, 37]
  (thing [0, 0] - [2, 2]
    (export [0, 0] - [2, 1]
      (var_decl [0, 7] - [2, 1]
        (declaration [0, 7] - [0, 10])
        (identifier [0, 11] - [0, 26])
        (assignment [0, 27] - [0, 28])
        (expr [0, 29] - [2, 1]
          (fn_decl [0, 29] - [2, 1]
            (fn_outline [0, 29] - [0, 51]
              (lparen [0, 29] - [0, 30])
              (typed_args [0, 30] - [0, 50]
                (typed_var [0, 30] - [0, 39]
                  (identifier [0, 30] - [0, 31])
                  (typed [0, 31] - [0, 39]
                    (colon [0, 31] - [0, 32])
                    (identifier [0, 33] - [0, 39])))
                (comma [0, 39] - [0, 40])
                (typed_var [0, 41] - [0, 50]
                  (identifier [0, 41] - [0, 42])
                  (typed [0, 42] - [0, 50]
                    (colon [0, 42] - [0, 43])
                    (identifier [0, 44] - [0, 50]))))
              (rparen [0, 50] - [0, 51]))
            (block [0, 55] - [2, 1]
              (lbrace [0, 55] - [0, 56])
              (thing [1, 4] - [1, 16]
                (return [1, 4] - [1, 16]
                  (expr [1, 11] - [1, 16]
                    (dyadic [1, 11] - [1, 16]
                      (term [1, 11] - [1, 12]
                        (term_excl [1, 11] - [1, 12]
                          (identifier [1, 11] - [1, 12])))
                      (add [1, 13] - [1, 14])
                      (term [1, 15] - [1, 16]
                        (term_excl [1, 15] - [1, 16]
                          (identifier [1, 15] - [1, 16])))))))
              (rbrace [2, 0] - [2, 1]))))))
    (semicolon [2, 1] - [2, 2]))
  (thing [4, 0] - [4, 37]
    (export [4, 0] - [4, 36]
      (var_decl [4, 7] - [4, 36]
        (declaration [4, 7] - [4, 10])
        (identifier [4, 11] - [4, 30])
        (assignment [4, 31] - [4, 32])
        (expr [4, 33] - [4, 36]
          (terms [4, 33] - [4, 36]
            (term [4, 33] - [4, 36]
              (term_excl [4, 33] - [4, 36]
                (literal [4, 33] - [4, 36]
                  (number [4, 33] - [4, 36]))))))))
    (semicolon [4, 36] - [4, 37])))