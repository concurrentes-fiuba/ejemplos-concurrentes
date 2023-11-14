
println("hello world")


import Pkg

Pkg.add("Calculus")

using Calculus

my_cos = derivative(x -> sin(x))
println(my_cos(π/2))

Pkg.add("Plots")

using Plots

# plot some data
plot([sin, my_cos], 0, 2π)

# save the current figure
savefig("plots.svg")
