defmodule Greeter do
  use GenServer

  # Client

  def start_link() do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def save(pid, element) do
    GenServer.cast(pid, {:save, element})
  end

  def print(pid) do
    GenServer.call(pid, :print)
  end

  # Server (callbacks)

  def init(init_arg) do
    {:ok, init_arg}
  end

  def handle_cast({:save, content}, acc) do
    IO.puts("save #{content}")
    {:noreply, acc ++ [content]}
  end

  def handle_call(:print, _from, acc) do
    IO.puts("print #{acc}")
    {:reply, "#{acc}", []}
  end
end

{:ok, pid} = Greeter.start_link()

Greeter.save(Greeter, "hello")
Greeter.save(pid, "world")

result = Greeter.print(Greeter)

IO.puts("got result #{result}")

