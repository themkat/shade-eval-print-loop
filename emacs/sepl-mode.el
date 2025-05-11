(require 's)
(require 'dash)

(defun sepl--remove-comments (code)
  "Removes comments and newlines from code given in CODE. After execution, all lines will be concatenated to a single line."
  (s-join " "
          (-map (lambda (line)
                  (car (s-split ";" line)))
                (s-lines code))))

(defun sepl-eval-sexp ()
  (interactive)
  (let* ((start (point))
         (end (save-excursion
                (backward-sexp)
                (point)))
         ;; sepl requires single lines
         (code (sepl--remove-comments (buffer-substring start end)))
         (tmp-buf (get-buffer-create "*sepl-tmp-buf*")))
    (when (boundp 'sepl-repl-process)
      (comint-redirect-send-command-to-process code tmp-buf sepl-repl-process nil t)
      (with-current-buffer tmp-buf
        ;; hack to wait for output to be present in tmp buffer
        (sleep-for 0.5)
        (message "=> %s" (s-replace "\n" "\n   " (s-trim (buffer-string))))))
    (kill-buffer tmp-buf)))

;; TODO: how to handle lines with comments?
(defun sepl-eval-buffer ()
  (interactive)
  (when (boundp 'sepl-repl-process)
    (comint-send-string sepl-repl-process
                        (s-replace "\n" " " (buffer-string)))))

(defvar sepl-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-x C-e") 'sepl-eval-sexp)
    map))

;; Mode that connects to the existing sepl process.
;; Sets up hotkeys for evaluation of expressions. 
(define-derived-mode sepl-mode scheme-mode
  "SEPL - Scheme code"
  "Scheme code that we can evaluate in SEPL."

  (set (make-local-variable 'sepl-repl-process)
       (get-buffer-process "*SEPL REPL*")))

(define-derived-mode sepl-repl-mode comint-mode
  "SEPL REPL"
  "Mode used inside the repl to the SEPL process. Dr. Seuss would be proud."

  (setq-local comint-use-prompt-regexp t
              comint-prompt-regexp "> "
              comint-prompt-read-only t)

  (require 'scheme)
  (setq-local font-lock-keywords scheme-font-lock-keywords)

  ;; TODO: completion of built-ins?
  )

(defcustom sepl-program-bin "/Users/marie/Programming/Rust/shade-eval-print-loop/target/debug/shade-eval-print-loop"
  "Path to the SEPL binary"
  :group 'sepl
  :type 'string)

(defun sepl-repl-connect ()
  "Connects to an existing SEPL instance, and starts a REPL interface."
  (interactive)
  (let ((buffer (get-buffer-create "*SEPL REPL*")))
    (with-current-buffer buffer
      (apply 'make-comint-in-buffer "SEPL" buffer '("localhost" . 42069) nil '())
      (sepl-repl-mode)
      (pop-to-buffer buffer))))

(defun sepl-repl-start ()
  "Starts a new SEPL instance and starts a REPL interface."
  (interactive)
  (let ((glsl-file (buffer-file-name (current-buffer))))
    (start-process "sepl" "*SEPL-STDOUT*" sepl-program-bin glsl-file)
    ;; give process 2 seconds to start
    (sleep-for 2)
    (sepl-repl-connect)))

(provide 'sepl-mode)
