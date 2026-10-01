import {
  type PortableScore,
  type PortableSection,
  type SectionId,
} from "../../../packages/runtime/src/index.ts";
import {
  score,
  cueSection,
  setFormHold,
  setSoloMode,
  soloMode,
  toggleEngine,
  playback,
} from "./state.ts";
import { isDebugBarSection } from "./playback-section.ts";

function formatVoice(voice: string): string {
  return voice.replace(/-/g, " ");
}

export function renderDebuggerGrid(): void {
  const phaseSelect = document.getElementById("debugger-phase-select") as HTMLSelectElement;
  const content = document.getElementById("debugger-content");
  if (!phaseSelect || !content || !score) return;

  const currentSections = score.sections.map((s) => s.id).filter((id) => !isDebugBarSection(id));
  const existingOptions = Array.from(phaseSelect.options).map((o) => o.value);

  if (existingOptions.join(",") !== currentSections.join(",")) {
    phaseSelect.replaceChildren(
      ...currentSections.map((id) => {
        const section = score.sections.find(s => s.id === id)!;
        const option = document.createElement("option");
        option.value = section.id;
        option.textContent = section.label;
        return option;
      })
    );
  }

  const selectedPhaseId = phaseSelect.value;
  if (!selectedPhaseId) return;

  const section = score.sections.find((s) => s.id === selectedPhaseId);
  if (!section) return;

  renderPhaseGrid(score, section, content);
}

function renderPhaseGrid(
  currentScore: PortableScore,
  section: PortableSection,
  container: HTMLElement
) {
  const barTicks = currentScore.beatsPerBar * currentScore.ticksPerBeat;
  const totalBars = section.lengthTicks / barTicks;
  
  // Determine if we need 16th or 8th subdivisions.
  // 8th notes happen every ticksPerBeat / 2.
  // 16th notes happen every ticksPerBeat / 4.
  const is16th = section.events.some(e => e.startTick % (currentScore.ticksPerBeat / 2) !== 0);
  const stepsPerBeat = is16th ? 4 : 2;
  const stepsPerBar = currentScore.beatsPerBar * stepsPerBeat;
  const ticksPerStep = currentScore.ticksPerBeat / stepsPerBeat;

  container.innerHTML = "";

  const voices = Array.from(
    new Set(
      section.events
        .filter((e) => e.kind === "note" || e.kind === "percussion")
        .map((e) => (e.kind === "note" || e.kind === "percussion" ? e.voice : ""))
    )
  ).filter(Boolean);

  voices.sort();

  for (let bar = 0; bar < totalBars; bar++) {
    const barStartTick = bar * barTicks;
    const barEndTick = barStartTick + barTicks;
    const barEvents = section.events.filter(
      (e) => e.startTick >= barStartTick && e.startTick < barEndTick
    );

    const barContainer = document.createElement("div");
    barContainer.className = "debugger-bar-container";

    const header = document.createElement("div");
    header.className = "debugger-bar-header";
    header.innerHTML = `<h3>Bar ${bar + 1}</h3>`;

    const playBarBtn = document.createElement("button");
    playBarBtn.type = "button";
    playBarBtn.textContent = "Play bar";
    playBarBtn.onclick = () => playBar(section.id, bar);
    header.appendChild(playBarBtn);

    const table = document.createElement("table");
    table.className = "debugger-grid";

    // Table Header
    const thead = document.createElement("thead");
    const headerRow = document.createElement("tr");
    headerRow.innerHTML = `<th>Voice</th><th>Controls</th>`;
    
    // Subdivisions
    for (let i = 0; i < stepsPerBar; i++) {
      const beat = Math.floor(i / stepsPerBeat) + 1;
      const sub = i % stepsPerBeat;
      let label = "";
      if (stepsPerBeat === 2) {
        label = sub === 0 ? `${beat}` : "&";
      } else {
        label = sub === 0 ? `${beat}` : sub === 1 ? "e" : sub === 2 ? "&" : "a";
      }
      headerRow.innerHTML += `<th>${label}</th>`;
    }
    headerRow.innerHTML += `<th>Hits</th><th>Range</th><th>Vel</th>`;
    thead.appendChild(headerRow);
    table.appendChild(thead);

    const tbody = document.createElement("tbody");

    for (const voice of voices) {
      const voiceEvents = barEvents.filter(
        (e) => (e.kind === "note" || e.kind === "percussion") && e.voice === voice
      );

      const row = document.createElement("tr");
      
      const voiceCell = document.createElement("td");
      voiceCell.textContent = formatVoice(voice);
      row.appendChild(voiceCell);

      const controlsCell = document.createElement("td");
      const soloBtn = document.createElement("button");
      soloBtn.type = "button";
      soloBtn.textContent = "Solo";

      const muteBtn = document.createElement("button");
      muteBtn.type = "button";
      muteBtn.textContent = "Mute";

      const voiceSolo = typeof soloMode === "object" ? soloMode : null;
      const soloed = voiceSolo !== null && voiceSolo.voice === voice && !voiceSolo.mute;
      const muted = voiceSolo !== null && voiceSolo.voice === voice && voiceSolo.mute;
      soloBtn.setAttribute("aria-pressed", String(soloed));
      muteBtn.setAttribute("aria-pressed", String(muted));

      const apply = (mute: boolean): void => {
        setSoloMode({ voice, mute });
        soloBtn.setAttribute("aria-pressed", String(!mute));
        muteBtn.setAttribute("aria-pressed", String(mute));
      };
      soloBtn.onclick = () => apply(false);
      muteBtn.onclick = () => apply(true);

      controlsCell.appendChild(soloBtn);
      controlsCell.appendChild(muteBtn);
      row.appendChild(controlsCell);

      let hits = 0;
      let minVel = 127;
      let maxVel = 0;
      let firstTick = -1;
      let lastTick = -1;

      for (let i = 0; i < stepsPerBar; i++) {
        const stepStartTick = barStartTick + i * ticksPerStep;
        const stepEndTick = stepStartTick + ticksPerStep;
        
        const stepEvents = voiceEvents.filter(
          (e) => e.startTick >= stepStartTick && e.startTick < stepEndTick
        );

        const cell = document.createElement("td");
        if (stepEvents.length > 0) {
          cell.textContent = voice.charAt(0).toUpperCase();
          cell.title = stepEvents.map(e => `Vel: ${Math.round(e.velocity * 127)}`).join(", ");
          cell.className = "debugger-cell-hit";
          
          hits += stepEvents.length;
          for (const e of stepEvents) {
            const vel = Math.round(e.velocity * 127);
            if (vel < minVel) minVel = vel;
            if (vel > maxVel) maxVel = vel;
            if (firstTick === -1 || e.startTick < firstTick) firstTick = e.startTick - barStartTick;
            if (e.startTick > lastTick) lastTick = e.startTick - barStartTick;
          }
        }
        row.appendChild(cell);
      }

      const hitsCell = document.createElement("td");
      hitsCell.textContent = hits > 0 ? hits.toString() : "-";
      row.appendChild(hitsCell);

      const rangeCell = document.createElement("td");
      rangeCell.textContent = hits > 0 ? `${firstTick} - ${lastTick}` : "-";
      row.appendChild(rangeCell);

      const velCell = document.createElement("td");
      velCell.textContent = hits > 0 ? (minVel === maxVel ? `${minVel}` : `${minVel}-${maxVel}`) : "-";
      row.appendChild(velCell);

      tbody.appendChild(row);
    }
    
    table.appendChild(tbody);
    barContainer.appendChild(header);
    barContainer.appendChild(table);
    container.appendChild(barContainer);
  }
}

