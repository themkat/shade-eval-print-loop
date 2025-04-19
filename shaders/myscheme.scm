(set-uniform! "myuni" 1.0)

(define screen_dim (screen-size))

(* (cadr screen_dim) 1.0)

(set-uniform! "screen_width" (* 1.0 (car screen_dim)))
(set-uniform! "screen_height" (* 1.0 (cadr screen_dim)))


;;(set-uniform! "noise" (noise 200 200 34))
;;(set-uniform! "mytex" (load-texture "sdfsdf"))

;;(change-mesh! SQUARE)


(define (square x)
  (* x x))

(square 3)

(matrix '(1.0 0.0 0.0 0.0)
        '(0.0 1.0 0.0 0.0)
        '(0.0 0.0 1.0 0.0)
        '(0.0 0.0 0.0 1.0))

(set-dynamic-uniform! "elapsed_time"
                      (lambda () (get-elapsed-time)))
(delete-dynamic-uniform! "elapsed_time")

(set-dynamic-uniform! "screen_width")

DYNAMIC_UNIFORM_TABLE
