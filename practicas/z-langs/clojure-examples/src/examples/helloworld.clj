(ns examples.helloworld)

(defn Example [to-add]
   (println (str "Hello " "World"))
   (println (+ 1 to-add)))

(defn -main []
   (Example 4))

