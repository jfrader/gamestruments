// Display copy for the Racing Genres view. This used to live in the retired
// Studio package; the lab only needs the labels, so they live with the app now.
export interface RacingGenreExperiment {
  slug: string;
  shortLabel: string;
  genre: string;
  description: string;
  scoreTitle: string;
}

export const racingGenreExperiments: readonly RacingGenreExperiment[] = [
  {
    slug: "countertop-velocity",
    shortLabel: "Fusion",
    genre: "Electronic fusion",
    description: "Polished keys, precise drums, and clean melodic motion.",
    scoreTitle: "Countertop Velocity",
  },
  {
    slug: "neon-hairpin",
    shortLabel: "Neon",
    genre: "Synthwave",
    description: "Night-drive bass, luminous leads, and cinematic pressure.",
    scoreTitle: "Neon Hairpin",
  },
  {
    slug: "tiny-torque",
    shortLabel: "Funk",
    genre: "Pocket funk",
    description: "Elastic bass, syncopated stabs, and playful momentum.",
    scoreTitle: "Tiny Torque",
  },
  {
    slug: "micro-motor-panic",
    shortLabel: "Chip",
    genre: "Chiptune",
    description: "Fast pixel arpeggios and miniature arcade urgency.",
    scoreTitle: "Micro Motor Panic",
  },
];
