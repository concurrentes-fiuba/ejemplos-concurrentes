(ns examples.swap)

(def account (atom 100))

(defn dec-and-print [current]
  (println current)
  (dec current))

(defn extract [ignore]
  (future
    (swap! account dec-and-print)))


(defn -main []
  (doall (map deref (map extract (range 100))))
  (println (deref account)))