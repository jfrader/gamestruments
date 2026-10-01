/** The AudioWorklet processor that plays the engine's rendered audio. */
export const PCM_QUEUE_PROCESSOR = "gamestruments-pcm-queue";

/** Render quanta (128 frames each) between two played-frame reports. */
export const PCM_REPORT_QUANTA = 16;

/** How many queued frames the processor has played, at its clock time. */
export interface PcmPlayedReport {
  played: number;
  time: number;
}
