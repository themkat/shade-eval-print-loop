;; Mode that connects to the existing sepl process.
;; Sets up hotkeys for evaluation of expressions. 
(define-derived-mode sepl-mode scheme-mode
  "SEPL - Scheme code"
  "Scheme code that we can evaluate in SEPL."
  )

;; TODO: maybe a function for starting the program, directly from Emacs?
;;       then we can automatically start the repl and just connect from scheme processes?

;; TODO: maybe some sort of error message if we try to start two proceses? Only one allowed at the moment. To make life simpler for myself. 
(define-derived-mode sepl-repl-mode comint-mode
  "SEPL REPL"
  "Mode used inside the repl to the SEPL process. Dr. Seuss would be proud."

  ;; TODO: rest of the buffer should be read only, except the prompt
  ;; (setq-local comint-use-prompt-regexp t
  ;;             comint-prompt-regexp "> ")

  ;; TODO: could we activate highlight of scheme keywords?
  (require 'scheme)
  (setq-local font-lock-keywords scheme-font-lock-keywords)

  
  ;; TODO: set some key bindings in buffer.
  ;;       C-x C-e should eval current sexp etc.
  ;;       something for eval buffer :)
)

(defcustom sepl-program-bin "/Users/marie/Programming/Rust/shade-eval-print-loop/target/debug/shade-eval-print-loop"
  "Path to the SEPL binary"
  :group 'sepl
  :type 'string)

(defun sepl-repl-start ()
  (interactive)
  (let ((buffer (get-buffer-create "*SEPL REPL*"))
        (glsl-file (buffer-file-name (current-buffer))))
    (start-process "sepl" "*SEPL-STDOUT*" sepl-program-bin glsl-file)
    ;; give process 2 seconds to start
    (sleep-for 2)
    ;; TODO: open the comint buffer in a split.
    ;; TODO: setup the special mode for the buffer.
    ;;    
    (apply 'make-comint-in-buffer "SEPL" buffer '("localhost" . 42069) nil '())))


;; TODO: some keywords we cna 
