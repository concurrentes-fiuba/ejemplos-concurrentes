defmodule Greeter do

  def start() do
    spawn(fn -> listen([]) end)
  end

  def listen(acc) do
    receive do
      {:save, content} ->
        IO.puts("save #{content}")
        listen(acc ++ [content])
      {:print} ->
        IO.puts("print #{acc}")
        listen([])
    after 5000 ->
      IO.puts("timeout !")
    end
  end

end

greeter = Greeter.start()

send(greeter, {:save, "hello"})
send(greeter, {:save, "world"})
send(greeter, {:save, "mas"})
send(greeter, {:save, "cosas"})
send(greeter, {:print})

send(greeter, {:save, "despues"})
send(greeter, {:print})

send(greeter, {:cualquiercosa})

receive do after 10000 -> IO.puts("saliendo") end
