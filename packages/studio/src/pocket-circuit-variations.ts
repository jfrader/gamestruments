import type { AuthoringScore } from "./export-score.js";
import {
  pocketCircuitRules,
  pocketCircuitScore,
} from "./pocket-circuit-score.js";

export interface ScoreExperiment {
  slug: string;
  shortLabel: string;
  gameType: string;
  genre: string;
  description: string;
  score: AuthoringScore;
}

export const neonHairpinScore = {
  id: "pocket-circuit-neon-hairpin",
  title: "Neon Hairpin",
  bpm: 136,
  beatsPerBar: 4,
  ticksPerBeat: 960,
  crossfadeBars: 2,
  defaultSection: "garage",
  sections: [
    {
      id: "garage",
      label: "Night Garage",
      feeling: "electric / restrained",
      color: "#b890ff",
      bars: 4,
      lanes: [
        { kind: "note", id: "neon-pad", pattern: "<[e3,g3,b3] [c3,e3,g3] [g2,b2,d3] [d3,f#3,a3]>", voice: "warm", velocity: 0.38, gate: 0.95 },
        { kind: "note", id: "sign-glow", pattern: "<[e5 ~ ~ ~ ~ g5 ~ ~] [c5 ~ ~ ~ ~ e5 ~ ~] [g5 ~ ~ ~ ~ b5 ~ ~] [d5 ~ ~ ~ ~ f#5 ~ ~]>", voice: "glass", role: "melody", velocity: 0.42, gate: 0.8 },
        { kind: "note", id: "idle-bass", pattern: "<[e2 ~ b1 ~] [c2 ~ g1 ~] [g2 ~ d2 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.5, gate: 0.88 },
        { kind: "percussion", id: "distant-kit", pattern: "kick ~ hat ~", velocity: 0.25 },
      ],
    },
    {
      id: "grid",
      label: "Neon Grid",
      feeling: "charged / precise",
      color: "#f15bb5",
      bars: 4,
      lanes: [
        { kind: "note", id: "grid-pad", pattern: "<[e3,g3,b3] [c3,e3,g3] [g2,b2,d3] [d3,f#3,a3]>", voice: "warm", velocity: 0.43, gate: 0.92 },
        { kind: "note", id: "laser-count", pattern: "<[e5 ~ ~ ~ ~ g5 ~ ~] [c5 ~ ~ ~ ~ e5 ~ ~] [g5 ~ ~ ~ ~ b5 ~ ~] [d5 ~ ~ ~ ~ f#5 ~ ~]>", voice: "supersaw", role: "melody", velocity: 0.48, gate: 0.55 },
        { kind: "note", id: "grid-bass", pattern: "<[e2 ~ b1 ~] [c2 ~ g1 ~] [g1 ~ d2 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.6, gate: 0.72 },
        { kind: "percussion", id: "grid-kit", pattern: "kick hat snare hat kick hat snare [hat hat]", velocity: 0.52 },
      ],
    },
    {
      id: "cruise",
      label: "Neon Run",
      feeling: "night speed / clarity",
      color: "#00d9ff",
      bars: 4,
      lanes: [
        { kind: "note", id: "wide-chords", pattern: "<[e3,g3,b3] [c3,e3,g3] [g2,b2,d3] [d3,f#3,a3]>", voice: "warm", velocity: 0.48, gate: 0.88 },
        { kind: "note", id: "neon-lead",           pattern: "<[e5 ~ ~ ~ ~ g5 ~ ~] [c5 ~ ~ ~ ~ e5 ~ ~] [g5 ~ ~ ~ ~ b5 ~ ~] [d5 ~ ~ ~ ~ f#5 ~ ~]>", voice: "supersaw", role: "melody", velocity: 0.58, gate: 0.6 },
        { kind: "note", id: "rolling-bass", pattern: "<[e2 ~ b1 ~] [c2 ~ g1 ~] [g1 ~ d2 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.72, gate: 0.65 },
        { kind: "percussion", id: "neon-kit", pattern: "kick hat snare [hat hat] kick hat snare hat", velocity: 0.66 },
      ],
    },
    {
      id: "attack",
      label: "Midnight Duel",
      feeling: "urgent / cinematic",
      color: "#ff477e",
      bars: 4,
      lanes: [
        { kind: "note", id: "duel-chords", pattern: "<[e3,g3,b3] [f3,a3,c4] [d3,f#3,a3] [b2,d3,f#3]>", voice: "pulse", velocity: 0.54, gate: 0.76 },
        { kind: "note", id: "duel-lead", pattern: "<[e5 ~ ~ ~ ~ g5 ~ ~] [c5 ~ ~ ~ ~ e5 ~ ~] [c5 ~ ~ ~ ~ e5 ~ ~] [e5 ~ ~ ~ ~ f#5 ~ ~]>", voice: "supersaw", role: "melody", velocity: 0.66, gate: 0.48 },
        { kind: "note", id: "duel-bass", pattern: "<[e2 ~ b1 ~] [f2 ~ c2 ~] [d2 ~ a1 ~] [b1 ~ f#2 ~]>", voice: "bass", velocity: 0.82, gate: 0.58 },
        { kind: "percussion", id: "duel-kit", pattern: "kick [hat hat] snare hat kick [kick hat] snare [hat hat]", velocity: 0.77 },
      ],
    },
    {
      id: "final-lap",
      label: "Redline Neon",
      feeling: "relentless / luminous",
      color: "#ff1744",
      bars: 4,
      lanes: [
        { kind: "note", id: "redline-chords", pattern: "<[e3,g3,b3] [d3,f#3,a3] [c3,e3,g3] [b2,d#3,f#3]>", voice: "pulse", velocity: 0.62, gate: 0.76 },
        { kind: "note", id: "redline-lead", pattern: "<[e5 ~ ~ ~ ~ g5 ~ ~] [c6 ~ ~ ~ ~ e6 ~ ~] [g5 ~ ~ ~ ~ b5 ~ ~] [d6 ~ ~ ~ ~ a5?0.5 ~ f#5]>", voice: "supersaw", role: "melody", velocity: 0.78, gate: 0.52 },
        { kind: "note", id: "redline-bass", pattern: "<[e2 ~ b1 ~] [d2 ~ a1 ~] [c2 ~ g1 ~] [b1 ~ f#2 ~]>", voice: "bass", velocity: 0.9, gate: 0.56 },
        { kind: "percussion", id: "redline-kit", pattern: "kick [hat hat] snare [hat hat] [kick kick] hat snare [hat hat]", velocity: 0.9 },
      ],
      stemMarkers: [{ lane: "neon-lift", asset: "neon-final-lap-layer.ogg" }],
    },
    {
      id: "victory",
      label: "City Lights",
      feeling: "bright / cinematic release",
      color: "#ffe66d",
      bars: 4,
      lanes: [
        { kind: "note", id: "lights-chords", pattern: "<[e3,g#3,b3] [a3,c#4,e4] [f#3,a3,c#4] [b2,d#3,f#3]>", voice: "warm", velocity: 0.58, gate: 0.94 },
        { kind: "note", id: "lights-lead", pattern: "<[e5 ~ ~ ~ ~ g#5 ~ ~] [b5 ~ ~ ~ ~ g#5 ~ ~] [a5 ~ ~ ~ ~ c#5 ~ ~] [b5 ~ ~ ~ ~ d#5 ~ ~]>", voice: "supersaw", role: "melody", velocity: 0.68, gate: 0.68 },
        { kind: "note", id: "lights-bass", pattern: "<[e2 ~ g#1 ~] [a1 ~ c#2 ~] [f#1 ~ a1 ~] [b1 ~ d#2 ~]>", voice: "bass", velocity: 0.64, gate: 0.82 },
        { kind: "percussion", id: "lights-kit", pattern: "kick hat snare hat kick hat snare [tom tom]", velocity: 0.6 },
      ],
    },
  ],
  rules: pocketCircuitRules,
} satisfies AuthoringScore;

export const pocketFunkScore = {
  id: "pocket-circuit-tiny-torque",
  title: "Tiny Torque",
  bpm: 118,
  beatsPerBar: 4,
  ticksPerBeat: 960,
  crossfadeBars: 2,
  defaultSection: "garage",
  sections: [
    {
      id: "garage",
      label: "Workshop Lounge",
      feeling: "playful / confident",
      color: "#f2cc8f",
      bars: 4,
      lanes: [
        { kind: "note", id: "lounge-keys", pattern: "<[g3,bb3,d4] [f3,a3,c4] [eb3,g3,bb3] [d3,f#3,a3]>", voice: "warm", velocity: 0.4, gate: 0.95 },
        { kind: "note", id: "lounge-hook", pattern: "<[~ ~ g4 ~ ~ ~ ~ bb4] [~ f4 ~ ~ ~ ~ a4 ~] [~ ~ eb4 ~ ~ g4 ~ ~] [d4 ~ ~ ~ ~ ~ f#4 ~]>", voice: "pluck", role: "melody", velocity: 0.46, gate: 0.78 },
        { kind: "note", id: "lounge-bass", pattern: "<[g2 ~ d2 ~] [f2 ~ c2 ~] [eb2 ~ bb1 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.52, gate: 0.7 },
        { kind: "percussion", id: "lounge-kit", pattern: "kick ~ snare hat", velocity: 0.3 },
      ],
    },
    {
      id: "grid",
      label: "Ready To Roll",
      feeling: "cheeky / focused",
      color: "#f4a261",
      bars: 4,
      lanes: [
        { kind: "note", id: "roll-stabs", pattern: "<[g3,bb3,d4] [f3,a3,c4] [eb3,g3,bb3] [d3,f#3,a3]>", voice: "pluck", velocity: 0.52, gate: 0.42 },
        { kind: "note", id: "roll-hook", pattern: "<[~ g5 ~ bb5 ~ ~ d6 ~] [f5 ~ ~ ~ a5 ~ c6 ~] [~ eb5 ~ g5 ~ ~ bb5 ~] [d5 ~ ~ f#5 ~ a5 ~ ~]>", voice: "pluck", role: "melody", velocity: 0.58, gate: 0.6 },
        { kind: "note", id: "roll-bass", pattern: "<[g2 ~ d2 ~] [f2 ~ c2 ~] [eb2 ~ bb1 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.68, gate: 0.55 },
        { kind: "percussion", id: "roll-kit", pattern: "kick hat snare [hat hat] ~ kick snare hat", velocity: 0.54 },
      ],
    },
    {
      id: "cruise",
      label: "Pocket Groove",
      feeling: "elastic / mischievous",
      color: "#81b29a",
      bars: 4,
      lanes: [
        { kind: "note", id: "funk-stabs", pattern: "<[g3,bb3,d4] [f3,a3,c4] [c3,eb3,g3] [d3,f#3,a3]>", voice: "pluck", velocity: 0.5, gate: 0.44 },
        { kind: "note", id: "funk-hook",           pattern: "<[~ g5 ~ ~ bb5 ~ ~ d6] [f5 ~ ~ a5 ~ ~ c6 ~] [~ eb5 ~ ~ g5 ~ ~ bb5] [d5 ~ f#5 ~ ~ a5 ~ ~]>", voice: "pluck", role: "melody", velocity: 0.5, gate: 0.58 },
        { kind: "note", id: "funk-bass", pattern: "<[g2 ~ d2 ~] [f2 ~ c2 ~] [eb2 ~ bb1 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.78, gate: 0.52 },
        { kind: "percussion", id: "funk-kit", pattern: "kick hat snare hat kick [hat hat] snare hat", velocity: 0.65 },
      ],
    },
    {
      id: "attack",
      label: "Bumper Jam",
      feeling: "scrappy / syncopated",
      color: "#e76f51",
      bars: 4,
      lanes: [
        { kind: "note", id: "jam-stabs", pattern: "<[g3,bb3,d4] [ab3,c4,eb4] [f3,a3,c4] [d3,f3,a3]>", voice: "pluck", velocity: 0.58, gate: 0.38 },
        { kind: "note", id: "jam-hook", pattern: "<[~ g5 ~ ab5 ~ ~ d6 ~] [bb5 ~ ~ c5 ~ eb5 ~ ~] [~ f5 ~ ~ g5 ~ c6 ~] [a5 ~ ~ f#5?0.5 ~ d5 ~ ~]>", voice: "pluck", role: "melody", velocity: 0.66, gate: 0.45 },
        { kind: "note", id: "jam-bass", pattern: "<[g2 ~ d2 ~] [ab2 ~ eb2 ~] [f2 ~ c2 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.86, gate: 0.48 },
        { kind: "percussion", id: "jam-kit", pattern: "kick [hat hat] snare hat [kick kick] hat snare [hat hat]", velocity: 0.76 },
      ],
    },
    {
      id: "final-lap",
      label: "Full Torque",
      feeling: "wild / joyful pressure",
      color: "#e63946",
      bars: 4,
      lanes: [
        { kind: "note", id: "torque-stabs", pattern: "<[g3,bb3,d4] [f3,a3,c4] [eb3,g3,bb3] [d3,f#3,a3]>", voice: "pluck", velocity: 0.66, gate: 0.38 },
        { kind: "note", id: "torque-hook", pattern: "<[~ g5 bb5 ~ d6 ~ ~ f6] [f5 ~ a5 ~ c6 ~ ~ a5] [~ eb6 g6 ~ bb6 ~ ~ g6] [d6 ~ f#5 ~ a5 ~ ~ d6]>", voice: "pluck", role: "melody", velocity: 0.78, gate: 0.48 },
        { kind: "note", id: "torque-bass", pattern: "<[g2 ~ d2 ~] [f2 ~ c2 ~] [eb2 ~ bb1 ~] [d2 ~ a1 ~]>", voice: "bass", velocity: 0.92, gate: 0.46 },
        { kind: "percussion", id: "torque-kit", pattern: "kick [hat hat] snare [hat hat] [kick kick] hat snare [tom tom]", velocity: 0.9 },
      ],
      stemMarkers: [{ lane: "horn-lift", asset: "funk-final-lap-layer.ogg" }],
    },
    {
      id: "victory",
      label: "Victory Lap",
      feeling: "sunny / loose",
      color: "#f6bd60",
      bars: 4,
      lanes: [
        { kind: "note", id: "victory-stabs", pattern: "<[g3,b3,d4] [c4,e4,g4] [a3,c4,e4] [d3,f#3,a3]>", voice: "warm", velocity: 0.58, gate: 0.8 },
        { kind: "note", id: "victory-hook", pattern: "<[g5 ~ ~ b5 ~ d6 ~ ~] [c6 ~ e6 ~ ~ g6 ~ ~] [~ a5 ~ c6 ~ e6 ~ ~] [d6 ~ ~ f#5 ~ a5 ~ ~]>", voice: "pluck", role: "melody", velocity: 0.65, gate: 0.65 },
        { kind: "note", id: "victory-bass", pattern: "<[g2 ~ b1 ~] [c2 ~ e2 ~] [a1 ~ c#2 ~] [d2 ~ f#1 ~]>", voice: "bass", velocity: 0.66, gate: 0.72 },
        { kind: "percussion", id: "victory-kit", pattern: "kick hat snare hat kick hat snare [tom tom]", velocity: 0.58 },
      ],
    },
  ],
  rules: pocketCircuitRules,
} satisfies AuthoringScore;

export const microMotorScore = {
  id: "pocket-circuit-micro-motor",
  title: "Micro Motor Panic",
  bpm: 152,
  beatsPerBar: 4,
  ticksPerBeat: 960,
  crossfadeBars: 2,
  defaultSection: "garage",
  sections: [
    {
      id: "garage",
      label: "Title Screen",
      feeling: "tiny / curious",
      color: "#9bf6ff",
      bars: 4,
      lanes: [
        { kind: "note", id: "title-chords", pattern: "<[c4,eb4,g4] [bb3,d4,f4] [ab3,c4,eb4] [g3,b3,d4]>", voice: "chip", velocity: 0.35, gate: 0.82 },
        { kind: "note", id: "title-beep", pattern: "<[~ c5 ~ ~ eb5 ~ ~ ~] [~ ~ bb4 ~ d5 ~ ~ ~] [~ ab4 ~ ~ c5 ~ ~ ~] [~ ~ g4 ~ ~ b4 ~ ~]>", voice: "chip", role: "melody", velocity: 0.38, gate: 0.42 },
        { kind: "note", id: "title-bass", pattern: "<[c2 ~ g1 ~] [bb1 ~ f1 ~] [ab1 ~ eb1 ~] [g1 ~ d2 ~]>", voice: "triangle", velocity: 0.48, gate: 0.72 },
        { kind: "percussion", id: "title-kit", pattern: "kick ~ hat ~", velocity: 0.24 },
      ],
    },
    {
      id: "grid",
      label: "Insert Coin",
      feeling: "bright / impatient",
      color: "#a0c4ff",
      bars: 4,
      lanes: [
        { kind: "note", id: "coin-arp", pattern: "<[~ c6 ~ ~ eb6 ~ ~ g5] [~ ~ bb5 ~ ~ d6 ~ ~] [~ ab5 ~ ~ c6 ~ ~ ~] [~ ~ g5 ~ ~ b5 ~ c6]>", voice: "chip", role: "melody", velocity: 0.45, gate: 0.42 },
        { kind: "note", id: "coin-chords", pattern: "<[c4,eb4,g4] [ab3,c4,eb4] [bb3,d4,f4] [g3,b3,d4]>", voice: "chip", velocity: 0.38, gate: 0.52 },
        { kind: "note", id: "coin-bass", pattern: "<[c2 ~ g1 ~] [ab1 ~ eb1 ~] [bb1 ~ f1 ~] [g1 ~ d2 ~]>", voice: "triangle", velocity: 0.6, gate: 0.6 },
        { kind: "percussion", id: "coin-kit", pattern: "kick hat snare hat kick hat snare [hat hat]", velocity: 0.48 },
      ],
    },
    {
      id: "cruise",
      label: "Pixel Sprint",
      feeling: "nimble / arcade",
      color: "#bdb2ff",
      bars: 4,
      lanes: [
        { kind: "note", id: "pixel-arp",           pattern: "<[~ c6 ~ ~ eb6 ~ g5 ~] [~ ~ bb5 ~ ~ d6 ~ ~] [~ ab5 ~ ~ c6 ~ ~ ~] [~ ~ g5 ~ b5 ~ ~ d6]>", voice: "chip", role: "melody", velocity: 0.54, gate: 0.42 },
        { kind: "note", id: "pixel-counter", pattern: "<[g4 ~ ~ ~] [c5 ~ ~ ~] [ab4 ~ ~ ~] [bb4 ~ ~ ~]>", voice: "chip", velocity: 0.38, gate: 0.52 },
        { kind: "note", id: "pixel-bass", pattern: "<[c2 ~ g1 ~] [bb1 ~ f1 ~] [ab1 ~ eb1 ~] [g1 ~ d2 ~]>", voice: "triangle", velocity: 0.7, gate: 0.52 },
        { kind: "percussion", id: "pixel-kit", pattern: "kick hat snare [hat hat] kick hat snare hat", velocity: 0.62 },
      ],
    },
    {
      id: "attack",
      label: "Rival Alert",
      feeling: "glitchy / competitive",
      color: "#ff70a6",
      bars: 4,
      lanes: [
        { kind: "note", id: "alert-arp", pattern: "<[~ c6 ~ ~ eb6 ~ g5 ~] [~ ~ eb6 ~ ~ ~ bb5 ~] [~ bb5 ~ ~ c6 ~ ~ ~] [~ ~ ~ d6 ~ bb5?0.5 ~ g5]>", voice: "chip", role: "melody", velocity: 0.64, gate: 0.38 },
        { kind: "note", id: "alert-chords", pattern: "<[c4,eb4,g4] [db4,f4,ab4] [bb3,d4,f4] [g3,bb3,d4]>", voice: "chip", velocity: 0.46, gate: 0.48 },
        { kind: "note", id: "alert-bass", pattern: "<[c2 ~ g1 ~] [db2 ~ ab1 ~] [bb1 ~ f1 ~] [g1 ~ d2 ~]>", voice: "triangle", velocity: 0.8, gate: 0.48 },
        { kind: "percussion", id: "alert-kit", pattern: "kick [hat hat] snare hat [kick kick] hat snare [hat hat]", velocity: 0.76 },
      ],
    },
    {
      id: "final-lap",
      label: "One Life Left",
      feeling: "frantic / heroic",
      color: "#ff4d6d",
      bars: 4,
      lanes: [
        { kind: "note", id: "life-arp", pattern: "<[~ c6 ~ ~ eb6 ~ ~ g5] [~ ~ bb6 ~ ~ d6 ~ ~] [~ ab6 ~ ~ c7 ~ ~ ~] [~ ~ g6 ~ b6 ~ ~ ~]>", voice: "chip", role: "melody", velocity: 0.76, gate: 0.36 },
        { kind: "note", id: "life-counter", pattern: "<[g5 ~ ~ ~] [c6 ~ ~ ~] [f5 ~ ~ ~] [bb5 ~ ~ ~]>", voice: "chip", velocity: 0.5, gate: 0.42 },
        { kind: "note", id: "life-bass", pattern: "<[c2 ~ g1 ~] [bb1 ~ f1 ~] [ab1 ~ eb1 ~] [g1 ~ d2 ~]>", voice: "triangle", velocity: 0.88, gate: 0.48 },
        { kind: "percussion", id: "life-kit", pattern: "kick [hat hat] snare [hat hat] [kick kick] hat snare [tom tom]", velocity: 0.9 },
      ],
      stemMarkers: [{ lane: "extra-life", asset: "chip-final-lap-layer.ogg" }],
    },
    {
      id: "victory",
      label: "High Score",
      feeling: "sparkling / triumphant",
      color: "#fdffb6",
      bars: 4,
      lanes: [
        { kind: "note", id: "score-arp", pattern: "<[~ c6 ~ ~ e6 ~ ~ ~] [~ ~ f5 ~ a5 ~ ~ ~] [~ d6 ~ ~ f6 ~ ~ ~] [~ ~ g5 ~ b5 ~ ~ ~]>", voice: "chip", role: "melody", velocity: 0.6, gate: 0.5 },
        { kind: "note", id: "score-chords", pattern: "<[c4,e4,g4] [f4,a4,c5] [d4,f4,a4] [g3,b3,d4]>", voice: "chip", velocity: 0.42, gate: 0.66 },
        { kind: "note", id: "score-bass", pattern: "<[c2 ~ g1 ~] [f1 ~ c2 ~] [d2 ~ a1 ~] [g1 ~ d2 ~]>", voice: "triangle", velocity: 0.62, gate: 0.68 },
        { kind: "percussion", id: "score-kit", pattern: "kick hat snare hat kick hat snare [tom tom]", velocity: 0.55 },
      ],
    },
  ],
  rules: pocketCircuitRules,
} satisfies AuthoringScore;

export const pocketCircuitExperiments = [
  {
    slug: "countertop-velocity",
    shortLabel: "Fusion",
    gameType: "Racing",
    genre: "Electronic fusion",
    description: "Polished keys, precise drums, and clean melodic motion.",
    score: pocketCircuitScore,
  },
  {
    slug: "neon-hairpin",
    shortLabel: "Neon",
    gameType: "Racing",
    genre: "Synthwave",
    description: "Night-drive bass, luminous leads, and cinematic pressure.",
    score: neonHairpinScore,
  },
  {
    slug: "tiny-torque",
    shortLabel: "Funk",
    gameType: "Racing",
    genre: "Pocket funk",
    description: "Elastic bass, syncopated stabs, and playful momentum.",
    score: pocketFunkScore,
  },
  {
    slug: "micro-motor-panic",
    shortLabel: "Chip",
    gameType: "Racing",
    genre: "Chiptune",
    description: "Fast pixel arpeggios and miniature arcade urgency.",
    score: microMotorScore,
  },
] satisfies readonly ScoreExperiment[];
