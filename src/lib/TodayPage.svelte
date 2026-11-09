<script lang="ts">
  import type { CalEvent } from "./calendar";
  import { dayKey, type Todo, type Water } from "./productivity.svelte";
  import type { Weather } from "./weather";
  import { t } from "./i18n";

  // The day at a glance: next meeting and weather on the left, today's tasks
  // and water on the right. Everything here already lives in other pages.
  let {
    todo,
    water,
    weather,
    nextEvent,
    clockOf,
    waterOn,
    countdowns = [],
  }: {
    todo: Todo;
    water: Water;
    weather: Weather | null;
    nextEvent: CalEvent | null | undefined;
    clockOf: (t: number) => string;
    waterOn: boolean;
    /** Upcoming countdowns, nearest first. */
    countdowns?: { name: string; days: number }[];
  } = $props();
  const daysText = (n: number) => (n === 0 ? t("today.isToday") : n === 1 ? t("today.day") : t("today.days", { n }));

  const tasks = $derived(todo.forDay(dayKey()));
  const open = $derived(tasks.filter((x) => !x.done));
  const goal = $derived(water.settings?.goalMl ?? 2000);
  const drunk = $derived(water.today());
</script>

<div class="page">
  <div class="side">
    <div class="label">{t("today.next")}</div>
    {#if nextEvent}
      <div class="event" title={nextEvent.title}>
        <b>{clockOf(nextEvent.start)}</b>
        <span>{nextEvent.title}</span>
      </div>
    {:else}
      <div class="none">{t("today.noEvent")}</div>
    {/if}
    {#if weather}
      <div class="weather">{weather.icon} <b>{weather.temp}°</b></div>
    {/if}
    {#each countdowns.slice(0, weather ? 1 : 2) as c (c.name)}
      <div class="countdown" class:today={c.days === 0} title={c.name}>
        <b>{daysText(c.days)}</b>
        <span>{c.name}</span>
      </div>
    {/each}
  </div>

  <div class="main">
    <div class="label">
      {t("today.tasks", { n: open.length })}
    </div>
    <ul>
      {#each open.slice(0, 3) as task (task.id)}
        <li>
          <button class="check" onclick={() => todo.toggle(task.id)} aria-label={t("todo.markDone")}></button>
          <span class="text">{task.text}</span>
          {#if task.time}<span class="chip">{task.time}</span>{/if}
        </li>
      {:else}
        <li class="none">{tasks.length ? t("today.allDone") : t("today.noTasks")}</li>
      {/each}
      {#if open.length > 3}<li class="more">+{open.length - 3}</li>{/if}
    </ul>
    {#if waterOn}
      <div class="water" title="{drunk} / {goal} ml">
        <span>💧</span>
        <div class="bar"><i style:width="{Math.min(100, (drunk / goal) * 100)}%"></i></div>
        <small>{(drunk / 1000).toFixed(1)} / {(goal / 1000).toFixed(1)} L</small>
      </div>
    {/if}
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    gap: 18px;
    padding: 12px 40px 20px;
    font-size: 12.5px;
  }
  .side {
    flex: none;
    width: 118px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .label {
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.45);
    margin-bottom: 2px;
  }
  .countdown {
    display: flex;
    gap: 5px;
    align-items: baseline;
    min-width: 0;
    font-size: 11.5px;
  }
  .countdown b {
    flex: none;
    color: #fbbf24;
  }
  .countdown.today b {
    color: #34d399;
  }
  .countdown span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    color: rgba(255, 255, 255, 0.7);
  }
  .event {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .event b {
    font-size: 18px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .event span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    color: rgba(255, 255, 255, 0.75);
  }
  .weather {
    margin-top: auto;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.75);
  }
  .none {
    color: rgba(255, 255, 255, 0.45);
    font-size: 12px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 20px;
  }
  .check {
    flex: none;
    width: 13px;
    height: 13px;
    padding: 0;
    border: 1.5px solid rgba(255, 255, 255, 0.45);
    border-radius: 50%;
    background: none;
    cursor: pointer;
  }
  .check:hover {
    border-color: #34d399;
    background: rgba(52, 211, 153, 0.25);
  }
  .text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .chip {
    flex: none;
    padding: 0 6px;
    border-radius: 999px;
    background: rgba(251, 146, 60, 0.18);
    color: #fdba74;
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
  }
  .more {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.45);
    padding-left: 20px;
  }
  .water {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 11px;
  }
  .bar {
    flex: 1;
    height: 4px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.12);
    overflow: hidden;
  }
  .bar i {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: #38bdf8;
  }
  .water small {
    color: rgba(255, 255, 255, 0.55);
    font-variant-numeric: tabular-nums;
  }
</style>
