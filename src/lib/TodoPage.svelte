<script lang="ts">
  import { tick } from "svelte";
  import { dayKey, type Task, type Todo } from "./productivity.svelte";
  import { getLocale, t } from "./i18n";

  // `onTyping` asks Rust for keyboard focus (the notch normally never takes it).
  let { todo, onTyping }: { todo: Todo; onTyping: (typing: boolean) => void } = $props();

  // null: not editing; 0: a new task; otherwise the id being edited.
  let editing = $state<number | null>(null);
  let text = $state("");
  let time = $state("");
  let input = $state<HTMLInputElement>();

  // Week strip: today and the six days after it. Picking a day lists its tasks,
  // and new tasks go to that day.
  const today = dayKey();
  let day = $state(today);
  const week = Array.from({ length: 7 }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() + i);
    return { key: dayKey(d), num: d.getDate(), name: d.toLocaleDateString(getLocale() === "id" ? "id-ID" : "en-US", { weekday: "narrow" }) };
  });
  const tasks = $derived(todo.forDay(day));
  const busy = $derived(new Set(week.filter((w) => w.key !== day && todo.forDay(w.key).some((x) => !x.done)).map((w) => w.key)));

  async function edit(task?: Task) {
    editing = task?.id ?? 0;
    text = task?.text ?? "";
    time = task?.time ?? "";
    onTyping(true);
    await tick();
    // Give Windows a moment to hand the notch keyboard focus.
    setTimeout(() => input?.focus(), 80);
  }

  function finish(keep: boolean) {
    if (keep && editing === 0) todo.add(text, time, day);
    else if (keep && editing) todo.update(editing, text, time);
    editing = null;
    onTyping(false);
  }

  function keys(e: KeyboardEvent) {
    if (e.key === "Enter") finish(true);
    if (e.key === "Escape") finish(false);
  }

  // Leaving the form (click elsewhere) saves what was typed.
  function blur(e: FocusEvent) {
    const next = e.relatedTarget as Node | null;
    if (!(e.currentTarget as HTMLElement).contains(next)) finish(text.trim() !== "");
  }
</script>

<!-- Left: how the day is going, and the add button. Right: the tasks. -->
<div class="page">
  <div class="side">
    <div class="count"><b>{tasks.filter((x) => x.done).length}</b>/{tasks.length}</div>
    <div class="caption">{day === today ? t("todo.doneToday") : t("todo.doneThatDay")}</div>
    {#if editing === null}
      <button class="add" onclick={() => edit()}>{t("todo.add")}</button>
    {/if}
  </div>

  <div class="list">
  <div class="week" role="tablist">
    {#each week as w (w.key)}
      <button role="tab" aria-selected={w.key === day} class:on={w.key === day} class:busy={busy.has(w.key)} onclick={() => (day = w.key)}>
        <small>{w.name}</small>{w.num}
      </button>
    {/each}
  </div>
  {#if editing !== null}
    <div class="form" onfocusout={blur}>
      <input bind:this={input} bind:value={text} onkeydown={keys} placeholder={t("todo.placeholder")} class="text" />
      <input bind:value={time} onkeydown={keys} type="time" class="time" title={t("todo.timeTitle")} />
      <button onclick={() => finish(true)}>{t("todo.save")}</button>
    </div>
  {/if}

  <ul class="scroll">
    {#each tasks.filter((t) => t.id !== editing) as task (task.id)}
      <li class:done={task.done}>
        <button class="check" onclick={() => todo.toggle(task.id)} aria-label={task.done ? t("todo.markUndone") : t("todo.markDone")}>
          {#if task.done}✓{/if}
        </button>
        <button class="label" onclick={() => edit(task)}>{task.text}</button>
        {#if task.time}<span class="chip">{task.time}</span>{/if}
        <button class="remove" onclick={() => todo.remove(task.id)} aria-label={t("todo.delete")}>×</button>
      </li>
    {:else}
      {#if editing === null}<li class="empty">{t("todo.empty")}</li>{/if}
    {/each}
  </ul>
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    gap: 16px;
    padding: 10px 40px 20px;
  }
  .side {
    flex: none;
    width: 100px;
    padding-top: 2px;
  }
  .count {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 16px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.5);
    font-variant-numeric: tabular-nums;
  }
  .count b {
    font-size: 28px;
    color: #fff;
  }
  .caption {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.5);
  }
  .side .add {
    margin-top: 10px;
    background: #fff;
    color: #000;
    font-weight: 600;
  }
  .side .add:hover {
    background: #dfe3ee;
  }
  .list {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  button {
    border: 0;
    background: none;
    color: #fff;
    font: inherit;
    cursor: pointer;
  }
  .add,
  .form button {
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    font-size: 11.5px;
  }
  .add:hover,
  .form button:hover {
    background: rgba(255, 255, 255, 0.22);
  }
  .week {
    display: flex;
    gap: 3px;
    margin-bottom: 5px;
  }
  .week button {
    position: relative;
    flex: 1;
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: 3px;
    padding: 2px 0;
    border-radius: 7px;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: rgba(255, 255, 255, 0.7);
  }
  .week small {
    font-size: 9px;
    color: rgba(255, 255, 255, 0.4);
  }
  .week button:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .week button.on {
    background: #fff;
    color: #000;
    font-weight: 600;
  }
  .week button.on small {
    color: rgba(0, 0, 0, 0.5);
  }
  /* A dot under days that still have open tasks. */
  .week button.busy::after {
    content: "";
    position: absolute;
    bottom: 0;
    left: 50%;
    width: 3px;
    height: 3px;
    margin-left: -1.5px;
    border-radius: 50%;
    background: #fdba74;
  }
  .form {
    display: flex;
    gap: 6px;
    margin-bottom: 4px;
  }
  .form input {
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.06);
    color: #fff;
    font: inherit;
    font-size: 12.5px;
    padding: 4px 8px;
    color-scheme: dark;
  }
  .form .text {
    flex: 1;
    min-width: 0;
  }
  .form .time {
    width: 92px;
  }
  .form input:focus {
    outline: 1px solid #2d5bff;
  }
  ul {
    flex: 1;
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    scrollbar-width: none;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    padding: 4px 9px;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.07);
    font-size: 12.5px;
  }
  /* Done tasks: a green pill, struck through. */
  li.done {
    background: rgba(52, 211, 153, 0.14);
  }
  li.empty {
    background: none;
    padding: 4px 0;
  }
  .check {
    flex: none;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 1.5px solid rgba(255, 255, 255, 0.45);
    border-radius: 50%;
    font-size: 10px;
    line-height: 1;
  }
  .done .check {
    border-color: #34d399;
    background: #34d399;
    color: #000;
  }
  .label {
    flex: 1;
    min-width: 0;
    padding: 0;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .done .label {
    color: rgba(255, 255, 255, 0.4);
    text-decoration: line-through;
  }
  .chip {
    flex: none;
    padding: 1px 7px;
    border-radius: 999px;
    background: rgba(251, 146, 60, 0.18);
    color: #fdba74;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }
  .remove {
    flex: none;
    padding: 0 2px;
    color: rgba(255, 255, 255, 0.35);
    font-size: 15px;
    opacity: 0;
  }
  li:hover .remove {
    opacity: 1;
  }
  .remove:hover {
    color: #f87171;
  }
  .empty {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.45);
  }
</style>
