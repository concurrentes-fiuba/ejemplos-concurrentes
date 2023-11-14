defmodule Greeter.Agent do
  use Agent

  def start_link(initial_value) do
    Agent.start_link(fn -> initial_value end, name: __MODULE__)
  end

  def get() do
    Agent.get(__MODULE__, & &1)
  end

  def update(new_value) do
    Agent.update(__MODULE__, fn _state -> new_value end)
  end
end

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

  def init(_init_arg) do
    current = Greeter.Agent.get()
    {:ok, current}
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

  def terminate(_reason, acc) do
    Greeter.Agent.update(acc)
  end

end

{:ok, _sup} = Supervisor.start_link([Greeter.Agent, Greeter], strategy: :one_for_one)

Greeter.save(Greeter, "hello")
Greeter.save(Greeter, "world")
Greeter.save(Greeter, "goodbye")

Process.sleep(1000)

result = Greeter.print(Greeter)

IO.puts("got result #{result}")

