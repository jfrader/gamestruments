declare module "@strudel/mini" {
  interface StrudelFraction {
    valueOf(): number;
  }

  interface StrudelSpan {
    begin: StrudelFraction;
    end: StrudelFraction;
  }

  interface StrudelHap {
    whole?: StrudelSpan;
    part: StrudelSpan;
    value: unknown;
    hasOnset(): boolean;
  }

  interface StrudelPattern {
    queryArc(
      begin: number,
      end: number,
      controls?: Record<string, unknown>,
    ): StrudelHap[];
  }

  export function mini(...source: string[]): StrudelPattern;
}
