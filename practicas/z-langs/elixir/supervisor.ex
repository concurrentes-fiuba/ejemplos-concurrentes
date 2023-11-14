defmodule Greeter do
  use GenServer

  # Client

  def start_link(initial_arg) do
    IO.puts("starting Greeter #{inspect self()} with #{inspect initial_arg}")
    GenServer.start_link(__MODULE__, initial_arg, name: __MODULE__)
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

  def handle_cast({:save, content}, _acc) when content == "goodbye" do
    raise "goodbye"
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

{:ok, _sup} = Supervisor.start_link([Greeter], strategy: :one_for_one)

Greeter.save(Greeter, "hello")
Greeter.save(Greeter, "world")
Greeter.save(Greeter, "goodbye")

 Process.sleep(1000)

 result = Greeter.print(Greeter)

 IO.puts("got result #{result}")

