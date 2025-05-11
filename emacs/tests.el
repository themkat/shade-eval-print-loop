;; Basic unit tests for Elisp code
(require 'sepl-mode)
(require 's)

(ert-deftest remove-comments-single-line-test ()
  (should (s-equals? "hi there"
                     (sepl--remove-comments "hi there")))
  (should (s-equals? "(+ 2 3)"
                     (sepl--remove-comments "(+ 2 3)")))
  (should (s-equals? ""
                     (sepl--remove-comments "; Hi there"))))

(ert-deftest remove-comments-multiple-lines-test ()
  (should (s-equals? "(+ 1 2) 2 (define (square x)   (* x x))"
                     (sepl--remove-comments "(+ 1 2)\n2\n(define (square x)\n  (* x x))"))))