export function setupScoreDebugger(): void {
  const phaseSelect = document.getElementById("debugger-phase-select");
  if (phaseSelect) {
    phaseSelect.addEventListener("change", renderDebuggerGrid);
  }

  const cueBtn = document.getElementById("debugger-cue-btn");
  if (cueBtn) {
    cueBtn.addEventListener("click", () => {
      const phaseSelect = document.getElementById("debugger-phase-select") as HTMLSelectElement;
      if (phaseSelect && phaseSelect.value) {
        setSoloMode("full");
        setFormHold(false);
        cueSection(phaseSelect.value);
        if (!playback.running) {
          toggleEngine();
        }
      }
    });
  }
}

function playBar(sectionId: SectionId, barIndex: number): void {
  const section = score.sections.find((s) => s.id === sectionId);
  if (!section) return;

  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  const startTick = barIndex * barTicks;
  const endTick = startTick + barTicks;

  const barSectionId = `${sectionId}-bar-${barIndex}`;
  const sections = score.sections as PortableSection[];

  // The bar slice lives on the shared score, and the engine reloads it from
  // there. Keep exactly one: drop whatever a previous Play bar injected.
  for (let index = sections.length - 1; index >= 0; index -= 1) {
    const candidate = sections[index];
    if (candidate !== undefined && isDebugBarSection(candidate.id)) {
      sections.splice(index, 1);
    }
  }
  sections.push({
    ...section,
    id: barSectionId,
    label: `${section.label} (Bar ${barIndex + 1})`,
    lengthTicks: barTicks,
    events: section.events
      .filter((e) => e.startTick >= startTick && e.startTick < endTick)
      .map((e) => ({
        ...e,
        startTick: e.startTick - startTick,
        section: barSectionId,
      })),
  });
  playback.reloadScore();

  setFormHold(true);
  cueSection(barSectionId);
  if (!playback.running) {
    toggleEngine();
  }
}
