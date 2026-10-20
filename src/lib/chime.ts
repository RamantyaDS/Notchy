// A soft two-note chime, synthesized so there's no sound file to ship.
let ctx: AudioContext | null = null;

export function chime() {
  ctx ??= new AudioContext();
  const start = ctx.currentTime;
  for (const [i, freq] of [880, 1318.5].entries()) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = "sine";
    osc.frequency.value = freq;
    const t = start + i * 0.18;
    gain.gain.setValueAtTime(0, t);
    gain.gain.linearRampToValueAtTime(0.18, t + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.9);
    osc.connect(gain).connect(ctx.destination);
    osc.start(t);
    osc.stop(t + 0.95);
  }
}
