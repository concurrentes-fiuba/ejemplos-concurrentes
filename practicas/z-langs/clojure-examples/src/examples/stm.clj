(ns examples.stm)

(def account1 (ref 100))
(def account2 (ref 0))

(defn transfer [id]
  (future
    (dosync
      (print (str id " - " (deref account1) "\n"))
      (alter account1 - 1)
      (alter account2 + 1))))


(defn -main []
  (doall (map deref (map transfer (range 100))))
  (println "Terminó!")
  (println (deref account1))
  (println (deref account2)))
