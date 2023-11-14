(ns examples.agent)

(def account (agent 100))

(defn extract [ignore]
  (send account dec))

(defn -main []
  (doall (map extract (range 100)))
  (println (deref account))
  (await account)
  (println (deref account)))
