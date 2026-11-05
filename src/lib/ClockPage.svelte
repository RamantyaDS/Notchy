<script lang="ts">
  import type { CalEvent } from "./calendar";
  import type { Weather } from "./weather";

  let {
    time,
    date,
    weather,
    nextEvent,
    clockOf,
  }: {
    time: string;
    date: string;
    weather: Weather | null;
    nextEvent: CalEvent | null | undefined;
    clockOf: (t: number) => string;
  } = $props();
</script>

<div class="clock-content">
  <div class="time">{time}</div>
  <div class="date">{date}{#if weather} · {weather.icon} {weather.temp}°{/if}</div>
  {#if nextEvent}
    <div class="event" title={nextEvent.location}>
      <span class="event-time">{clockOf(nextEvent.start)}</span>{nextEvent.title}
    </div>
  {/if}
</div>

<style>
  .clock-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .time {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }
  .date {
    margin-top: 2px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.6);
  }
  .event {
    max-width: 300px;
    margin-top: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(251, 146, 60, 0.14);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .event-time {
    margin-right: 6px;
    color: #fdba74;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
</style>
