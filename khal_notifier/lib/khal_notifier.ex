defmodule KhalNotifier do
  @moduledoc """
  Sync calendars with vdirsyncer, then send notify-send
  alerts for upcoming khal events.
  """
  require Logger

  @notify_icon "calendar"

  def main(args) do
    {opts, _, _} =
      OptionParser.parse(args,
        switches: [lookahead: :string, urgency: :string, skip_sync: :boolean, clean: :boolean],
        aliases: []
      )

    lookahead = Keyword.get(opts, :lookahead, "15m")
    urgency = Keyword.get(opts, :urgency, "critical")
    skip_sync = Keyword.get(opts, :skip_sync, false)

    unless skip_sync do
      unless sync_vdirsyncer() do
        Logger.warning("Sync failed — continuing with local calendar data")
      end
    end

    events = get_upcoming_events(lookahead)

    events = filter_events(events)

    if Enum.empty?(events) do
      Logger.info("No upcoming events")
    else
      Enum.each(events, fn event ->
        summary = event.title
        body = format_event(event)
        Logger.info("Notifying: #{summary} — #{body}")
        send_notification(summary, body, urgency)
      end)
    end
  end

  defp filter_events(events) do
    {now_time, 0} = System.cmd("date", ["+%H:%M"])
    now_time = String.trim(now_time)

    filtered =
      Enum.reject(events, fn event ->
        event.start_time != "" and event.start_time <= now_time
      end)

    filtered
  end

  defp run_cmd(cmd, args, timeout \\ 30_000) do
    Logger.debug("Running: #{cmd} #{Enum.join(args, " ")}")

    case System.find_executable(cmd) do
      nil ->
        {:error, :enoent}

      path ->
        try do
          task = Task.async(fn -> System.cmd(path, args, stderr_to_stdout: true) end)

          case Task.yield(task, timeout) || Task.shutdown(task) do
            {:ok, {output, exit_code}} ->
              {:ok, output, exit_code}

            nil ->
              {:error, :timeout}
          end
        rescue
          e -> {:error, e}
        end
    end
  end

  defp sync_vdirsyncer do
    Logger.info("Syncing calendars with vdirsyncer")

    case run_cmd("vdirsyncer", ["sync"], 60_000) do
      {:ok, _output, 0} ->
        Logger.info("vdirsyncer sync complete")
        true

      {:ok, output, code} ->
        Logger.warning("vdirsyncer exited with code #{code}: #{String.trim(output)}")
        false

      {:error, :enoent} ->
        Logger.error("vdirsyncer not found")
        false

      {:error, :timeout} ->
        Logger.error("vdirsyncer timed out after 60 seconds")
        false

      {:error, e} ->
        Logger.error("vdirsyncer failed: #{inspect(e)}")
        false
    end
  end

  defp get_upcoming_events(lookahead) do
    Logger.info("Checking khal for events in the next #{lookahead}...")

    case run_cmd("khal", [
           "list",
           "now",
           lookahead,
           "--format",
           "{start-time}|{end-time}|{title}|{organizer}",
           "--day-format",
           ""
         ]) do
      {:ok, output, 0} ->
        parse_events(output)

      {:ok, output, code} ->
        Logger.warning("khal exited with code #{code}: #{String.trim(output)}")
        []

      {:error, :enoent} ->
        Logger.error("khal not found. Is it installed and on PATH?")
        []

      {:error, :timeout} ->
        Logger.error("khal timed out.")
        []

      {:error, _e} ->
        Logger.error("khal execution failed.")
        []
    end
  end

  defp parse_events(output) do
    events =
      output
      |> String.split("\n", trim: true)
      |> Enum.map(&String.trim/1)
      |> Enum.reject(&(&1 == ""))
      |> Enum.map(fn line ->
        case String.split(line, "|", parts: 4) do
          [start_time, end_time, title, organizer] ->
            %{
              start_time: String.trim(start_time),
              end_time: String.trim(end_time),
              title: String.trim(title),
              organizer: String.trim(organizer)
            }

          [start_time, end_time, title] ->
            %{
              start_time: String.trim(start_time),
              end_time: String.trim(end_time),
              title: String.trim(title),
              organizer: ""
            }

          [start_time, title] ->
            %{
              start_time: String.trim(start_time),
              end_time: "",
              title: String.trim(title),
              organizer: ""
            }

          [title] ->
            %{
              start_time: "",
              end_time: "",
              title: String.trim(title),
              organizer: ""
            }
        end
      end)
      |> Enum.reject(&(&1.title == ""))

    Logger.info("Found #{length(events)} upcoming event(s).")
    events
  end

  defp format_event(event) do
    time_str =
      case {event.start_time, event.end_time} do
        {"", ""} -> "All day"
        {start, ""} -> start
        {start, end_t} -> "#{start} - #{end_t}"
      end

    if Map.has_key?(event, :organizer) and event.organizer != "" do
      "#{time_str}\nOrganizer: #{event.organizer}"
    else
      time_str
    end
  end

  defp send_notification(summary, body, urgency) do
    args = [
      "--icon",
      @notify_icon,
      "--urgency",
      urgency,
      summary,
      body
    ]

    case run_cmd("notify-send", args) do
      {:ok, _output, 0} ->
        true

      {:ok, output, code} ->
        Logger.error("notify-send failed (code #{code}): #{String.trim(output)}")
        false

      {:error, :enoent} ->
        Logger.error("notify-send not found")
        false

      {:error, _} ->
        Logger.error("notify-send failed")
        false
    end
  end
end
