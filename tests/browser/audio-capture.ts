import type { Page } from "@playwright/test";

interface AudioCapture {
  firstStart: number | null;
  sampleRate: number;
  kicks: { time: number; scheduledAt: number }[];
  hats: { time: number; scheduledAt: number }[];
  blocks: { time: number; samples: number[] }[];
  finishRecording?: () => Promise<string>;
}

declare global { interface Window { scanAudio: AudioCapture } }

export async function installAudioCapture(page: Page, record = false): Promise<void> {
  await page.addInitScript((record) => {
    const capture: AudioCapture = { firstStart: null, sampleRate: 0, kicks: [], hats: [], blocks: [] };
    window.scanAudio = capture;
    const Native = window.AudioContext;
    window.AudioContext = class extends Native {
      constructor(options?: AudioContextOptions) {
        super(options);
        capture.sampleRate = this.sampleRate;
        const connections = new WeakMap<AudioNode, AudioNode>();
        const gains = new WeakMap<GainNode, { value: number; time: number }[]>();
        const trace = (node: AudioNode): void => {
          const connect = node.connect.bind(node);
          node.connect = ((destination: AudioNode | AudioParam, output: number = 0, input: number = 0) => {
            if (destination instanceof AudioNode) { connections.set(node, destination); return connect(destination, output, input); }
            connect(destination, output);
          }) as typeof node.connect;
        };
        const createGain = this.createGain.bind(this);
        this.createGain = () => {
          const node = createGain(); trace(node);
          const values: { value: number; time: number }[] = [];
          gains.set(node, values);
          const set = node.gain.setValueAtTime.bind(node.gain);
          node.gain.setValueAtTime = (value, time) => { values.push({ value, time }); return set(value, time); };
          const ramp = node.gain.linearRampToValueAtTime.bind(node.gain);
          node.gain.linearRampToValueAtTime = (value, time) => { values.push({ value, time }); return ramp(value, time); };
          const cancel = node.gain.cancelScheduledValues.bind(node.gain);
          node.gain.cancelScheduledValues = (time) => {
            const retained = values.filter((entry) => entry.time < time);
            values.splice(0, values.length, ...retained);
            return cancel(time);
          };
          const hold = node.gain.cancelAndHoldAtTime?.bind(node.gain);
          if (hold) node.gain.cancelAndHoldAtTime = (time) => {
            const value = node.gain.value;
            const retained = values.filter((entry) => entry.time < time);
            values.splice(0, values.length, ...retained, { value, time });
            return hold(time);
          };
          return node;
        };
        const open = (source: AudioNode, time: number): boolean => {
          let node = connections.get(source);
          for (let depth = 0; node && depth < 6; depth++) {
            if (node instanceof GainNode) {
              const values = (gains.get(node) ?? []).filter((entry) => entry.time <= time);
              const value = values.sort((left, right) => left.time - right.time).at(-1)?.value ?? node.gain.value;
              if (value === 0) return false;
            }
            node = connections.get(node);
          }
          return true;
        };
        const tap = this.createScriptProcessor(1024, 1, 1);
        const mute = this.createGain(); mute.gain.value = 0;
        tap.connect(mute).connect(this.destination);
        tap.onaudioprocess = (event) => {
          capture.blocks.push({ time: event.playbackTime, samples: Array.from(event.inputBuffer.getChannelData(0)) });
          if (capture.blocks.length > 500) capture.blocks.splice(0, 100);
        };
        const recording = record ? this.createMediaStreamDestination() : null;
        if (recording) {
          const recorder = new MediaRecorder(recording.stream, { mimeType: "audio/webm;codecs=opus" });
          const chunks: Blob[] = [];
          recorder.ondataavailable = (event) => chunks.push(event.data);
          recorder.start();
          capture.finishRecording = () => new Promise((resolve, reject) => {
            recorder.onstop = () => {
              const reader = new FileReader();
              reader.onload = () => resolve(String(reader.result).split(",")[1]!);
              reader.onerror = () => reject(reader.error);
              reader.readAsDataURL(new Blob(chunks, { type: recorder.mimeType }));
            };
            recorder.stop();
          });
        }
        const createCompressor = this.createDynamicsCompressor.bind(this);
        this.createDynamicsCompressor = () => {
          const node = createCompressor(); const connect = node.connect.bind(node);
          node.connect = ((destination: AudioNode | AudioParam, output: number = 0, input: number = 0) => {
            if (destination === this.destination) { connect(tap); if (recording) connect(recording); }
            if (destination instanceof AudioNode) return connect(destination, output, input);
            connect(destination, output);
          }) as typeof node.connect;
          return node;
        };
        const createOscillator = this.createOscillator.bind(this);
        this.createOscillator = () => {
          const node = createOscillator(); trace(node); let kick = false;
          const ramp = node.frequency.exponentialRampToValueAtTime.bind(node.frequency);
          node.frequency.exponentialRampToValueAtTime = (value, time) => { if (value === 49) kick = true; return ramp(value, time); };
          const start = node.start.bind(node);
          node.start = (time = 0) => {
            capture.firstStart ??= time;
            if (kick && open(node, time)) capture.kicks.push({ time, scheduledAt: this.currentTime });
            start(time);
          };
          return node;
        };
        const createFilter = this.createBiquadFilter.bind(this);
        this.createBiquadFilter = () => { const filter = createFilter(); trace(filter); return filter; };
        const createSource = this.createBufferSource.bind(this);
        this.createBufferSource = () => {
          const node = createSource(); trace(node); const start = node.start.bind(node);
          node.start = (time = 0, offset = 0, duration?: number) => {
            capture.firstStart ??= time;
            const filter = connections.get(node);
            if (filter instanceof BiquadFilterNode && filter.frequency.value === 6200 && open(node, time)) {
              capture.hats.push({ time, scheduledAt: this.currentTime });
            }
            start(time, offset, duration);
          };
          return node;
        };
      }
    };
  }, record);
}
