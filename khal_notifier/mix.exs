defmodule KhalNotifier.MixProject do
  use Mix.Project

  def project do
    [
      app: :khal_notifier,
      version: "0.1.0",
      elixir: "~> 1.19",
      start_permanent: Mix.env() == :prod,
      escript: escript(),
      releases: releases(),
      deps: deps()
    ]
  end

  def releases do
    [
      khal_notifier: [
        steps: [:assemble, &Burrito.wrap/1],
        burrito: [
          targets: [
            linux: [os: :linux, cpu: :x86_64]
          ]
        ]
      ]
    ]
  end

  defp escript do
    [main_module: KhalNotifier]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {KhalNotifier.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:burrito, "~> 1.0"}
    ]
  end
end
