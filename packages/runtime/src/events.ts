import type { MusicEvent, PortableSection } from "./types.js";

export function eventsInRange(
  section: PortableSection,
  fromTick: number,
  toTick: number,
  loopOrigin = 0,
): MusicEvent[] {
  const effectiveFromTick = Math.max(fromTick, loopOrigin);
  if (toTick <= effectiveFromTick || section.lengthTicks <= 0) {
    return [];
  }

  const firstLoop = Math.floor(
    (effectiveFromTick - loopOrigin) / section.lengthTicks,
  );
  const lastLoop = Math.floor(
    (toTick - 1 - loopOrigin) / section.lengthTicks,
  );
  const events: MusicEvent[] = [];

  for (let loop = firstLoop; loop <= lastLoop; loop += 1) {
    const offset = loopOrigin + loop * section.lengthTicks;
    for (const event of section.events) {
      const startTick = event.startTick + offset;
      if (startTick < fromTick || startTick >= toTick) {
        continue;
      }
      events.push({
        ...event,
        id: `${event.id}@${loop}`,
        startTick,
      });
    }
  }

  return events.sort(
    (left, right) =>
      left.startTick - right.startTick || left.id.localeCompare(right.id),
  );
}
